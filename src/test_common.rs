use clap::{Arg, ArgAction, ArgMatches, Command, command};
use std::{
    ops::Deref,
    path::{Path, PathBuf},
    sync::Mutex,
};
use tempfile::{Builder, TempDir};
use thread_local::ThreadLocal;

use crate::git_rust::RepoRust;

pub static SETUP_RESULT: ThreadLocal<Mutex<Option<TestSetup>>> = ThreadLocal::new();

#[allow(dead_code)]
pub struct TestDir {
    pub root: TempDir,
    pub test_dir: PathBuf,
}

#[allow(dead_code)]
pub struct TestSetup {
    pub dir: TestDir,
}

impl TestDir {
    pub fn new_dir() -> Self {
        let cwd = std::env::current_dir().expect("Failed to fetch cwd");
        let root = Builder::new()
            .prefix("test_dir_")
            .rand_bytes(4)
            .tempdir_in(cwd)
            .expect("Failed to create test dir");
        let temp_dir = PathBuf::from(root.path().file_name().unwrap());
        TestDir {
            root,
            test_dir: temp_dir,
        }
    }

    pub fn path(&self) -> &Path {
        &self.test_dir
    }
}

pub fn run_test<T>(f: T)
where
    T: FnOnce(&Mutex<Option<TestSetup>>) + Send + 'static,
{
    let setup = SETUP_RESULT.get_or(|| Mutex::new(None));
    {
        let dir = TestDir::new_dir();
        let mut setup = setup.lock().unwrap();
        *setup = Some(TestSetup { dir });
    }
    f(setup);
    RepoRust::clear_repo();
}

impl Deref for TestDir {
    type Target = Path;
    fn deref(&self) -> &Self::Target {
        self.path()
    }
}

impl AsRef<Path> for TestDir {
    fn as_ref(&self) -> &Path {
        self.path()
    }
}

fn cat_file_mock(args: Vec<&str>) -> ArgMatches {
    let matches = command!().subcommand(
        Command::new("cat-file")
            .arg(
                Arg::new("pretty")
                    .short('p')
                    .help("Pretty print the object's contents")
                    .action(ArgAction::SetTrue),
            )
            .arg(Arg::new("hash").required(true).value_name("HASH")),
    );

    let mut matches = matches.get_matches_from(args);
    let (_, arg) = matches.remove_subcommand().unwrap();
    arg
}

fn hash_object_mock(args: Vec<&str>) -> ArgMatches {
    let matches = command!().subcommand(
        Command::new("hash-object")
            .about("Compute object ID and optionally create an object from a file")
            .arg(
                Arg::new("write")
                    .short('w')
                    .help("Actually write the object into the object database.")
                    .action(ArgAction::SetTrue),
            )
            .arg(Arg::new("file").required(true).value_name("FILE")),
    );
    let mut matches = matches.get_matches_from(args);
    let (_, arg) = matches.remove_subcommand().unwrap();
    arg
}

fn add_mock(args: Vec<&str>) -> ArgMatches {
    let matches = command!().subcommand(
        Command::new("add")
            .about("Update the index using the content found in the working tree.")
            .arg(Arg::new("path").required(true).value_name("path")),
    );
    let mut matches = matches.get_matches_from(args);
    let (_, arg) = matches.remove_subcommand().unwrap();
    arg
}

fn ls_tree_mock(args: Vec<&str>) -> ArgMatches {
    let matches = command!().subcommand(
        Command::new("ls-tree")
            .about("List the contents of a tree object")
            .arg(Arg::new("hash").required(true).value_name("HASH")),
    );
    let mut matches = matches.get_matches_from(args);
    let (_, arg) = matches.remove_subcommand().unwrap();
    arg
}

fn write_tree_mock(args: Vec<&str>) -> ArgMatches {
    let matches = command!().subcommand(
        Command::new("write-tree").about("Create a tree object from the current index"),
    );
    let mut matches = matches.get_matches_from(args);
    let (_, arg) = matches.remove_subcommand().unwrap();
    arg
}

fn commit_tree_mock(args: Vec<&str>) -> ArgMatches {
    let matches = command!().subcommand(
        Command::new("commit-tree")
            .about("Create a new commit object")
            .arg(
                Arg::new("hash")
                    .required(true)
                    .value_name("HASH")
                    .help("The tree object hash to commit"),
            )
            .arg(
                Arg::new("commit")
                    .short('p')
                    .value_name("COMMIT")
                    .action(clap::ArgAction::Append)
                    .help("Optional parent commit hash (for non-root commits)"),
            )
            .arg(
                Arg::new("message")
                    .short('m')
                    .value_name("MESSAGE")
                    .default_value("")
                    .help("Commit message (if not provided, reads from stdin)"),
            ),
    );
    let mut matches = matches.get_matches_from(args);
    let (_, arg) = matches.remove_subcommand().unwrap();
    arg
}

fn commit_mock(args: Vec<&str>) -> ArgMatches {
    let matches = command!().subcommand(
            Command::new("commit").about("Record changes to the repository")
            .arg(
                Arg::new("add")
                .short('a')
                .action(ArgAction::SetTrue)
                .help("Automatically stage files that have been modified and deleted, but new files you have not told Git about are not affected."))
            .arg(Arg::new("message")
                .short('m')
                .value_name("MESSAGE")
                .help("Add a commit message.")));
    let mut matches = matches.get_matches_from(args);
    let (_, arg) = matches.remove_subcommand().unwrap();
    arg
}

pub fn run_test_matches(args: Vec<&str>) -> ArgMatches {
    match args[1] {
        "cat-file" => return cat_file_mock(args),
        "hash-object" => return hash_object_mock(args),
        "ls-tree" => return ls_tree_mock(args),
        "add" => return add_mock(args),
        "write-tree" => return write_tree_mock(args),
        "commit-tree" => return commit_tree_mock(args),
        "commit" => return commit_mock(args),
        _ => panic!("Wrong test command!"),
    }
}
