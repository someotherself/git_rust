use std::{io::Write, os::unix::fs::MetadataExt, path::PathBuf};

use crate::{
    git_rust::{self, BASE_DIR},
    index::Index,
    objects::{GitObject, blob},
    test_common::{run_test, run_test_matches},
};

#[test]
fn test_init_repo_in_temp_folder() {
    run_test(|setup| {
        let setup = setup.lock().unwrap().take().unwrap();
        let path = &setup.dir;

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
        let setup = setup.lock().unwrap().take().unwrap();
        let path = &setup.dir;

        let result = git_rust::RepoRust::new_repo(path.to_str().unwrap());
        assert!(result.is_ok());
        result.unwrap();

        let result = git_rust::RepoRust::new_repo(path.to_str().unwrap());
        assert!(matches!(result, Err(e) if e.to_string() == "Repo already initialized"));
    });
}

#[test]
fn test_hash_file_without_init() {
    run_test(|setup| {
        let setup = setup.lock().unwrap().take().unwrap();
        let path = &setup.dir;

        // -- Try hashing '-w' without initalizing repo
        let file_path = path.join("test1.txt");
        let file_path_str = file_path.to_str().unwrap();
        let mut file = std::fs::File::create(&file_path).unwrap();
        file.write_all(b"this is a test").unwrap();

        let args_1 = run_test_matches(vec!["", "hash-object", "-w", &file_path_str]);
        let result_1 = blob::Blob::encode_object(&args_1);
        // SHOULD BE ERROR TODO
        assert!(result_1.is_ok());
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
        let setup = setup.lock().unwrap().take().unwrap();
        let path = &setup.dir;

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
        dbg!(path.join(BASE_DIR).join("objects"));
    });
}

#[test]
fn test_hash_file_in_temp_folder() {
    run_test(|setup| {
        // Get test dir
        let setup = setup.lock().unwrap().take().unwrap();
        let path = &setup.dir;
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
        let repo = Repository::init(path.path()).unwrap();
        let blob_oid = repo.blob_path(&file_path).unwrap();
        let git2_hash = repo.find_blob(blob_oid).unwrap();

        // Compare against git
        assert_eq!(git2_hash.id().to_string(), format!("{}", git_rust_hash));

        // cat-file the object
        let cat_args = run_test_matches(vec!["", "cat-file", "-p", &format!("{}", git_rust_hash)]);
        let git_rust_content = blob::Blob::decode_object(&cat_args).unwrap();

        let oid = repo.find_blob(blob_oid).unwrap();
        let git2_content = oid.content();

        assert_eq!(
            format!("{}", str::from_utf8(git2_content).unwrap()),
            format!("{}", git_rust_content)
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
        let setup = setup.lock().unwrap().take().unwrap();
        let path = &setup.dir;
        // Create file to hash
        let file_path_1 = path.join("test1.txt");
        let file_path_str_1 = file_path_1.to_str().unwrap();
        let mut file_1 = std::fs::File::create(&file_path_1).unwrap();
        file_1.write_all(b"this is a test").unwrap();
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
        let setup = setup.lock().unwrap().take().unwrap();
        let path = &setup.dir;
        // Create file to hash
        let file_path_1 = path.join("test1.txt");
        let file_path_str_1 = file_path_1.to_str().unwrap();
        let mut file_1 = std::fs::File::create(&file_path_1).unwrap();
        file_1.write_all(b"this is a test").unwrap();

        git_rust::RepoRust::new_repo(path.to_str().unwrap()).unwrap();
        git_rust::RepoRust::init().unwrap();
        let index_path = &git_rust::RepoRust::get_root()
            .unwrap()
            .base_path
            .join(BASE_DIR)
            .join("INDEX");

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
        let expected_sha1 = Index::sha1_entry(file_contents_1).unwrap();
        assert_eq!(expected_sha1, entry.sha1);

        // Compare SHA-1 with real git
        use git2::Repository;
        let repo = Repository::init(path.path()).unwrap();
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
        let expected_sha1_2 = Index::sha1_entry(file_contents_2).unwrap();
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
        let setup = setup.lock().unwrap().take().unwrap();
        let path = &setup.dir;
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

#[test]
fn test_git_add_folder() {
    run_test(|setup| {
        // Get test dir
        let mut input = setup.lock().unwrap();
        let setup = input.take().unwrap();
        let path = &setup.dir;
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
        let add_args = run_test_matches(vec!["", "add", path_folder_1_str]);
        git_rust::RepoRust::add(&add_args).unwrap();
        let _index = Index::read_index().unwrap();

        let read_index = Index::read_index();
        assert!(read_index.is_ok());
        let index = read_index.unwrap();

        assert_eq!(index.header.sign, [b'D', b'I', b'R', b'C']);
        assert_eq!(index.header.version, 2_u32.to_be_bytes());
        assert_eq!(index.header.entries, 5_u32.to_be_bytes());

        for (idx, (path, entry)) in index.entries.iter().enumerate() {
            assert_eq!(
                PathBuf::from(path),
                path_folder_1.join(format!("file_{}", idx))
            );
            assert_eq!(entry.flags & 0x0FFF, 67);
        }
    });
}
