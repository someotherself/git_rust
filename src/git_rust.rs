use clap::ArgMatches;
use std::{
    fs,
    io::Error,
    path::{Path, PathBuf},
    sync::{Arc, OnceLock},
};

use crate::objects::{GitObject, blob::Blob, tree::Tree};

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
        // Check if path is same as old
        todo!()
    }

    pub fn move_repo(path: &str) -> std::io::Result<()> {
        // Check if path is same as old
        // Check if a repo already exists
        todo!()
    }

    pub fn get_root() -> std::io::Result<Arc<Repo_Rust>> {
        let dir = Repo_Rust::find_root()?;
        let repo = REPO
            .get_or_init(|| Arc::new(Repo_Rust { base_path: dir }))
            .clone();
        Ok(repo)
    }

    fn find_root() -> std::io::Result<PathBuf> {
        let mut dir = std::env::current_dir()
            .map_err(|_| Error::new(std::io::ErrorKind::Other, "Failed to read filesystem"))?;
        loop {
            if dir.join(BASE_DIR).is_dir() {
                return Ok(dir);
            }
            if !dir.pop() {
                return Err(Error::new(std::io::ErrorKind::NotFound, "No repo found!"));
            }
        }
    }

    pub fn get_object_folder(root: PathBuf) -> std::io::Result<PathBuf> {
        Ok(root.join(BASE_DIR).join("objects"))
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
        let blob = Blob::decode_object(args)?;
        print!("{}", blob);
        Ok(())
    }

    pub fn hash_object(args: &ArgMatches) -> std::io::Result<()> {
        let blob = Blob::encode_object(args)?;
        println!("{}", blob);
        Ok(())
    }

    // FIX - 1
    pub fn ls_tree(args: &ArgMatches) -> std::io::Result<()> {
        let tree = Tree::decode_object(args)?;
        println!("{}", tree);
        Ok(())
    }
}
