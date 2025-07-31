#![allow(dead_code)]

use std::{
    fmt::Display,
    io::{Read, Write},
    path::PathBuf,
};

use crate::{
    git_rust::RepoRust,
    objects::{blob::Blob, commit::Commit, tree::Tree},
};
use tracing::instrument;

pub mod blob;
pub mod commit;
pub mod tree;

#[cfg(test)]
mod test;

#[derive(Debug)]
pub enum ObjectType {
    Blob,
    Tree,
    Commit,
}

#[derive(Debug)]
pub struct Header {
    pub(crate) object: ObjectType,
    pub(crate) size: usize,
}

impl Header {
    pub fn head_length(&self) -> usize {
        let object = format!("{}", self.object);
        let content_size = format!("{}", self.size);
        object.len() + content_size.len() + 1
    }

    pub fn from_binary(file: &[u8]) -> std::io::Result<Self> {
        let header = file
            .iter()
            .copied()
            .take_while(|&b| b != b'\0')
            .collect::<Vec<u8>>();
        let string = String::from_utf8(header).unwrap();
        let vec: Vec<&str> = string.split(' ').collect();
        let size: usize = vec
            .get(1)
            .ok_or_else(|| std::io::Error::other("Missing size field"))?
            .parse()
            .map_err(|_| std::io::Error::other("Missing size field"))?;

        let object_type = match vec[0] {
            "blob" => ObjectType::Blob,
            "tree" => ObjectType::Tree,
            "commit" => ObjectType::Commit,
            _ => return Err(std::io::Error::other("Invalid object type")),
        };
        Ok(Self {
            object: object_type,
            size,
        })
    }

    // TODO: size must be bytes, not number of entries
    pub fn from_tree_entries(entries: usize) -> Self {
        Self {
            object: ObjectType::Tree,
            size: entries,
        }
    }
}

#[instrument]
pub fn cat_file(hash: &str, pretty: bool) -> std::io::Result<Vec<u8>> {
    let root_path = RepoRust::get_object_folder(&RepoRust::get_root().absolute_path);
    let (folder_name, file_name) = hash.split_at(2);
    let file_path = root_path.join(folder_name).join(file_name);
    let file = std::fs::read(file_path)?;
    let de_compressed_file = de_compress(&file)?;

    let header = Header::from_binary(&de_compressed_file)?;
    let mut content: Vec<u8> = Vec::new();
    match header.object {
        // -p not implemented for all
        ObjectType::Blob => {
            content = Blob::decode_object(&de_compressed_file)?;
            std::io::stdout().write_all(&content)?;
        }
        ObjectType::Tree => {
            if pretty {
                let tree = Tree::decode_object(hash)?;
                println!("{tree}");
            } else {
                content = Tree::de_compress(&file)?;
                std::io::stdout().write_all(&content)?;
            }
        }
        ObjectType::Commit => {
            let commit = Commit::decode(hash)?;
            print!("{commit}");
        }
    }
    Ok(content)
}

pub fn de_compress(content: &[u8]) -> std::io::Result<Vec<u8>> {
    let mut decompressed = Vec::new();
    let cursor = std::io::Cursor::new(content);
    let mut decoder = flate2::bufread::ZlibDecoder::new(cursor);
    decoder.read_to_end(&mut decompressed)?;
    Ok(decompressed)
}

pub fn get_object_path(hash: &str) -> Option<PathBuf> {
    let root_path = RepoRust::get_object_folder(&RepoRust::get_root().absolute_path);
    let (folder_name, file_name) = hash.split_at(2);
    let file_path = root_path.join(folder_name).join(file_name);
    if file_path.exists() {
        Some(file_path)
    } else {
        None
    }
}

impl Display for ObjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> core::fmt::Result {
        match *self {
            Self::Blob => f.write_str("blob"),
            Self::Tree => f.write_str("tree"),
            Self::Commit => f.write_str("commit"),
        }
    }
}
