use std::io::Write;

use crate::{
    git_rust,
    objects::{GitObject, blob},
    test_common::{run_test, run_test_matches},
};

#[test]
fn test_init_repo_in_temp_folder() {
    run_test(|setup| {
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
        let path = &setup.dir;

        let result = git_rust::RepoRust::new_repo(path.to_str().unwrap());
        assert!(result.is_ok());
        result.unwrap();

        let result = git_rust::RepoRust::new_repo(path.to_str().unwrap());
        assert!(matches!(result, Err(e) if e.to_string() == "Repo already initialized"));
    });
}

#[test]
fn test_hash_file_in_temp_folder() {
    run_test(|setup| {
        // Get test dir
        let path = &setup.dir;
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

        use git2::Repository;
        let repo = Repository::init(path.path()).unwrap();
        let blob_oid = repo.blob_path(&file_path).unwrap();
        let git2_hash = repo.find_blob(blob_oid).unwrap();

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
