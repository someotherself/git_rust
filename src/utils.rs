use std::{
    io::{Error, Read}, path::PathBuf, sync::Arc
};

use flate2::bufread::ZlibDecoder;

use crate::{
    git_rust::{Repo_Rust, BASE_DIR, REPO_ROOT, REPO},
    objects::GitObject,
};

pub fn de_compress(content: Vec<u8>) -> std::io::Result<Vec<u8>> {
    let mut buffer = vec![0; 1024];
    let mut decompressed = ZlibDecoder::new(&content[..]);
    decompressed.read(&mut buffer)?;
    Ok(buffer)
}

pub fn write_object<O: GitObject>(object: &O, file: Vec<u8>) -> std::io::Result<()> {
    // let folder_path = PathBuf::from(format!(".git/objects/{}", object.folder));
    // let file_path = PathBuf::from(format!(".git/objects/{}/{}", object.folder, object.file));
    // if !folder_path.exists() {
    //     std::fs::create_dir(folder_path)?;
    // }
    // let new_blob = std::fs::File::create(file_path)?;
    // let mut enc =
    //     ZlibEncoder::new_with_compress(new_blob, Compress::new(Compression::best(), true));
    // enc.write_all(&file)?;
    Ok(())
}

pub fn find_file_in_repo() -> Option<PathBuf> {
    if REPO_ROOT.get().is_some() {
        return REPO_ROOT.get().cloned();
    } else {
        let cwd = std::env::current_dir().unwrap();
        let mut abs_path: PathBuf = PathBuf::new();
        if cwd.join(BASE_DIR).exists() {
            REPO_ROOT.set(cwd.join(BASE_DIR)).unwrap();
            return Some(cwd.join(BASE_DIR));
        } else {
            for comp in cwd.as_path().components() {
                abs_path.push(comp);
                for dir in std::fs::read_dir(&abs_path).unwrap() {
                    let dir = dir.unwrap();
                    if dir.metadata().unwrap().is_dir() == true && dir.file_name() == BASE_DIR {
                        REPO_ROOT.set(dir.path()).unwrap();
                        return Some(dir.path());
                    }
                }
            }
        }
    }
    None
}

pub fn get_root() -> std::io::Result<Arc<Repo_Rust>> {
    let dir = find_root()?;
    let repo = REPO.get_or_init(|| Arc::new(Repo_Rust {base_path: dir}));
    Ok(repo.clone())
}

fn find_root() -> std::io::Result<PathBuf> {
        let mut dir = std::env::current_dir().map_err(|_| Error::new(std::io::ErrorKind::Other, "Failed to read filesystem"))?;
    loop {
        if dir.join(BASE_DIR).is_dir() {
            return Ok(dir);
        }
        if !dir.pop() {
            return Err(Error::new(std::io::ErrorKind::NotFound, "No repo found!"));
        }
    }
}
