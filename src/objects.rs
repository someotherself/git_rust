#![allow(dead_code)]

use std::{fmt::Display, path::PathBuf};

use clap::ArgMatches;

pub mod blob;
pub mod commit;
pub mod tree;

#[cfg(test)]
mod test;
pub enum ObjectType {
    Blob,
    Tree,
    Commit,
}

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

    pub fn from_binary(file: Vec<u8>) -> std::io::Result<Header> {
        let header = file
            .iter()
            .copied()
            .take_while(|&b| b != b'\0')
            .collect::<Vec<u8>>();
        let string = String::from_utf8(header).unwrap();
        let vec: Vec<&str> = string.split(" ").collect();
        let size: usize = vec[1].parse().unwrap();
        Ok(Header {
            object: ObjectType::Tree,
            size,
        })
    }
}

impl ObjectType {
    fn get_mode(_path: PathBuf) -> std::io::Result<()> {
        todo!()
    }
}

pub trait GitObject: Display {
    const TYPE: ObjectType;

    type Output: GitObject;

    fn header(&self) -> &Header;

    fn write_object_to_file(&self, file: Vec<u8>) -> std::io::Result<()>;

    fn encode_object(args: &ArgMatches) -> std::io::Result<Self::Output>;

    fn decode_object(args: &ArgMatches) -> std::io::Result<Self::Output>;
}

impl Display for ObjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ObjectType::Blob => f.write_str("blob"),
            ObjectType::Tree => f.write_str("tree"),
            ObjectType::Commit => f.write_str("commit"),
        }
    }
}
