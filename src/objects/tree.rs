use std::fmt::Display;

use clap::ArgMatches;
use hex::ToHex;

use crate::objects::{GitObject, Header, ObjectType};

pub struct TreeEntry {
    pub(crate) mode: String,
    pub(crate) object_type: ObjectType,
    pub(crate) name: String,
    pub(crate) hash: [u8; 20],
}

pub struct Tree {
    pub(crate) header: Header,
    pub(crate) hash: String,
    pub(crate) entries: Vec<TreeEntry>,
}

impl Tree {
    pub fn from(_file: Vec<u8>) -> std::io::Result<Self> {
        todo!()
    }

    fn serialize(&self) {
        todo!()
    }

    fn deserialize(&self) {
        todo!()
    }

    fn compress(&self) {
        todo!()
    }

    fn decompress(&self) {
        todo!()
    }
}

impl GitObject for Tree {
    const TYPE: ObjectType = ObjectType::Tree;
    type Output = Tree;

    fn header(&self) -> &Header {
        &self.header
    }

    fn encode_object(_args: &ArgMatches) -> std::io::Result<Tree> {
        todo!()
    }

    fn decode_object(_args: &ArgMatches) -> std::io::Result<Tree> {
        todo!()
    }
    fn write_object_to_file(&self, _file: Vec<u8>) -> std::io::Result<()> {
        todo!()
    }
}

impl Display for TreeEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} {} {}",
            &self.mode,
            &self.object_type,
            &self.hash.encode_hex::<String>(),
            &self.name
        )
    }
}

impl Display for Tree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, entry) in self.entries.iter().enumerate() {
            if i > 0 {
                writeln!(f)?;
            }
            write!(f, "{entry}")?;
        }
        Ok(())
    }
}
