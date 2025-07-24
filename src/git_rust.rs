use clap::ArgMatches;
use std::{
    fs,
    io::Error,
    path::{Component, Path, PathBuf},
    sync::{Arc, Mutex},
};
use thread_local::ThreadLocal;

use crate::{
    index::Index,
    objects::{
        self,
        blob::Blob,
        commit::{Commit, CommitSummary},
        tree::Tree,
    },
};

pub const BASE_DIR: &str = ".git_rust";

pub static REPO: ThreadLocal<Mutex<Option<Arc<RepoRust>>>> = ThreadLocal::new();

// Built internally, to hold information about the repo
// Used finding the repo location when not in root
// root_path - relative path
#[allow(dead_code)]
pub struct RepoRust {
    pub absolute_path: PathBuf,
    pub root_path: PathBuf,
}

#[allow(dead_code)]
impl RepoRust {
    // Used internaly. No connection to git init
    pub fn new_repo(path: &str) -> std::io::Result<()> {
        let path_buf = PathBuf::from(path);
        let root = path_buf
            .file_name()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid path"))?
            .into();
        let repo = Arc::new(RepoRust {
            absolute_path: path_buf,
            root_path: root,
        });

        let cell = REPO.get_or(|| Mutex::new(None));
        let mut guard = cell.lock().unwrap();
        *guard = Some(repo);
        Ok(())
    }

    pub fn clear_repo() {
        if let Some(mutex) = REPO.get() {
            let mut guard = mutex.lock().unwrap();
            guard.take();
        }
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
        let mutex = REPO.get_or(|| Mutex::new(None));
        let mut guard = mutex.lock().unwrap();

        if let Some(repo) = &*guard {
            return repo.clone();
        }

        let dir = Self::find_root()
            .unwrap_or_else(|_| std::env::current_dir().expect("Failed to read filesystem"));
        let root = PathBuf::from(dir.file_name().unwrap());
        let repo = Arc::new(Self {
            absolute_path: dir,
            root_path: root,
        });

        *guard = Some(Arc::clone(&repo));
        repo
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

    // Should not allow paths with // or ..
    // Ensure paths are correctly parsed and inside root
    // Correctly format the paths for the index
    pub fn check_paths(path: String) -> std::io::Result<()> {
        let path = Path::new(&path);
        for comp in path.components() {
            match comp {
                Component::ParentDir => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "`..` not allowed in paths",
                    ));
                }
                Component::RootDir | Component::Prefix(_) => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "Absolute paths or prefixes not allowed",
                    ));
                }
                _ => {}
            }
        }
        Ok(())
    }

    pub fn init() -> std::io::Result<()> {
        let root = Self::get_root();
        let head = root.absolute_path.join(BASE_DIR).join("HEAD");
        if head.try_exists()? {
            return Err(Error::other("Git already initialized!"));
        }
        fs::create_dir(root.absolute_path.join(BASE_DIR))?;
        fs::create_dir(root.absolute_path.join(BASE_DIR).join("objects"))?;
        fs::create_dir(root.absolute_path.join(BASE_DIR).join("refs"))?;
        fs::write(head, "ref: refs/heads/master\n")?;
        println!("Initialized git directory!");
        Ok(())
    }

    pub fn cat_file(args: &ArgMatches) -> std::io::Result<Vec<u8>> {
        let sub_arg = args.get_flag("pretty");
        let hash = args.get_one::<String>("hash").unwrap();
        objects::cat_file(hash, sub_arg)
    }

    pub fn hash_object(args: &ArgMatches) -> std::io::Result<()> {
        let blob = Blob::encode_object(args)?;
        println!("{blob}");
        Ok(())
    }

    pub fn ls_tree(args: &ArgMatches) -> std::io::Result<()> {
        let hash_str = args
            .get_one::<String>("hash")
            .expect("Object is required.")
            .to_owned();
        let tree = Tree::decode_object(hash_str)?;
        println!("{tree}");
        Ok(())
    }

    pub fn add(args: &ArgMatches) -> std::io::Result<()> {
        let path = args.get_one::<String>("path").unwrap().to_owned();
        Self::check_paths(path.clone())?;
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
        let (trees, root_hash) = Tree::encode_object()?;
        Tree::write_object_to_file(trees)?;
        let root_hash = hex::encode(root_hash);
        println!("{root_hash}");
        Ok(())
    }

    pub fn commit_tree(args: &ArgMatches) -> std::io::Result<()> {
        let hash = args.get_one::<String>("hash").unwrap().to_owned();
        let commit = args
            .get_many::<String>("commit")
            .unwrap_or_default()
            .cloned()
            .collect::<Vec<_>>();
        let message = args
            .get_one::<String>("message")
            .unwrap_or(&"".into())
            .to_owned();
        let commit = Commit::encode(hash, commit.clone(), &message)?;
        commit.write_commit_to_file()?;
        Ok(())
    }

    // TODO Crate refs/heads/master if initial commit
    pub fn commit(args: &ArgMatches) -> std::io::Result<()> {
        // TODO Add the -a flag
        let message = args
            .get_one::<String>("message")
            .unwrap_or(&"".into())
            .to_owned();

        // Build the current index. Get trees and the hash for the root tree.
        let (trees, new_tree_hash_bytes) = Tree::encode_object()?;
        let new_tree_hash = hex::encode(new_tree_hash_bytes);
        // Get commit hash from head. Early return if a detached head.
        // let last_branch_commit = Commit::get_branch_commit()?.ok_or_else(|| std::io::Error::other("Detached head. Not implemented"))?;
        // // Read tree hash from last commit
        // let last_tree_hash = Commit::get_tree_from_commit(&last_branch_commit)?;

        // Read head and get path.
        let head_str = Commit::read_head()?;
        let branch_path = Commit::get_branch_from_head(&head_str)?;
        // Check if it exists
        let mut parent_commits: Vec<String> = vec![];
        if !Path::new(&branch_path).exists() {
            // Create the branch file and leave parents_branches empty
            std::fs::create_dir_all(branch_path.parent().unwrap())?;
            std::fs::File::create_new(&branch_path)?;
        } else {
            // Read it and push to parent_branches
            let parent_hash = Commit::get_branch_commit()?;
            parent_commits.push(parent_hash.clone());
            let parent_commit = Commit::decode(&parent_hash)?;
            let last_tree_hash = parent_commit.tree_hash;

            // Use root tree hash to check if there's anything new in staging
            if new_tree_hash == last_tree_hash {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Commit object not found",
                ));
            };
        }
        // If we can commit, write the trees to file...
        Tree::write_object_to_file(trees)?;

        // ..and the new commit
        let commit = Commit::encode(new_tree_hash, parent_commits, &message)?;
        let new_commit_hash = commit.write_commit_to_file()?;

        // Update the branch to point to the new commit
        let branch = Commit::update_branch_hash(&new_commit_hash)?;
        let commit_summary = CommitSummary {
            branch,
            commit_hash: new_commit_hash,
            message,
        };
        // Update reflog
        // TODO

        // Show summary w/ git diff - TODO
        println!("{commit_summary}");
        Ok(())
    }
}
