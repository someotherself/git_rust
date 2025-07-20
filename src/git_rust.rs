use clap::ArgMatches;
use std::{
    fs,
    io::{Error, Write},
    path::{Path, PathBuf},
    sync::Arc,
};
use thread_local::ThreadLocal;

use crate::{
    index::Index,
    objects::{blob::Blob, tree::Tree},
};

pub const BASE_DIR: &str = ".git_rust";

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
        let repo = Self {
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
    pub fn get_root() -> Arc<Self> {
        if let Some(repo) = REPO.get() {
            return repo.clone();
        }

        let dir = Self::find_root()
            .unwrap_or_else(|_| std::env::current_dir().expect("Failed to read filesystem"));

        REPO.get_or(|| Arc::new(Self { base_path: dir })).clone()
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

    pub fn get_object_folder(root: &Path) -> PathBuf {
        root.join(BASE_DIR).join("objects")
    }

    pub fn init() -> std::io::Result<()> {
        let root = Self::get_root();
        let head = root.base_path.join(BASE_DIR).join("HEAD");
        if head.try_exists()? {
            return Err(Error::other("Git already initialized!"));
        }
        fs::create_dir(root.base_path.join(BASE_DIR))?;
        fs::create_dir(root.base_path.join(BASE_DIR).join("objects"))?;
        fs::create_dir(root.base_path.join(BASE_DIR).join("refs"))?;
        fs::write(head, "ref: refs/heads/master\n")?;
        println!("Initialized git directory!");
        Ok(())
    }

    pub fn cat_file(args: &ArgMatches) -> std::io::Result<()> {
        let _sub_arg = args.get_flag("pretty");
        let hash = args.get_one::<String>("hash").expect("Hash is required.");
        // Contents without the header
        // TODO Get the blob and display contents based on object type
        // let blob = Blob::from_hash(hash.clone())?;
        let contents = Blob::decode_object(hash)?;
        std::io::stdout().write_all(&contents)?;
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

    pub fn add(args: &ArgMatches) -> std::io::Result<()> {
        let path = args
            .get_one::<String>("path")
            .expect("File is required.")
            .to_owned();
        Index::build_index(path)?;
        Ok(())
    }

    pub fn ls_files(_args: &ArgMatches) -> std::io::Result<()> {
        let entries = Index::ls_index()?;
        for (path, _entry) in entries {
            println!("{path}");
        }
        Ok(())
    }

    pub fn write_tree(_args: &ArgMatches) -> std::io::Result<()> {
        Tree::encode_object()?;
        Ok(())
    }
}
