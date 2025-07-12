use clap::ArgMatches;
use std::{fs, io::Error, path::PathBuf, sync::Arc};
use thread_local::ThreadLocal;

use crate::objects::{GitObject, blob::Blob, tree::Tree};

pub const BASE_DIR: &str = ".git_rust";

// ThreadLocal for testing purposes
pub static REPO: ThreadLocal<Arc<RepoRust>> = ThreadLocal::new();

// Built internally, to hold information about the repo
// base_path - used finding the repo location when not in root
pub struct RepoRust {
    pub base_path: PathBuf,
}

#[allow(dead_code)]
impl RepoRust {
    // Used internaly. No connection to git init
    pub fn new_repo(path: &str) -> std::io::Result<()> {
        let repo = RepoRust {
            base_path: path.into(),
        };

        if REPO.get().is_some() {
            return Err(Error::other("Repo already initialized"));
        }
        REPO.get_or(|| Arc::new(repo));
        Ok(())
    }

    // TODO
    // Used to add an existing repo to RepoRust
    pub fn change_path(_path: &str) -> std::io::Result<()> {
        // Check if path is same as old
        todo!()
    }

    // TODO
    // Used to add an existing repo.
    // Will change the path in RepoRust and move in storage
    pub fn move_repo(_path: &str) -> std::io::Result<()> {
        // Check if path is same as old
        // Check if a repo already exists in that dir
        todo!()
    }

    // Used by git init when repo already initialized (when testing)
    pub fn get_root() -> std::io::Result<Arc<RepoRust>> {
        if let Some(repo) = REPO.get() {
            return Ok(repo.clone());
        }

        let dir = RepoRust::find_root()
            .unwrap_or_else(|_| std::env::current_dir().expect("Failed to read filesystem"));

        Ok(REPO
            .get_or(|| Arc::new(RepoRust { base_path: dir }))
            .clone())
    }

    // Used by git init to find repo
    // Will search starting in project root and upwards
    fn find_root() -> std::io::Result<PathBuf> {
        let mut dir =
            std::env::current_dir().map_err(|_| Error::other("Failed to read filesystem"))?;
        loop {
            if dir.join(BASE_DIR).is_dir() {
                return Ok(dir);
            }
            if !dir.pop() {
                return Err(std::io::Error::other("Could not find any repo folder"));
            }
        }
    }

    pub fn get_object_folder(root: PathBuf) -> std::io::Result<PathBuf> {
        Ok(root.join(BASE_DIR).join("objects"))
    }

    pub fn init() -> std::io::Result<()> {
        let root = RepoRust::get_root()?;
        let head = root.base_path.join(BASE_DIR).join("HEAD");
        if head.try_exists()? {
            return Err(Error::other("Git already initialized!"));
        } else {
            fs::create_dir(root.base_path.join(BASE_DIR))?;
            fs::create_dir(root.base_path.join(BASE_DIR).join("objects"))?;
            fs::create_dir(root.base_path.join(BASE_DIR).join("refs"))?;
            fs::write(head, "ref: refs/heads/master\n")?;
            println!("Initialized git directory!");
        }
        Ok(())
    }

    pub fn cat_file(args: &ArgMatches) -> std::io::Result<()> {
        let blob = Blob::decode_object(args)?;
        print!("{blob}");
        Ok(())
    }

    pub fn hash_object(args: &ArgMatches) -> std::io::Result<()> {
        let blob = Blob::encode_object(args)?;
        println!("{blob}");
        Ok(())
    }

    pub fn ls_tree(args: &ArgMatches) -> std::io::Result<()> {
        let tree = Tree::decode_object(args)?;
        println!("{tree}");
        Ok(())
    }
}
