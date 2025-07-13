#![allow(dead_code)]

use core::fmt;
use std::{fmt::Display, io::Error, os::unix::fs::MetadataExt, path::PathBuf};

use sha1::{Digest, Sha1};

use crate::git_rust::{BASE_DIR, RepoRust};

pub struct Index {
    header: IndexHeader,
    entries: Vec<IndexEntry>,
}

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
    pub object_id: [u8; 20],
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

    fn build_index(path: String) -> std::io::Result<Index> {
        let root = &RepoRust::get_root()?.base_path;
        if root.join(BASE_DIR).join("INDEX").exists() {
            return Err(Error::other("INDEX already exists"));
        }
        let mut entries: Vec<IndexEntry> = Vec::new();
        let path = PathBuf::from(path);
        if path.is_file() {
            entries.push(Index::index_file(path)?);
        } else if path.is_dir() {
            // entries = Index::index_dir(path)?;
        }
        // Build the index and return
        todo!()
    }

    fn sha1_entry(file: Vec<u8>) -> std::io::Result<[u8; 20]> {
        let mut hasher = Sha1::new();
        hasher.update(&file);
        let result = hasher.finalize();
        Ok(result.into())
    }

    fn index_dir(path: PathBuf) -> std::io::Result<Vec<IndexEntry>> {
        let mut _entries: Vec<IndexEntry> = Vec::new();
        for node in path.read_dir()? {
            let node = node?;
            match node.file_type()? {
                n if n.is_dir() => {
                    // recurse???
                }
                n if n.is_file() => {
                    // write entry
                    // return SHA-1
                }
                _ => continue,
            }
        }
        todo!()
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

        let file = std::fs::read(PathBuf::from(&path))?;
        let object_id = Index::sha1_entry(file)?;

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
            object_id,
            flags,
        };
        Ok(entry)
    }

    fn update_index() {}
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
