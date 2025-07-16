use core::fmt;
use std::{
    collections::BTreeMap, fmt::Display, io::Write, os::unix::fs::MetadataExt, path::PathBuf,
};

use sha1::{Digest, Sha1};

use crate::{
    git_rust::{BASE_DIR, RepoRust},
    objects::blob::Blob,
};

#[derive(Default)]
pub struct Index {
    pub header: IndexHeader,
    // (path, IndexEntry)
    pub entries: BTreeMap<String, IndexEntry>,
}

#[derive(Default)]
pub struct IndexHeader {
    pub sign: [u8; 4],
    pub version: [u8; 4],
    pub entries: [u8; 4],
}

impl From<u32> for IndexHeader {
    fn from(entries: u32) -> Self {
        let sign = [b'D', b'I', b'R', b'C'];
        Self {
            sign,
            version: 2_u32.to_be_bytes(),
            entries: entries.to_be_bytes(),
        }
    }
}

impl IndexHeader {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();

        buf.extend(&self.sign);
        buf.extend(&self.version);
        buf.extend(&self.entries);

        buf
    }

    fn header(file: &[u8]) -> std::io::Result<Self> {
        if file.len() < 12 {
            return Err(std::io::Error::other("Invalid header."));
        }
        let sign: [u8; 4] = file[0..4].try_into().unwrap();
        let version: [u8; 4] = file[4..8].try_into().unwrap();
        let entries: [u8; 4] = file[8..12].try_into().unwrap();
        let header = Self {
            sign,
            version,
            entries,
        };
        Ok(header)
    }
}

#[derive(Debug, Clone)]
pub struct IndexEntry {
    pub ctime: u32,
    pub ctime_nanos: u32,
    pub mtime: u32,
    pub mtime_nanos: u32,
    pub dev: u32,
    pub ino: u32,
    pub mode: u32,
    pub uid: u32,
    pub gid: u32,
    pub file_size: u32,
    pub sha1: [u8; 20],
    pub flags: u16,
    pub path: Vec<u8>,
}

impl IndexEntry {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();

        // Write fixed-size fields
        buf.extend(&self.ctime.to_be_bytes());
        buf.extend(&self.ctime_nanos.to_be_bytes());
        buf.extend(&self.mtime.to_be_bytes());
        buf.extend(&self.mtime_nanos.to_be_bytes());
        buf.extend(&self.dev.to_be_bytes());
        buf.extend(&self.ino.to_be_bytes());
        buf.extend(&self.mode.to_be_bytes());
        buf.extend(&self.uid.to_be_bytes());
        buf.extend(&self.gid.to_be_bytes());
        buf.extend(&self.file_size.to_be_bytes());
        buf.extend(&self.sha1);
        buf.extend(&self.flags.to_be_bytes());

        // Write path name as null-terminated string (not padded yet)
        buf.extend(self.path.clone());
        buf.push(0); // Null terminator

        // Pad to 8-byte alignment
        let padding = (8 - (buf.len() % 8)) % 8;
        buf.extend(vec![0u8; padding]);
        buf
    }

    pub fn from_bytes(buf: &[u8]) -> std::io::Result<(Self, usize)> {
        use std::convert::TryInto;

        if buf.len() < 62 {
            return Err(std::io::Error::other("Invalid IndexEntry (too short)."));
        }

        let ctime = u32::from_be_bytes(buf[0..4].try_into().unwrap());
        let ctime_nanos = u32::from_be_bytes(buf[4..8].try_into().unwrap());
        let mtime = u32::from_be_bytes(buf[8..12].try_into().unwrap());
        let mtime_nanos = u32::from_be_bytes(buf[12..16].try_into().unwrap());
        let dev = u32::from_be_bytes(buf[16..20].try_into().unwrap());
        let ino = u32::from_be_bytes(buf[20..24].try_into().unwrap());
        let mode = u32::from_be_bytes(buf[24..28].try_into().unwrap());
        let uid = u32::from_be_bytes(buf[28..32].try_into().unwrap());
        let gid = u32::from_be_bytes(buf[32..36].try_into().unwrap());
        let file_size = u32::from_be_bytes(buf[36..40].try_into().unwrap());
        let sha1: [u8; 20] = buf[40..60].try_into().unwrap();
        let flags = u16::from_be_bytes(buf[60..62].try_into().unwrap());

        // File size is in the last 12 bits of the 2 byte flags
        let name_len = (flags & 0x0FFF) as usize;
        // 62 bytes already read
        let name_start = 62;
        // Add the length of the name
        let name_end = name_start + name_len;

        if buf.len() <= name_end {
            return Err(std::io::Error::other("Path data exceeds buffer"));
        }

        let path = buf[62..name_end].to_vec();

        if buf[name_end] != 0 {
            return Err(std::io::Error::other("Path is not null-terminated"));
        }

        // Add the null termination
        let base_size = name_end + 1;

        let padding = (8 - (base_size % 8)) % 8;
        let total_size = base_size + padding;

        if buf.len() < total_size {
            return Err(std::io::Error::other("Entry buffer too small for padding"));
        }

        Ok((
            Self {
                ctime,
                ctime_nanos,
                mtime,
                mtime_nanos,
                dev,
                ino,
                mode,
                uid,
                gid,
                file_size,
                sha1,
                flags,
                path,
            },
            total_size,
        ))
    }
}

impl Index {
    fn header(&self) -> &IndexHeader {
        &self.header
    }

    fn from_entries(entries: BTreeMap<String, IndexEntry>) -> Self {
        let header = IndexHeader::from(entries.len() as u32);
        Self { header, entries }
    }

    // TODO: Compare metadata when file already exists in index
    // git add
    // Create the index if it doesn't exist, or creates a new one
    // Creates blobs for each object, but not trees.
    // Checks if blobs and entries already exist.
    //
    // A blob can exist on disk but not in the index (git hash-object -w)
    // A blob cannot exist in the index, but not on disk
    // 1. Get entry - index the file
    // 2. Check if blob already exists on disk
    //  If yes - Move on (to Check if it exists in index)
    //  If no - Create the file. Move on (to Check if it exists in index)
    // 3. Check if blob exists in index
    //      A. Path exists and SHA1 is different         -> Update index. Persist change
    //      B. Path does not exist in index              -> Add to index. Persist change
    //      C. Path exists and SHA1 is same              -> Move on
    pub fn build_index(path: String) -> std::io::Result<()> {
        let root = &RepoRust::get_root().base_path;
        // let mut entries: BTreeMap<String, IndexEntry> = BTreeMap::new();
        // if root.join(BASE_DIR).join("INDEX").exists() {
        //     entries = Self::read_index()?.entries;
        // }
        let mut entries = if root.join(BASE_DIR).join("INDEX").exists() {
            Self::read_index()?.entries
        } else {
            BTreeMap::new()
        };
        let path = PathBuf::from(path);
        let mut stack = vec![path];
        while let Some(current_path) = stack.pop() {
            if current_path.is_file() {
                // 1. Get entry - index the file
                let entry = Self::index_entry_from_file(&current_path)?;
                let key: String = current_path.to_string_lossy().into();

                // 2. Check if blob already exists on disk
                let blob_exists = Blob::blob_exists(entry.sha1);
                if !blob_exists {
                    // If no - Create the file
                    let file = std::fs::read(current_path)?;
                    let blob = Blob::blob_with_sha1(file.as_slice());
                    blob.write_object_to_file(file.as_slice())?;
                } // If yes, move on

                // 3. Check if blob exists in index
                match entries.get(&key) {
                    // A.Path does not exist in index -> Add to index
                    None => {
                        entries.insert(key, entry.clone());
                    }
                    // B. Path exists and SHA1 is different -> Update index
                    Some(existing_entry) if existing_entry.sha1 != entry.sha1 => {
                        entries.insert(key, entry.clone());
                    }
                    // C. Path exists and SHA1 is same
                    _ => {}
                }
            } else if current_path.is_dir() {
                if current_path.ends_with(".git_rust") {
                    continue;
                }

                for entry in std::fs::read_dir(current_path)? {
                    let entry = entry?.path();
                    stack.push(entry);
                }
            }
        }

        // Create and update the index
        let index = Self::from_entries(entries);
        index.write_index_to_file()?;
        Ok(())
    }

    pub fn sha1_entry(file: &[u8]) -> [u8; 20] {
        let mut hasher = Sha1::new();
        hasher.update(format!("blob {}\0", file.len()).as_bytes());
        hasher.update(file);
        let result = hasher.finalize();
        result.into()
    }

    pub fn index_entry_from_file(path: &PathBuf) -> std::io::Result<IndexEntry> {
        let metadata = path.metadata()?;
        let ctime = metadata.ctime() as u32;
        let ctime_nanos = metadata.ctime_nsec() as u32;
        let mtime = metadata.mtime() as u32;
        let mtime_nanos = metadata.mtime_nsec() as u32;
        let dev = metadata.dev() as u32;
        let ino = metadata.ino() as u32;
        let mode = metadata.mode();
        let uid = metadata.uid();
        let gid = metadata.gid();
        let file_size = metadata.size() as u32;

        let file = std::fs::read(path)?;
        let sha1 = Self::sha1_entry(file.as_slice());

        let path_str = path.to_string_lossy();
        let name_len = path_str.len();
        let assume_valid = 0_u8;
        let extended = 0_u8;
        let stage = 0b00;
        let name_len_field = if name_len >= 0xFFF {
            0xFFF
        } else {
            u16::try_from(name_len)
                .map_err(|_| std::io::Error::other("Could not index file name"))?
        };
        let flags: u16 = (u16::from(assume_valid) << 15)
            | (u16::from(extended) << 14)
            | ((stage as u16) << 12)
            | name_len_field;

        let path_bytes = path_str.as_bytes();
        let mut buf = vec![];
        buf.extend(path_bytes);

        let entry = IndexEntry {
            ctime,
            ctime_nanos,
            mtime,
            mtime_nanos,
            dev,
            ino,
            mode,
            uid,
            gid,
            file_size,
            sha1,
            flags,
            path: buf,
        };
        Ok(entry)
    }

    fn write_index_to_file(&self) -> std::io::Result<()> {
        let mut buffer = Vec::new();
        let index_path = &RepoRust::get_root().base_path.join(BASE_DIR).join("INDEX");

        let header = self.header();
        let header_bytes = header.to_bytes();
        buffer.extend_from_slice(&header_bytes);
        for entry in self.entries.values() {
            buffer.extend_from_slice(&entry.to_bytes());
        }

        let mut hasher = Sha1::new();
        hasher.update(&buffer);
        let checksum = hasher.finalize();

        let mut file = std::fs::File::create(index_path)?;
        file.write_all(&buffer)?;
        file.write_all(&checksum)?;
        Ok(())
    }

    pub fn read_index() -> std::io::Result<Self> {
        let index_path = &RepoRust::get_root().base_path.join(BASE_DIR).join("INDEX");
        let file = std::fs::read(index_path)?;

        // Parse header
        let header = IndexHeader::header(&file[..12])?;
        let total_entries = u32::from_be_bytes(header.entries);

        let mut entries: BTreeMap<String, IndexEntry> = BTreeMap::new();
        let mut bytes_read = 12;
        for _ in 0..total_entries {
            let (entry, size) = IndexEntry::from_bytes(&file[bytes_read..])?;
            let path = String::from_utf8(entry.path.clone())
                .map_err(|_| std::io::Error::other("Invalid path when parsing IndexEntry"))?;
            entries.insert(path, entry);
            bytes_read += size;
        }
        Ok(Self { header, entries })
    }

    pub fn ls_index() -> std::io::Result<BTreeMap<String, IndexEntry>> {
        let index = Self::read_index()?;
        Ok(index.entries)
    }
}

impl fmt::Display for IndexEntry {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

impl Display for Index {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}
