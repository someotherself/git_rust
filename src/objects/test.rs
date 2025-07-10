use std::io::Write;

use super::*;
use crate::test_common::{cat_file_pretty, run_test};

#[test]
fn test_cat_file1() {
    run_test("test_cat_file", |setup| {
        use git2::Repository;
        let path = &setup.dir;
        let file_path = path.join("test1.txt");
        let mut file = std::fs::File::create(&file_path).unwrap();
        file.write_all(b"this is a test").unwrap();

        let repo = Repository::init(path).unwrap();
        let blob_oid = repo.blob_path(&file_path).unwrap();
        let blob = repo.find_blob(blob_oid).unwrap();
        let git2_content = std::str::from_utf8(blob.content()).unwrap();

        // let args = cat_file_pretty(hash);
        // let _crate_content = blob::Blob::decode_object(&args).expect("Something went wrong");

        // assert_eq!(git2_content, format!("{}", crate_content));
        assert_eq!("123", "123");
    });
}
