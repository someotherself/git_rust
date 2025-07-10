use std::fmt::Display;
use std::io::Read;

use clap::ArgMatches;
use flate2::bufread::ZlibDecoder;
use hex::ToHex;

use crate::{
    git_rust::RepoRust,
    objects::{GitObject, Header, ObjectType},
};

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
    pub fn from_bytes(_file: Vec<u8>) -> std::io::Result<Self> {
        todo!()
    }

    pub fn de_compress(content: Vec<u8>) -> std::io::Result<Vec<u8>> {
        let mut buffer = vec![0; 1024];
        let mut decompressed = ZlibDecoder::new(&content[..]);
        decompressed.read_exact(&mut buffer)?;
        Ok(buffer)
    }

    pub fn get_tree_entries(
        bytes_output: Vec<u8>,
        head: &Header,
    ) -> std::io::Result<Vec<TreeEntry>> {
        // head.head_length() = length of the head, in order to skip it
        // head.size = size of the content to parse starting after head.head_length()
        //
        // object-> mode+b' '+file name+b'\0'+hash [u8; 20]
        // object-> 100644+b' '+test.txt+b'\0'+63aa9936a393155f43c2b03d42d79b1c83290f41
        // Output-> 100644 blob 63aa9936a393155f43c2b03d42d79b1c83290f41 file.txt

        let mut entries: Vec<TreeEntry> = vec![];
        let mut i = head.head_length() + 1;
        while i < head.size {
            let mut start = i;
            while bytes_output[i] != b' ' {
                i += 1
            }
            let mode = String::from_utf8(bytes_output[start..i].to_vec()).unwrap();
            let objecttype: ObjectType;
            match mode.as_str() {
                "100644" => objecttype = ObjectType::Blob,
                "040000" => objecttype = ObjectType::Tree,
                _ => {
                    panic!("Invalid object type.")
                }
            }
            start = i + 1;
            while bytes_output[i] != b'\0' {
                i += 1
            }
            let name = String::from_utf8(bytes_output[start..i].to_vec()).unwrap();
            start = i + 1;
            let mut hash = [0_u8; 20];
            hash.copy_from_slice(&bytes_output[start..start + 20]);
            i += 21;
            let tree = TreeEntry {
                mode,
                object_type: objecttype,
                name,
                hash,
            };
            entries.push(tree);
        }
        Ok(entries)
    }
}

impl GitObject for Tree {
    const TYPE: ObjectType = ObjectType::Tree;
    type Output = Tree;

    fn header(&self) -> &Header {
        &self.header
    }

    // write-tree
    fn encode_object(_args: &ArgMatches) -> std::io::Result<Tree> {
        todo!()
    }

    // ls-tree
    fn decode_object(args: &ArgMatches) -> std::io::Result<Tree> {
        let root_path = RepoRust::get_object_folder(RepoRust::get_root()?.base_path.clone())?;

        let hash = args
            .get_one::<String>("hash")
            .expect("Object is required.")
            .to_owned();
        let (folder_name, file_name) = hash.split_at(2);
        let file_path = root_path.join(folder_name).join(file_name);
        let file_content = std::fs::read(file_path)?;

        let bytes_output = Tree::de_compress(file_content)?;
        let header = Header::from_binary(bytes_output.clone())?;

        let entries = Tree::get_tree_entries(bytes_output, &header)?;

        let tree = Tree {
            header,
            hash,
            entries,
        };
        Ok(tree)
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
