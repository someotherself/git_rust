use std::collections::{BTreeMap, HashMap};
use std::fmt::Display;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use clap::{ArgMatches, crate_name};
use flate2::bufread::ZlibDecoder;
use flate2::{Compress, Compression, write::ZlibEncoder};
use hex::ToHex;
use sha1::{Digest, Sha1};

use crate::index::{Index, IndexEntry};
use crate::{
    git_rust::RepoRust,
    objects::{Header, ObjectType},
};

pub struct TreeEntry {
    pub mode: String,
    pub object_type: ObjectType,
    pub name: String,
    pub hash: [u8; 20],
}

pub struct Tree {
    pub header: Header,
    pub hash: [u8; 20],
    pub entries: Vec<TreeEntry>,
}

impl Tree {
    pub fn header(&self) -> &Header {
        &self.header
    }

    // write-tree TODO
    pub fn encode_object() -> std::io::Result<()> {
        let index = Index::read_index()?;
        let entries_by_folder = Self::group_entries_for_tree_build(index.entries);
        let _trees = Self::build_trees(entries_by_folder);
        // Self::write_object_to_file(trees)?; // TODO
        Ok(())
    }

    // ls-tree
    pub fn decode_object(args: &ArgMatches) -> std::io::Result<Self> {
        let root_path = RepoRust::get_object_folder(&RepoRust::get_root().base_path);

        let hash_str = args
            .get_one::<String>("hash")
            .expect("Object is required.")
            .to_owned();

        let hash_vec = hex::decode(&hash_str).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Invalid SHA1: {e}"),
            )
        })?;

        let hash: [u8; 20] = hash_vec.try_into().map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "SHA1 must be 20 bytes")
        })?;

        let (folder_name, file_name) = hash_str.split_at(2);
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

    fn write_object_to_file(trees: Vec<Self>) -> std::io::Result<()> {
        let objects_path = RepoRust::get_object_folder(&RepoRust::get_root().base_path);
        for tree in trees {
            let mut content: Vec<u8> = Vec::new();
            let hex_hash = hex::encode(tree.hash);
            let (folder_name, file_name) = hex_hash.split_at(2);

            let folder_path = objects_path.join(folder_name);
            let file_path = folder_path.join(file_name);

            if !folder_path.exists() {
                std::fs::create_dir(folder_path)?;
            }
            let new_tree = std::fs::File::create(file_path)?;

            let mut enc =
                ZlibEncoder::new_with_compress(new_tree, Compress::new(Compression::best(), true));

            let header = format!("tree {}\0", tree.header.size);

            content.extend_from_slice(header.as_bytes());

            for entry in tree.entries {
                let mode = entry.mode;
                let tree_name = entry.name;
                content.extend_from_slice(mode.as_bytes());
                content.extend_from_slice(b" ");
                content.extend_from_slice(tree_name.as_bytes());
                content.extend_from_slice(b"\0");
                content.extend_from_slice(&entry.hash);
            }
            enc.write_all(&content)?;
            enc.finish()?;
        }
        Ok(())
    }

    // Will create and sort Tree struct given a Vec of TreeEntries
    pub fn from_entries(mut entries: Vec<TreeEntry>) -> Self {
        entries.sort_by(|a, b| {
            let name_cmp = a.name.as_bytes().cmp(b.name.as_bytes());
            if name_cmp == std::cmp::Ordering::Equal {
                match (&a.object_type, &b.object_type) {
                    (ObjectType::Blob, ObjectType::Tree) => std::cmp::Ordering::Less,
                    (ObjectType::Tree, ObjectType::Blob) => std::cmp::Ordering::Greater,
                    _ => std::cmp::Ordering::Equal,
                }
            } else {
                name_cmp
            }
        });
        let header = Header::from_tree_entries(entries.len());
        let hash = Self::sha1_tree(&entries);
        Self {
            header,
            hash,
            entries,
        }
    }

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
                "040000" | "40000" => objecttype = ObjectType::Tree,
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

    // write-tree command
    // Takes the entries (from the index), and prepares the Tree objects
    pub fn group_entries_for_tree_build(
        entries: BTreeMap<String, IndexEntry>,
    ) -> HashMap<PathBuf, Vec<(PathBuf, IndexEntry)>> {
        // root must have a tree as well. Add it in front of the path
        let mut root = RepoRust::get_root().root_path.clone();
        // Flatten the list of paths and combine all the files in each folder
        // HashMap<"path_without_file", "file">
        // Go through each entry and recurse the path
        // Create trees and sha1 each
        let mut entries_by_folder: HashMap<PathBuf, Vec<(PathBuf, IndexEntry)>> = HashMap::new();
        // Example index entries: "src/objects/blob.rs", "src/objects/tree.rs"
        // entries_by_folder: HashMap<root/src/objects/, Vec<blob.rs, tree.rs(as index entries)>>
        for (path, entry) in entries {
            let path = PathBuf::from(path);
            // Paths such as "/" or "." should not be possible
            let file_name = path.file_name().unwrap().to_owned();

            // Add "root" to the path to keep track of it
            root.push(path);
            let parent_path = root.parent().filter(|p| *p != Path::new("")).unwrap();
            entries_by_folder
                .entry(PathBuf::from(parent_path))
                .or_default()
                .push((PathBuf::from(file_name), entry));
        }
        entries_by_folder
    }

    pub fn build_trees(
        entries_by_folder: HashMap<PathBuf, Vec<(PathBuf, IndexEntry)>>,
    ) -> Vec<Self> {
        // Create a hash table of all the folders and trees
        // Create and update trees and write to file at the end
        let mut tree_list: HashMap<PathBuf, Self> = HashMap::new();
        for (path, children) in &entries_by_folder {
            let mut tree_entries: Vec<TreeEntry> = Vec::new();
            // Create the blob for each file
            for (child, entry) in children {
                let blob_entry = TreeEntry {
                    mode: entry.mode.to_string(),
                    object_type: ObjectType::Blob,
                    name: child.to_str().unwrap().to_string(),
                    hash: entry.sha1,
                };
                tree_entries.push(blob_entry);
            }
            // Create new tree and add it
            // Tree only contains blobs and belongs to the last folder down the path
            let tree = Self::from_entries(tree_entries);
            tree_list.insert(path.clone(), tree);
        }
        // Finished adding the files.
        // Go through folder, bottom to top and create trees
        // Example: root/dir1/dir2/dir3
        // The tree created so far represents dir3.
        // Pop that and create new trees for each folder up the path
        for (mut path, _) in entries_by_folder {
            loop {
                if let Some(tree) = tree_list.get(&path) {
                    if !path.pop() || path == PathBuf::from("/") || path == PathBuf::from("") {
                        break;
                    }
                    let tree_name = path
                        .file_name()
                        .unwrap_or(path.as_os_str())
                        .to_string_lossy()
                        .into_owned();
                    let tree_entry = TreeEntry {
                        mode: "40000".to_string(),
                        object_type: ObjectType::Tree,
                        name: tree_name,
                        hash: tree.hash,
                    };
                    let mut new_tree_vec = vec![tree_entry];
                    if let Some(existing_tree) = tree_list.remove(&path) {
                        // A tree was already made for this dir. Remove and add those entries
                        new_tree_vec.extend(existing_tree.entries);
                    }
                    let new_tree = Self::from_entries(new_tree_vec);
                    tree_list.insert(path.clone(), new_tree);
                }
            }
        }
        tree_list.into_values().collect::<Vec<Self>>()
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
