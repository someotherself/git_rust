use std::collections::{BTreeMap, HashMap};
use std::fmt::Display;
use std::io::Read;
use std::path::{Path, PathBuf};

use clap::ArgMatches;
use flate2::bufread::ZlibDecoder;
use hex::ToHex;
use sha1::{Digest, Sha1};

use crate::index::IndexEntry;
use crate::{
    git_rust::RepoRust,
    objects::{Header, ObjectType},
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
    pub fn header(&self) -> &Header {
        &self.header
    }

    // write-tree TODO
    pub fn encode_object(args: &ArgMatches) -> std::io::Result<Self> {
        // TODO: Check if blob already exists. Add test for it.
        // let sub_arg = args.get_flag("___");
        let _object = args
            .get_one::<String>("file")
            .expect("File is required.")
            .to_owned();
        todo!()
    }

    // ls-tree
    pub fn decode_object(args: &ArgMatches) -> std::io::Result<Self> {
        let root_path = RepoRust::get_object_folder(&RepoRust::get_root().base_path);

        let hash = args
            .get_one::<String>("hash")
            .expect("Object is required.")
            .to_owned();
        let (folder_name, file_name) = hash.split_at(2);
        let file_path = root_path.join(folder_name).join(file_name);
        let file_content = std::fs::read(file_path)?;

        let bytes_output = Self::de_compress(&file_content)?;
        let header = Header::from_binary(&bytes_output)?;

        let entries = Self::get_tree_entries(&bytes_output, &header);

        let tree = Self {
            header,
            hash,
            entries,
        };
        Ok(tree)
    }

    fn write_object_to_file(&self, _file: Vec<u8>) -> std::io::Result<()> {
        todo!()
    }

    // Will create a Tree struct given a Vec of TreeEntries
    pub fn from_entries(entries: Vec<TreeEntry>) -> std::io::Result<Self> {
        let header = Header::from_tree_entries(entries.len());
        let hash = Self::sha1_tree(&entries);
        let hash = str::from_utf8(&hash).unwrap().to_string();
        Ok(Self {
            header,
            hash,
            entries,
        })
    }

    // TODO
    // Will prepare the hash for a tree from a Vec of TreeEntries
    // Used before writing to disk
    pub fn sha1_tree(entries: &Vec<TreeEntry>) -> [u8; 20] {
        let mut hasher = Sha1::new();
        let mut content = Vec::new();
        for tree_entry in entries {
            let mode_str = match tree_entry.object_type {
                ObjectType::Blob => "100644",
                ObjectType::Tree => "40000",
                _ => continue,
            };
            content.extend_from_slice(mode_str.as_bytes());
            content.push(b' ');
            content.extend_from_slice(tree_entry.name.as_bytes());
            content.push(b'\0');
            content.extend_from_slice(&tree_entry.hash);
        }
        let header = format!("tree {}\0", content.len());
        hasher.update(header);
        hasher.update(content);
        hasher.finalize().into()
    }

    // Decompresses the contents of a tree to be read/displayed
    pub fn de_compress(content: &[u8]) -> std::io::Result<Vec<u8>> {
        let mut buffer = vec![0; 1024];
        let mut decompressed = ZlibDecoder::new(content);
        decompressed.read_exact(&mut buffer)?;
        Ok(buffer)
    }

    // Parses the contents of a tree objects into a Vec of TreeEntries
    pub fn get_tree_entries(bytes_output: &[u8], head: &Header) -> Vec<TreeEntry> {
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
                i += 1;
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
                i += 1;
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
        entries
    }

    // Flatten the list of paths:
    // HashMap<"path_without_file", "file">
    // Go through each entry and recurse the path
    // Create trees and sha1 each
    // write-tree command
    // Takes the entries (from the index), and prepares the Tree objects
    pub fn write_trees_from_index(entries: BTreeMap<String, IndexEntry>) -> std::io::Result<()> {
        let mut flat_entries: BTreeMap<PathBuf, Vec<(PathBuf, IndexEntry)>> = BTreeMap::new();
        for (path, entry) in entries {
            let path = PathBuf::from(path);
            // Paths such as "/" or "." should not be possible
            let file_name = path.file_name().unwrap().to_owned();
            let mut root = PathBuf::from("root");
            root.push(path);
            let parent_path = root.parent().filter(|p| *p != Path::new("")).unwrap();
            flat_entries
                .entry(PathBuf::from(parent_path))
                .or_default()
                .push((PathBuf::from(file_name), entry));
        }
        Self::build_trees(flat_entries)?;
        Ok(())
    }

    fn build_trees(entries: BTreeMap<PathBuf, Vec<(PathBuf, IndexEntry)>>) -> std::io::Result<()> {
        // Create a hash table of all the folders and trees
        // Create and update trees and write to file at the end
        let mut tree_list: HashMap<PathBuf, Tree> = HashMap::new();
        for (path, children) in entries {
            let mut tree_entries: Vec<TreeEntry> = Vec::new();
            for (child, entry) in children {
                let blob_entry = TreeEntry {
                    mode: entry.mode.to_string(),
                    object_type: ObjectType::Blob,
                    name: child.to_str().unwrap().to_string(),
                    hash: entry.sha1,
                };
                tree_entries.push(blob_entry);
            }
            let path_comp = path
                .components()
                .map(|e| PathBuf::from(e.as_os_str()))
                .collect::<Vec<PathBuf>>();
            match tree_list.get(path_comp.last().unwrap()) {
                Some(_) => {
                    // Update the tree
                    // let old_tree = tree_list.get(path_comp.last().unwrap()).unwrap();
                    // let old_entries = &old_tree.entries;
                }
                None => {
                    let tree = Self::from_entries(tree_entries)?;
                    tree_list.insert(path_comp.last().unwrap().clone(), tree);
                }
            }
            // Check if it folder exists in tree_list
            // Add children entries into the vec of tree entries
            //
        }
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
