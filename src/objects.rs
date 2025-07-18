#![allow(dead_code)]

use std::fmt::Display;

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
        Ok(Self {
            object: ObjectType::Tree,
            size,
        })
    }

    pub fn from_tree_entries(entries: usize) -> Self {
        Self {
            object: ObjectType::Tree,
            size: entries,
        }
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
