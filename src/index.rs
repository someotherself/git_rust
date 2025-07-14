#![allow(dead_code)]

use core::fmt;
use std::{collections::BTreeMap, fmt::Display, os::unix::fs::MetadataExt, path::PathBuf};

use sha1::{Digest, Sha1};

use crate::{
    git_rust::{BASE_DIR, RepoRust},
    objects::{GitObject, blob::Blob},
};

#[derive(Default)]
pub struct Index {
    header: IndexHeader,
    // (path, IndexEntry)
    entries: BTreeMap<String, IndexEntry>,
}

#[derive(Default)]
pub struct IndexHeader {
    pub sign: [u8; 4],
    pub version: u32,
    pub entries: u32,
}

impl IndexHeader {
    fn header(entries: u32) -> Self {
        let sign = [b'D', b'R', b'R', b'C'];
        Self {
            sign,
            version: 2_u32,
            entries,
        }
    }
}

#[repr(C)]
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
}

// TODO
impl Display for Index {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

#[allow(dead_code)]
impl Index {
    fn header(&self) -> &IndexHeader {
        &self.header
    }

    // TODO have a separate function that update index file and presists to disk
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
    // 3. Paths now merge. Check if blob exists in index
    //      A. Path exists and SHA1 is different         -> Update index. Persist change
    //      B. Path does not exist in index              -> Add to index. Persist change
    //      C. Path exists and SHA1 is same              -> Move on
    fn update_index(path: String) -> std::io::Result<Index> {
        let root = &RepoRust::get_root()?.base_path;
        let mut index = Index::default();
        if root.join(BASE_DIR).join("INDEX").exists() {
            // Read the index to the struct TODO
        }
        let path = PathBuf::from(path);
        let mut stack = vec![path];

        while let Some(current_path) = stack.pop() {
            if current_path.is_file() {
                // 1. Get entry - index the file
                let entry = Index::index_file(current_path.clone())?;
                let key: String = current_path.to_string_lossy().into();

                // 2. Check if blob already exists on disk
                if !Blob::blob_exists(String::from_utf8(entry.sha1.to_vec()).unwrap())? {
                    // If no - Create the file
                    let file = std::fs::read(current_path)?;
                    let blob = Blob::blob_with_sha1(file.clone())?;
                    blob.write_object_to_file(file)?;
                } // If yes, move on

                // 3. Check if blob exists in index
                match index.entries.get(&key) {
                    // A.Path does not exist in index -> Add to index.
                    None => {
                        index.entries.insert(key, entry);
                    }
                    // B. Path exists and SHA1 is different -> Update index. Persist change
                    Some(existing_entry) if existing_entry.sha1 != entry.sha1 => {
                        // Persist to file
                        // TODO
                    }
                    // C. Path exists and SHA1 is same
                    _ => continue,
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
        todo!()
    }

    fn sha1_entry(file: Vec<u8>) -> std::io::Result<[u8; 20]> {
        let mut hasher = Sha1::new();
        hasher.update(&file);
        let result = hasher.finalize();
        Ok(result.into())
    }

    #[allow(unused_variables)]
    pub fn index_file(path: PathBuf) -> std::io::Result<IndexEntry> {
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

        let file = std::fs::read(&path)?;
        let sha1 = Index::sha1_entry(file)?;

        let path_str = path.to_string_lossy();
        let name_len = path_str.len();
        let assume_valid = false;
        let extended = false;
        let stage = 0b00;
        let name_len_field = if name_len >= 0xFFF {
            0xFFF
        } else {
            name_len as u16
        };
        let flags: u16 = ((assume_valid as u16) << 15)
            | ((extended as u16) << 14)
            | ((stage as u16) << 12)
            | name_len_field;

        let path_bytes = path_str.as_bytes();
        let padding = (8 - ((62 + path_bytes.len()) % 8)) % 8;

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
        };
        Ok(entry)
    }
}

impl fmt::Display for IndexEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ctime: {}.{}  mtime: {}.{}  dev: {}  ino: {}  mode: {:o}  uid: {}  gid: {}  size: {}  flags: {}",
            self.ctime,
            self.ctime_nanos,
            self.mtime,
            self.mtime_nanos,
            self.dev,
            self.ino,
            self.mode,
            self.uid,
            self.gid,
            self.file_size,
            self.flags,
        )
    }
}
