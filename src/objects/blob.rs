use std::{fmt::Display, io::Write, path::PathBuf};

use clap::ArgMatches;
use flate2::{Compress, Compression, write::ZlibEncoder};
use hex::ToHex;
use sha1::{Digest, Sha1};

use crate::git_rust::BASE_DIR;
use crate::objects::{GitObject, Header, ObjectType, blob};
use crate::utils::{de_compress, find_file_in_repo, write_object};

pub(crate) struct Blob {
    pub(crate) header: Header,
    pub(crate) hash: String,
    pub(crate) folder: String,
    pub(crate) file: String,
}

impl Blob {
    fn new_from_bytes(bytes: Vec<u8>) -> std::io::Result<Self> {
        let mut pos: usize = 0;
        for (idx, &byte) in bytes.iter().enumerate() {
            if byte == b' ' {
                pos = idx
            }
        }
        let hash = std::str::from_utf8(&&bytes[pos..]).unwrap().to_owned();

        let header = Header::from_binary(bytes)?;
        let (folder, file) = hash.split_at(2).to_owned();

        Ok(Self {
            header,
            hash: hash.to_owned(),
            folder: folder.to_owned(),
            file: file.to_owned(),
        })
    }

    fn blob_with_sha1(file: Vec<u8>) -> std::io::Result<Self> {
        let mut hasher = Sha1::new();
        hasher.update(format!("blob {}\0", file.len()).as_bytes());
        hasher.update(&file);
        let result = hasher.finalize();
        let full_hash = result.encode_hex::<String>();
        let folder = result[..1].to_vec().encode_hex::<String>();
        let file = result[1..].to_vec().encode_hex::<String>();
        let blob = Self {
            header: Header {
                object: ObjectType::Blob,
                size: file.len(),
            },
            hash: full_hash,
            folder,
            file,
        };
        Ok(blob)
    }
}


impl GitObject for Blob {
    const TYPE: ObjectType = ObjectType::Blob;
    type Output = Blob;

    fn header(&self) -> &Header {
        &self.header
    }

    fn write_object_to_file(&self, file: Vec<u8>) -> std::io::Result<()> {
        let folder_path = PathBuf::from(format!(".git/objects/{}", self.folder));
        let file_path = PathBuf::from(format!(".git/objects/{}/{}", self.folder, self.file));
        if !folder_path.exists() {
            std::fs::create_dir(folder_path)?;
        }
        let new_blob = std::fs::File::create(file_path)?;
        let mut enc =
            ZlibEncoder::new_with_compress(new_blob, Compress::new(Compression::best(), true));
        enc.write_all(&file)?;
        Ok(())
    }

    // hash-object command
    // FIX
    fn encode_object(args: &ArgMatches) -> std::io::Result<Blob> {
        let sub_arg = args.get_flag("pretty");
        let object = args
            .get_one::<String>("file")
            .expect("File is required.")
            .to_owned();
        let file = std::fs::read(PathBuf::from(object))?;
        let blob = blob::Blob::blob_with_sha1(file.clone())?;
        if sub_arg {
            write_object(&blob, file)?;
        }
        Ok(blob)
    }

    // cat-file command
    fn decode_object(args: &ArgMatches) -> std::io::Result<Blob> {
        let _sub_arg = args.get_flag("pretty");
        let hash = args
            .get_one::<String>("hash")
            .expect("Hash is required.")
            .to_owned();
        find_file_in_repo();
        let (folder_name, file_name) = hash.split_at(2);
        let file_path = PathBuf::from(format!(
            "{}/objects/{}/{}",
            BASE_DIR, folder_name, file_name
        ));
        let file = std::fs::read(file_path)?;
        let bytes_output = de_compress(file)?;
        let blob = Blob::new_from_bytes(bytes_output)?;
        Ok(blob)
    }
}

impl Display for Blob {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let hash = self.hash.clone();
        write!(f, "{}", hash)
    }
}