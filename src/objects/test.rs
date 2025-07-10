use std::io::Write;

use super::*;
use crate::{
    git_rust,
    test_common::{run_test, run_test_matches},
};

#[test]
fn test_hash_file() {
    run_test(|setup| {

        let path = &setup.dir;
        let file_path = path.join("test1.txt");
        let file_path_str = file_path.to_str().unwrap();
        let mut file = std::fs::File::create(&file_path).unwrap();
        file.write_all(b"this is a test").unwrap();
        
        git_rust::RepoRust::new_repo(path.to_str().unwrap()).expect("errro repo");
        let args = run_test_matches(vec!["", "hash-object", "-w", &file_path_str]);
        git_rust::RepoRust::init().unwrap();
        let git_rust_hash = blob::Blob::encode_object(&args).unwrap();

        use git2::Repository;
        let repo = Repository::init(path.path()).unwrap();
        let blob_oid = repo.blob_path(&file_path).unwrap();
        let git2_hash = repo.find_blob(blob_oid).unwrap();

        assert_eq!(git2_hash.id().to_string(), format!("{}", git_rust_hash));
    });
}
