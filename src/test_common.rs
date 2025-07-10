use clap::{Arg, ArgAction, ArgMatches, Command, command};
use std::{
    ops::Deref,
    path::{Path, PathBuf},
    sync::{LazyLock, Mutex},
};
use tempfile::{Builder, NamedTempFile, TempDir};

pub struct TestDir {
    root: TempDir,
}

#[allow(dead_code)]
pub struct TestSetup {
    pub dir: TestDir,
    // Remove this
    key: &'static str,
}

impl TestDir {
    pub fn new_dir() -> Self {
        let root = Builder::new()
            .prefix("test_dir_")
            .rand_bytes(4)
            .tempdir()
            .expect("Failed to create test dir");
        TestDir { root }
    }

    pub fn path(&self) -> &Path {
        &self.root.path()
    }
}

#[allow(dead_code)]
pub fn run_test<T>(key: &'static str, f: T)
where
    T: FnOnce(&TestSetup),
{
    let dir = TestDir::new_dir();
    let r = TestSetup { dir, key };
    f(&r);
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

pub fn cat_file_pretty(hash: &str) -> ArgMatches {
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

    let mut matches = matches.get_matches_from(["git_rust", "cat-file", "-p", &hash]);
    let (_, arg) = matches.remove_subcommand().unwrap();
    arg
}

// Rename the git folder for the crate
// Test each one individually and compare results
// Copy a file from one to the other and compare results

// Create tempdir called .git_rust in the parent dir
// Init git_rust in there and rename the one in the root dir
