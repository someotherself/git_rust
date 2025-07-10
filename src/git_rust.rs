use clap::ArgMatches;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::{Arc, OnceLock},
    vec,
};

use crate::{
    objects::{
        GitObject, Header, ObjectType,
        blob::Blob,
        tree::{Tree, TreeEntry},
    },
    utils::{de_compress, get_root},
};

pub const BASE_DIR: &str = ".git_rust";
pub static REPO_ROOT: OnceLock<PathBuf> = OnceLock::new();

pub static REPO: OnceLock<Arc<Repo_Rust>> = OnceLock::new();

pub struct Repo_Rust {
    pub base_path: PathBuf,
}

impl Repo_Rust {
    pub fn new_repo(path: &str) -> std::io::Result<()> {
        todo!()
    }

    pub fn change_path(path: &str) -> std::io::Result<()> {
        todo!()
    }

    pub fn move_repo(path: &str) -> std::io::Result<()> {
        todo!()
    }

    pub fn init() -> std::io::Result<()> {
        let head = Path::new(".git/HEAD");
        if head.try_exists()? {
            println!("Git already initialized!");
        } else {
            let base_dir = PathBuf::from(BASE_DIR);
            fs::create_dir(&base_dir)?;
            fs::create_dir(&base_dir.join("/objects"))?;
            fs::create_dir(&base_dir.join("/refs"))?;
            fs::write(head, "ref: refs/heads/master\n").unwrap();
            println!("Initialized git directory!");
        }
        Ok(())
    }

    pub fn cat_file(args: &ArgMatches) -> std::io::Result<()> {
        // if let Some(path) = find_file_in_repo() {
        //     println!("{:?}", path);
        // } else {
        //     panic!("Repo does not exist");
        // }
        let repo = get_root()?;
        dbg!(&repo.base_path);
        let blob = Blob::decode_object(args)?;
        print!("{}", blob);
        Ok(())
    }

    // FIX - 1 - NEXT TODO
    pub fn hash_object(_args: &ArgMatches) -> std::io::Result<()> {
        // let sub_arg = args.get_flag("pretty");
        // let blob = GitObject::encode_object(args)?;
        // if sub_arg {
        //     // blob.write_object(blob)?;
        //     println!("{}", blob);
        // } else {
        //     println!("{}", blob);
        // }
        Ok(())
    }

    // FIX - 2
    pub fn ls_tree(args: &ArgMatches) -> std::io::Result<()> {
        let hash = args
            .get_one::<String>("hash")
            .expect("Object is required.")
            .to_owned();
        let (folder_name, file_name) = hash.split_at(2);
        let file_path = PathBuf::from(format!(".git/objects/{}/{}", folder_name, file_name));
        let file_content = fs::read(file_path)?;

        let bytes_output = de_compress(file_content)?;
        let header = Header::from_binary(bytes_output.clone())?;
        let pos = bytes_output.iter().copied().position(|b| b == 0).unwrap();

        let mut entries: Vec<TreeEntry> = vec![];
        let mut i = pos + 1;
        while i < header.size {
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
        // println!("{:?}", entries);
        let tree = Tree {
            header,
            hash,
            entries,
        };
        println!("{}", tree);
        Ok(())
    }
}
