use std::{
    collections::BTreeMap,
    io::Write,
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
};

use crate::{
    git_rust::{self, BASE_DIR},
    index::Index,
    objects::{blob, tree::Tree},
    test_common::{run_test, run_test_matches},
};

#[test]
fn test_init_repo_in_temp_folder() {
    run_test(|setup| {
        let setup = setup.lock().unwrap().take().unwrap().dir;
        let path = PathBuf::from(&setup.test_dir);

        git_rust::RepoRust::new_repo(path.to_str().unwrap()).unwrap();

        // Check succesful initialization
        let result = git_rust::RepoRust::init();
        assert!(result.is_ok());
        result.unwrap();
        // Test re-initialization will fail
        assert!(git_rust::RepoRust::init().is_err());
    });
}

#[test]
fn test_init_repo_struct_in_temp_folder() {
    run_test(|setup| {
        let setup = setup.lock().unwrap().take().unwrap().dir;
        let path = PathBuf::from(&setup.test_dir);

        let result = git_rust::RepoRust::new_repo(path.to_str().unwrap());
        assert!(result.is_ok());

        let result = git_rust::RepoRust::new_repo(path.to_str().unwrap());
        assert!(result.is_ok());
    });
}

// Only works if cargo run init is not run either
#[ignore]
#[test]
fn test_hash_file_without_init() {
    run_test(|setup| {
        let setup = setup.lock().unwrap().take().unwrap().dir;
        let path = PathBuf::from(&setup.test_dir);

        // -- Try hashing '-w' without initalizing repo
        let file_path = path.join("test1.txt");
        let file_path_str = file_path.to_str().unwrap();
        let mut file = std::fs::File::create(&file_path).unwrap();
        file.write_all(b"this is a test").unwrap();

        let args_1 = run_test_matches(vec!["", "hash-object", "-w", &file_path_str]);
        let result_1 = blob::Blob::encode_object(&args_1);
        // SHOULD BE ERROR
        assert!(result_1.is_err());

        // -- Without '-w' should still work
        let args_2 = run_test_matches(vec!["", "hash-object", &file_path_str]);
        let result_2 = blob::Blob::encode_object(&args_2);
        // Should be ok
        assert!(result_2.is_ok());
    });
}

#[test]
fn test_hash_file_without_w() {
    run_test(|setup| {
        let setup = setup.lock().unwrap().take().unwrap().dir;
        let path = PathBuf::from(&setup.test_dir);

        // -- Try hashing '-w' without initalizing repo
        let file_path = path.join("test1.txt");
        let file_path_str = file_path.to_str().unwrap();
        let mut file = std::fs::File::create(&file_path).unwrap();
        file.write_all(b"this is a test").unwrap();

        git_rust::RepoRust::new_repo(path.to_str().unwrap()).unwrap();
        git_rust::RepoRust::init().unwrap();
        let args = run_test_matches(vec!["", "hash-object", &file_path_str]);

        let git_rust_hash = blob::Blob::encode_object(&args).unwrap();

        let (folder_name, file_name) = git_rust_hash.hash.split_at(2);
        let folder_path = path.join(BASE_DIR).join("objects").join(folder_name);
        let file_path = folder_path.join(file_name);
        // Make sure objects folder exists
        assert!(path.join(BASE_DIR).join("objects").exists());
        // Make sure blob does not exist
        assert!(!file_path.exists());
        assert!(!folder_path.exists());
        // Make sure the objects folder is empty
        let mut entries = path.join(BASE_DIR).join("objects").read_dir().unwrap();
        assert!(entries.next().is_none());
    });
}

#[test]
fn test_hash_file_in_temp_folder() {
    run_test(|setup| {
        // Get test dir
        let setup = setup.lock().unwrap().take().unwrap().dir;
        let path = PathBuf::from(&setup.test_dir);

        // -- Hash a file, compate with git and then
        // Create file to hash
        let file_path = path.join("test1.txt");
        let file_path_str = file_path.to_str().unwrap();
        let mut file = std::fs::File::create(&file_path).unwrap();
        file.write_all(b"this is a test").unwrap();

        git_rust::RepoRust::new_repo(path.to_str().unwrap()).unwrap();
        let args = run_test_matches(vec!["", "hash-object", "-w", &file_path_str]);

        // Check succesful initialization
        let result = git_rust::RepoRust::init();
        assert!(result.is_ok());
        result.unwrap();
        // Test re-initialization will fail
        assert!(git_rust::RepoRust::init().is_err());
        let git_rust_hash = blob::Blob::encode_object(&args).unwrap();

        // -- Hash file again. Make sure hash is the same --
        let git_rust_hash_dup = blob::Blob::encode_object(&args).unwrap();
        assert_eq!(git_rust_hash, git_rust_hash_dup);

        // hash-object with git
        use git2::Repository;
        let repo = Repository::init(path.clone()).unwrap();
        let blob_oid = repo.blob_path(&file_path).unwrap();
        let git2_hash = repo.find_blob(blob_oid).unwrap();

        // Compare against git
        assert_eq!(git2_hash.id().to_string(), format!("{}", git_rust_hash));

        // cat-file the object
        let git_rust_content = blob::Blob::decode_object(&git_rust_hash.hash).unwrap();

        let oid = repo.find_blob(blob_oid).unwrap();
        let git2_content = oid.content();

        assert_eq!(
            format!("{}", str::from_utf8(git2_content).unwrap()),
            format!("{}", String::from_utf8(git_rust_content).unwrap())
        );

        // --  Compare hash of files with new line '\n' and without --
        // Create file to hash
        let file_path_2 = path.join("test2.txt");
        let file_path_str_2 = file_path_2.to_str().unwrap();
        let mut file_2 = std::fs::File::create(&file_path_2).unwrap();
        file_2.write_all(b"this is a test\n").unwrap();

        // Hash new file
        let args = run_test_matches(vec!["", "hash-object", "-w", &file_path_str_2]);
        let git_rust_hash_2 = blob::Blob::encode_object(&args).unwrap();

        // Should not equal
        assert_ne!(format!("{}", git_rust_hash_2), format!("{}", git_rust_hash));

        // --  Hash a file with no content --
        // Create file to hash
        let file_path_3 = path.join("test3.txt");
        let file_path_str_3 = file_path_3.to_str().unwrap();
        let mut file_3 = std::fs::File::create(&file_path_3).unwrap();
        file_3.write_all(b"      ").unwrap();

        // Hash new file
        let args = run_test_matches(vec!["", "hash-object", "-w", &file_path_str_3]);
        let git_rust_hash_3 = blob::Blob::encode_object(&args).unwrap();

        // e69de29bb2d1d6434b8b29ae775ad8c2e48c5391 = "blob 0\0"
        assert_ne!(
            format!("{}", git_rust_hash_3),
            "e69de29bb2d1d6434b8b29ae775ad8c2e48c5391".to_string()
        );

        // -- Hash a non existent file --
        let args = run_test_matches(vec!["", "hash-object", "-w", "fake_file"]);
        let result_1 = blob::Blob::encode_object(&args);
        assert!(result_1.is_err());

        // Hash a dir
        // Create a dir
        let dir_1 = path.join("new_dir");
        let dir_path_str_1 = dir_1.to_str().unwrap();
        std::fs::create_dir_all(&dir_path_str_1).unwrap();

        let args = run_test_matches(vec!["", "hash-object", "-w", &dir_path_str_1]);
        let result_2 = blob::Blob::encode_object(&args);
        assert!(result_2.is_err());
    });
}

#[test]
fn test_hash_cat_raw_bytes() {
    run_test(|setup| {
        let setup = setup.lock().unwrap().take().unwrap().dir;
        let path = PathBuf::from(&setup.test_dir);
        // Create file to hash
        let file_path_1 = path.join("test1.txt");
        let file_path_str_1 = file_path_1.to_str().unwrap();
        let mut file_1 = std::fs::File::create(&file_path_1).unwrap();
        file_1
            .write_all(&[0x00, 0xFF, 0xFE, 0x41, 0x42, 0x00])
            .unwrap();

        git_rust::RepoRust::new_repo(path.to_str().unwrap()).unwrap();
        git_rust::RepoRust::init().unwrap();
        let args = run_test_matches(vec!["", "hash-object", "-w", &file_path_str_1]);

        let git_rust_hash = blob::Blob::encode_object(&args).unwrap();

        // cat-file the object
        let git_rust_content = blob::Blob::decode_object(&git_rust_hash.hash).unwrap();

        assert_eq!(git_rust_content, &[0x00, 0xFF, 0xFE, 0x41, 0x42, 0x00]);
    });
}

// Can cause damage if other tests fail
#[test]
#[ignore]
fn test_repo_in_project_dir() {
    run_test(|_setup| {
        // Test normal function. Create repo in project folder.
        // If a repo exists, rename it and fixed after the test.
        let cwd = std::env::current_dir().unwrap();
        let cur_repo = cwd.join(".git_rust");
        let new_repo = cwd.join(".temp_git_rust");
        let repo_already_exists = cur_repo.exists();

        if repo_already_exists {
            std::fs::rename(&cur_repo, &new_repo).expect("Existing repo could not be ranemed");
        }
        assert!(!cur_repo.exists());
        assert!(new_repo.exists());

        // Initialize repo
        let result = git_rust::RepoRust::init();
        assert!(result.is_ok());
        result.unwrap();
        assert!(cur_repo.exists());

        // Test re-initialization will fail
        assert!(git_rust::RepoRust::init().is_err());

        // Delete the temp repo
        std::fs::remove_dir_all(&cur_repo).expect("Failed to remove temp repo");

        if repo_already_exists && !cur_repo.exists() {
            std::fs::rename(&new_repo, &cur_repo).expect("Temp repo could not be ranemed");
        }
        assert!(cur_repo.exists());
        assert!(!new_repo.exists());
    });
}

#[test]
fn test_git_add_files() {
    run_test(|setup| {
        // Get test dir
        let setup = setup.lock().unwrap().take().unwrap().dir;
        let path = PathBuf::from(&setup.test_dir);
        // Create file to hash
        let file_path_1 = path.join("test1.txt");
        let file_path_str_1 = file_path_1.to_str().unwrap();
        let mut file_1 = std::fs::File::create(&file_path_1).unwrap();
        file_1.write_all(b"this is a test").unwrap();

        git_rust::RepoRust::new_repo(path.to_str().unwrap()).unwrap();
        git_rust::RepoRust::init().unwrap();
        let index_path = &git_rust::RepoRust::get_root()
            .absolute_path
            .join(BASE_DIR)
            .join("index");

        // INDEX one file
        let add_args = run_test_matches(vec!["", "add", &format!("{}", file_path_str_1)]);
        let result = git_rust::RepoRust::add(&add_args);
        assert!(result.is_ok());
        result.unwrap();
        assert!(index_path.exists());
        let read_index = Index::read_index();
        assert!(read_index.is_ok());
        let index = read_index.unwrap();

        assert_eq!(index.header.sign, [b'D', b'I', b'R', b'C']);
        assert_eq!(index.header.version, 2_u32.to_be_bytes());
        assert_eq!(index.header.entries, 1_u32.to_be_bytes());

        // There should be one entry
        assert_eq!(index.entries.len(), 1);

        // Get the entry for the indexed file
        let entry = index
            .entries
            .get(file_path_str_1)
            .expect("Missing entry in index");

        let metadata = file_path_1.metadata().unwrap();

        // Check the important metadata fields
        assert_eq!(entry.file_size, metadata.len() as u32);
        assert_eq!(entry.dev, metadata.dev() as u32);
        assert_eq!(entry.ino, metadata.ino() as u32);
        assert_eq!(entry.mode, metadata.mode());
        assert_eq!(entry.uid, metadata.uid());
        assert_eq!(entry.gid, metadata.gid());

        // Allow slight time diff between file creation and index writing
        assert!((entry.ctime as i64 - metadata.ctime()) <= 1);
        assert!((entry.mtime as i64 - metadata.mtime()) <= 1);

        // Validate SHA-1 hash
        let file_contents_1 = std::fs::read(&file_path_1).unwrap();
        let expected_sha1 = Index::sha1_entry(&file_contents_1);
        assert_eq!(expected_sha1, entry.sha1);

        // Compare SHA-1 with real git
        use git2::Repository;
        let repo = Repository::init(&path).unwrap();
        let blob_oid = repo.blob_path(&file_path_1).unwrap();
        let git2_hash = repo.find_blob(blob_oid).unwrap();
        assert_eq!(expected_sha1, git2_hash.id().as_bytes());
        assert_eq!(entry.sha1, git2_hash.id().as_bytes());

        // INDEX a second file
        let file_path_2 = path.join("test2.txt");
        let file_path_str_2 = file_path_2.to_str().unwrap();
        let mut file_2 = std::fs::File::create(&file_path_2).unwrap();
        file_2.write_all(b"this is second test").unwrap();

        let add_args_2 = run_test_matches(vec!["", "add", &format!("{}", file_path_str_2)]);
        let result = git_rust::RepoRust::add(&add_args_2);
        assert!(result.is_ok());
        result.unwrap();

        let read_index = Index::read_index();
        assert!(read_index.is_ok());
        let index = read_index.unwrap();

        assert_eq!(index.header.sign, [b'D', b'I', b'R', b'C']);
        assert_eq!(index.header.version, 2_u32.to_be_bytes());
        assert_eq!(index.header.entries, 2_u32.to_be_bytes());

        // There should be two entries
        assert_eq!(index.entries.len(), 2);

        // Get the entry for the indexed file
        let entry_2 = index
            .entries
            .get(file_path_str_2)
            .expect("Missing entry in index");

        let metadata_2 = file_path_2.metadata().unwrap();

        // Check the important metadata fields
        assert_eq!(entry_2.file_size, metadata_2.len() as u32);
        assert_eq!(entry_2.dev, metadata_2.dev() as u32);
        assert_eq!(entry_2.ino, metadata_2.ino() as u32);
        assert_eq!(entry_2.mode, metadata_2.mode());
        assert_eq!(entry_2.uid, metadata_2.uid());
        assert_eq!(entry_2.gid, metadata_2.gid());

        // Validate SHA-1 hash
        let file_contents_2 = std::fs::read(&file_path_2).unwrap();
        let expected_sha1_2 = Index::sha1_entry(&file_contents_2);
        assert_eq!(expected_sha1_2, entry_2.sha1);

        // Compare SHA-1 with real git
        let blob_oid_2 = repo.blob_path(&file_path_2).unwrap();
        let git2_hash_2 = repo.find_blob(blob_oid_2).unwrap();
        assert_eq!(expected_sha1_2, git2_hash_2.id().as_bytes());
        assert_eq!(entry_2.sha1, git2_hash_2.id().as_bytes());

        // Add the same file twice
        git_rust::RepoRust::add(&add_args_2).unwrap();

        // There should be two entries
        assert_eq!(index.entries.len(), 2);
    });
}

#[test]
fn test_git_add_same_file_twice() {
    run_test(|setup| {
        // Get test dir
        let setup = setup.lock().unwrap().take().unwrap().dir;
        let path = PathBuf::from(&setup.test_dir);
        // Create file to hash
        let file_path_1 = path.join("test1.txt");
        let file_path_str_1 = file_path_1.to_str().unwrap();
        let mut file_1 = std::fs::File::create(&file_path_1).unwrap();
        file_1.write_all(b"this is a test").unwrap();

        git_rust::RepoRust::new_repo(path.to_str().unwrap()).unwrap();
        git_rust::RepoRust::init().unwrap();

        // INDEX file once
        let add_args = run_test_matches(vec!["", "add", &format!("{}", file_path_str_1)]);
        git_rust::RepoRust::add(&add_args).unwrap();
        let index = Index::read_index().unwrap();

        assert_eq!(index.header.sign, [b'D', b'I', b'R', b'C']);
        assert_eq!(index.header.version, 2_u32.to_be_bytes());
        assert_eq!(index.header.entries, 1_u32.to_be_bytes());

        // INDEX the same file again
        git_rust::RepoRust::add(&add_args).unwrap();
        let index = Index::read_index().unwrap();

        assert_eq!(index.header.sign, [b'D', b'I', b'R', b'C']);
        assert_eq!(index.header.version, 2_u32.to_be_bytes());
        assert_eq!(index.header.entries, 1_u32.to_be_bytes());
    });
}

// TODO: Check what happend when calling add on an empty root or folder
#[test]
fn test_git_add_folder() {
    run_test(|setup| {
        // Get test dir
        let setup = setup.lock().unwrap().take().unwrap().dir;
        let path = PathBuf::from(&setup.test_dir);
        // Create file to hash
        let path_folder_1 = path.join("folder_1");
        let path_folder_1_str = path_folder_1.to_str().unwrap();
        std::fs::create_dir_all(&path_folder_1).unwrap();
        for i in 0..5 {
            let mut file =
                std::fs::File::create_new(path_folder_1.join(format!("file_{}", i))).unwrap();
            file.write_all(format!("This is test file number {}", i).as_bytes())
                .unwrap();
        }
        assert!(path_folder_1.is_dir());

        git_rust::RepoRust::new_repo(path.to_str().unwrap()).unwrap();
        git_rust::RepoRust::init().unwrap();

        // INDEX the folder
        let add_args_1 = run_test_matches(vec!["", "add", path_folder_1_str]);
        git_rust::RepoRust::add(&add_args_1).unwrap();

        let read_index = Index::read_index();
        assert!(read_index.is_ok());
        let index = read_index.unwrap();

        assert_eq!(index.header.sign, [b'D', b'I', b'R', b'C']);
        assert_eq!(index.header.version, 2_u32.to_be_bytes());
        assert_eq!(index.header.entries, 5_u32.to_be_bytes());
        let file_len = path_folder_1.join("file_0").as_os_str().len();

        for (idx, (path, entry)) in index.entries.iter().enumerate() {
            assert_eq!(
                PathBuf::from(path),
                path_folder_1.join(format!("file_{}", idx))
            );
            assert_eq!(entry.flags & 0x0FFF, file_len as u16);
        }

        // Continue adding more files from root
        let file_path_1 = path.join("test1.txt");
        let file_path_str_1 = file_path_1.to_str().unwrap();
        let mut file_1 = std::fs::File::create_new(&file_path_1).unwrap();
        file_1.write_all(b"this is a test").unwrap();

        let file_path_2 = path.join("test2.txt");
        let file_path_str_2 = file_path_2.to_str().unwrap();
        let mut file_2 = std::fs::File::create_new(&file_path_2).unwrap();
        file_2.write_all(b"this is a test.").unwrap();

        // INDEX the files
        let add_args_2 = run_test_matches(vec!["", "add", file_path_str_1]);
        git_rust::RepoRust::add(&add_args_2).unwrap();

        // INDEX the files
        let add_args_3 = run_test_matches(vec!["", "add", file_path_str_2]);
        git_rust::RepoRust::add(&add_args_3).unwrap();

        let index = Index::read_index().unwrap();

        assert_eq!(index.header.sign, [b'D', b'I', b'R', b'C']);
        assert_eq!(index.header.version, 2_u32.to_be_bytes());
        assert_eq!(index.header.entries, 7_u32.to_be_bytes());
    });
}

#[test]
fn test_git_write_trees() {
    run_test(|setup| {
        // Get test dir
        let setup = setup.lock().unwrap().take().unwrap().dir;
        let path = PathBuf::from(&setup.test_dir);
        // Create file to hash
        let file_path_1 = path.join("test1.txt");
        let file_path_str_1 = file_path_1.to_str().unwrap();
        let mut file_1 = std::fs::File::create(&file_path_1).unwrap();
        file_1.write_all(b"this is test 1").unwrap();

        let file_path_2 = path.join("test2.txt");
        let file_path_str_2 = file_path_2.to_str().unwrap();
        let mut file_2 = std::fs::File::create(&file_path_2).unwrap();
        file_2.write_all(b"this is test 2").unwrap();

        let file_path_3 = path.join("test3.txt");
        let file_path_str_3 = file_path_3.to_str().unwrap();
        let mut file_3 = std::fs::File::create(&file_path_3).unwrap();
        file_3.write_all(b"this is test 3").unwrap();

        git_rust::RepoRust::new_repo(path.to_str().unwrap()).unwrap();
        git_rust::RepoRust::init().unwrap();

        // INDEX the files
        let add_args_1 = run_test_matches(vec!["", "add", file_path_str_1]);
        let add_args_2 = run_test_matches(vec!["", "add", file_path_str_2]);
        let add_args_3 = run_test_matches(vec!["", "add", file_path_str_3]);
        git_rust::RepoRust::add(&add_args_1).unwrap();
        git_rust::RepoRust::add(&add_args_2).unwrap();
        git_rust::RepoRust::add(&add_args_3).unwrap();

        let index = Index::read_index().unwrap();

        let entries_by_folder = Tree::group_entries_for_tree_build(index.entries);
        Tree::build_trees(entries_by_folder);
    });
}

#[test]
fn test_compare_index_with_git() {
    run_test(|setup| {
        // Get test dir
        let setup = setup.lock().unwrap().take().unwrap().dir;
        let path = PathBuf::from(&setup.test_dir);

        // Create files in root
        for i in 0..3 {
            let mut file =
                std::fs::File::create_new(path.join(PathBuf::from(format!("test{}.txt", i))))
                    .unwrap();
            file.write_all(format!("Test file {} in root", i).as_bytes())
                .unwrap();
        }

        // Create folder 1 and files to hash
        let path_folder_1 = path.join("folder_1");
        std::fs::create_dir_all(&path_folder_1).unwrap();
        for i in 0..5 {
            let mut file =
                std::fs::File::create_new(path_folder_1.join(format!("file_in_dir1_{}", i)))
                    .unwrap();
            file.write_all(format!("This is test file number {}", i).as_bytes())
                .unwrap();
        }

        // Create folder 2 in folder 1 and files to hash
        let path_folder_2 = path_folder_1.join("folder_2");
        std::fs::create_dir_all(&path_folder_2).unwrap();

        for i in 0..3 {
            let mut file =
                std::fs::File::create_new(path_folder_2.join(format!("file_in_dir2_{}", i)))
                    .unwrap();
            file.write_all(format!("This is test file number {}", i).as_bytes())
                .unwrap();
        }

        assert!(path_folder_1.exists());
        assert!(path_folder_2.exists());
        assert!(path_folder_1.join(format!("file_in_dir1_0")).exists());
        assert!(path_folder_1.join(format!("file_in_dir1_1")).exists());
        assert!(path_folder_1.join(format!("file_in_dir1_2")).exists());
        assert!(path_folder_1.join(format!("file_in_dir1_3")).exists());
        assert!(path_folder_1.join(format!("file_in_dir1_4")).exists());
        assert!(path_folder_2.join(format!("file_in_dir2_0")).exists());
        assert!(path_folder_2.join(format!("file_in_dir2_1")).exists());
        assert!(path_folder_2.join(format!("file_in_dir2_2")).exists());
        assert!(path.join(PathBuf::from("test1.txt")).exists());
        assert!(path.join(PathBuf::from("test2.txt")).exists());

        git_rust::RepoRust::new_repo(path.to_str().unwrap()).unwrap();
        git_rust::RepoRust::init().unwrap();

        // INDEX the root with git_rust
        let root_as_str = path.to_str().unwrap();
        let add_args = run_test_matches(vec!["", "add", root_as_str]);
        git_rust::RepoRust::add(&add_args).unwrap();

        let rust_index = Index::read_index().unwrap();

        // Create a .gitignore file for .git_rust
        std::fs::write(
            path.join(".gitignore"),
            ".git_rust/\n.gitignore\n.gitrust_ignore\n",
        )
        .unwrap();
        std::fs::write(
            path.join(".gitrust_ignore"),
            ".git_rust/\n.gitignore\n.gitrust_ignore\n",
        )
        .unwrap();

        // INDEX the root with git
        use git2::Repository;
        let repo = Repository::init(path).unwrap();
        let mut git_index = repo.index().unwrap();
        git_index
            .add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)
            .unwrap();
        git_index.write().unwrap();

        let mut git_map: BTreeMap<String, git2::IndexEntry> = BTreeMap::new();
        for entry in git_index.iter() {
            let path_str = std::str::from_utf8(&entry.path).unwrap().to_string();
            git_map.insert(path_str, entry);
        }

        assert_eq!(
            rust_index.entries.len(),
            git_map.len(),
            "Index entry count mismatch"
        );
    });
}

#[test]
fn test_read_index_created_by_git() {
    run_test(|setup| {
        // Get test dir
        let setup = setup.lock().unwrap().take().unwrap().dir;
        let path = PathBuf::from(&setup.test_dir);

        // Create files in root
        for i in 0..3 {
            let mut file =
                std::fs::File::create_new(path.join(PathBuf::from(format!("test{}.txt", i))))
                    .unwrap();
            file.write_all(format!("Test file {} in root", i).as_bytes())
                .unwrap();
        }

        // Create folder 1 and files to hash
        let path_folder_1 = path.join("folder_1");
        std::fs::create_dir_all(&path_folder_1).unwrap();
        for i in 0..5 {
            let mut file =
                std::fs::File::create_new(path_folder_1.join(format!("file_in_dir1_{}", i)))
                    .unwrap();
            file.write_all(format!("This is test file number {}", i).as_bytes())
                .unwrap();
        }

        // Create folder 2 in folder 1 and files to hash
        let path_folder_2 = path_folder_1.join("folder_2");
        std::fs::create_dir_all(&path_folder_2).unwrap();

        for i in 0..3 {
            let mut file =
                std::fs::File::create_new(path_folder_2.join(format!("file_in_dir2_{}", i)))
                    .unwrap();
            file.write_all(format!("This is test file number {}", i).as_bytes())
                .unwrap();
        }

        // INDEX the root with git
        use git2::Repository;
        let repo = Repository::init(&path).unwrap();
        let mut index = repo.index().unwrap();
        index
            .add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)
            .unwrap();
        index.write().unwrap();

        let git_index = path.join(".git").join("index");
        assert!(git_index.exists());

        git_rust::RepoRust::new_repo(path.to_str().unwrap()).unwrap();
        git_rust::RepoRust::init().unwrap();
        let repo_index = path.join(".git_rust").join("index");

        std::fs::copy(&git_index, &repo_index).unwrap();
        assert!(repo_index.exists());

        let index = Index::read_index().unwrap();
        for (idx, (_, entry)) in index.entries.iter().enumerate() {
            let all_files_indexed = Vec::from([
                "folder_1/file_in_dir1_0",
                "folder_1/file_in_dir1_1",
                "folder_1/file_in_dir1_2",
                "folder_1/file_in_dir1_3",
                "folder_1/file_in_dir1_4",
                "folder_1/folder_2/file_in_dir2_0",
                "folder_1/folder_2/file_in_dir2_1",
                "folder_1/folder_2/file_in_dir2_2",
                "test0.txt",
                "test1.txt",
                "test2.txt",
            ]);
            let path_str = str::from_utf8(&entry.path).unwrap();
            assert_eq!(path_str, all_files_indexed[idx]);
        }
        let path_to_check = "folder_1/file_in_dir1_2";
        let abs_path_to_check = path.join("folder_1/file_in_dir1_2");
        assert_eq!(index.entries.len(), 11);

        let entries = index.entries;
        let entry = entries.get(path_to_check).unwrap();
        let path_length = path_to_check.len();
        assert_eq!((entry.flags & 0x0FFF) as usize, path_length);
        let file_to_check = std::fs::File::open(abs_path_to_check).unwrap();

        let ctime = file_to_check.metadata().unwrap().ctime();
        let ctime_nano = file_to_check.metadata().unwrap().ctime_nsec();
        assert_eq!(entry.ctime as i64, ctime);
        assert_eq!(entry.ctime_nanos as i64, ctime_nano);

        let inode = file_to_check.metadata().unwrap().ino();
        assert_eq!(entry.ino as u64, inode);

        let mtime = file_to_check.metadata().unwrap().mtime();
        let mtime_nano = file_to_check.metadata().unwrap().mtime_nsec();
        assert_eq!(entry.mtime as i64, mtime);
        assert_eq!(entry.mtime_nanos as i64, mtime_nano);
    });
}

#[test]
fn test_git_ignore() {
    run_test(|setup| {
        // Get test dir
        let setup = setup.lock().unwrap().take().unwrap().dir;
        let path = PathBuf::from(&setup.test_dir);

        // Create files in root
        for i in 0..3 {
            let mut file =
                std::fs::File::create_new(path.join(PathBuf::from(format!("test{}.txt", i))))
                    .unwrap();
            file.write_all(format!("Test file {} in root", i).as_bytes())
                .unwrap();
        }

        // Create folder 1 and files to hash
        let path_folder_1 = path.join("folder_1");
        std::fs::create_dir_all(&path_folder_1).unwrap();
        for i in 0..5 {
            let mut file =
                std::fs::File::create_new(path_folder_1.join(format!("file_in_dir1_{}", i)))
                    .unwrap();
            file.write_all(format!("This is test file number {}", i).as_bytes())
                .unwrap();
        }

        // Create folder 2 in folder 1 and files to hash
        let path_folder_2 = path_folder_1.join("folder_2");
        std::fs::create_dir_all(&path_folder_2).unwrap();

        for i in 0..3 {
            let mut file =
                std::fs::File::create_new(path_folder_2.join(format!("file_in_dir2_{}", i)))
                    .unwrap();
            file.write_all(format!("This is test file number {}", i).as_bytes())
                .unwrap();
        }

        // Create .gitignore file - total 11 files
        // Ignore folder1/folder2 - 3 files
        // Ignore test0.txt - 1 file
        std::fs::write(
            path.join(".gitignore"),
            ".git_rust/\n.gitignore\nfolder_1/folder_2\ntest0.txt\ngitrust_ignore\n",
        )
        .unwrap();
        std::fs::write(
            path.join(".gitrust_ignore"),
            ".git_rust/\n.gitignore\nfolder_1/folder_2\ntest0.txt\n.gitrust_ignore\n",
        )
        .unwrap();

        git_rust::RepoRust::new_repo(path.to_str().unwrap()).unwrap();
        git_rust::RepoRust::init().unwrap();

        // INDEX the root with git_rust
        let root_as_str = path.to_str().unwrap();
        let add_args = run_test_matches(vec!["", "add", root_as_str]);
        git_rust::RepoRust::add(&add_args).unwrap();

        let rust_index = Index::read_index().unwrap();

        assert_eq!(rust_index.entries.len(), 11 - 4);
    });
}

#[test]
fn test_write_tree_one_file_in_root() {
    run_test(|setup| {
        // Get test dir
        let setup = setup.lock().unwrap().take().unwrap().dir;
        let path = PathBuf::from(&setup.test_dir);

        // Create file to hash
        let file_path_1 = path.join("test1.txt");
        // let file_path_str_1 = file_path_1.to_str().unwrap();
        let mut file_1 = std::fs::File::create(&file_path_1).unwrap();
        file_1.write_all(b"this is test 1").unwrap();

        let git_rust_objects_folder = path.join(".git_rust/objects");

        // init, add and write-tree with git_rust
        // TODO: Extra tree created in test for the git_rust folder.
        git_rust::RepoRust::new_repo(path.to_str().unwrap()).unwrap();
        git_rust::RepoRust::init().unwrap();

        let add_args = run_test_matches(vec!["", "add", "text.txt"]);
        git_rust::RepoRust::add(&add_args).unwrap();

        for folder in git_rust_objects_folder.read_dir().unwrap() {
            let folder = folder.unwrap();
            dbg!(folder);
        }

        let write_tree_args = run_test_matches(vec!["", "write-tree"]);
        git_rust::RepoRust::write_tree(&write_tree_args).unwrap();

        for folder in git_rust_objects_folder.read_dir().unwrap() {
            let folder = folder.unwrap();
            dbg!(folder);
        }

        // let git_rust_content = blob::Blob::decode_object("7b62f0cd9a737ce285a0425161ec56ca082f8af1").unwrap();
        // dbg!(String::from_utf8(git_rust_content).unwrap());

        // init, add and write-tree with git_rust
        use git2::Repository;
        let repo = Repository::init(&path).unwrap();
        let mut index = repo.index().unwrap();
        index.add_path(&Path::new("test1.txt")).unwrap();
        index.write().unwrap();

        let _tree_oid = index.write_tree().unwrap();

        let git_objects_folder = path.join(".git/objects");
        // Delete /pack and /info
        assert!(git_objects_folder.join("info").exists());
        std::fs::remove_dir(git_objects_folder.join("info")).unwrap();
        assert!(!git_objects_folder.join("info").exists());
        std::fs::remove_dir(git_objects_folder.join("pack")).unwrap();
        for folder in git_objects_folder.read_dir().unwrap() {
            let folder = folder.unwrap();
            dbg!(folder);
        }
    });
}
