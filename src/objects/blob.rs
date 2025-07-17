use std::{
    fmt::Display,
    io::{Read, Write},
    path::PathBuf,
};

use clap::ArgMatches;
use flate2::{Compress, Compression, bufread::ZlibDecoder, write::ZlibEncoder};
use hex::ToHex;
use sha1::{Digest, Sha1};

use crate::{
    git_rust::RepoRust,
    objects::{Header, ObjectType},
};

#[derive(Debug)]
pub struct Blob {
    pub header: Header,
    pub hash: String,
    pub folder: String,
    pub file: String,
}

impl PartialEq for Blob {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash
    }
}

impl Blob {
    fn header(&self) -> &Header {
        &self.header
    }

    pub fn get_folder(&self) -> &str {
        &self.folder
    }

    pub fn get_file(&self) -> &str {
        &self.file
    }

    // cat-file command
    pub fn decode_object(hash: &str) -> std::io::Result<Vec<u8>> {
        let root_path = RepoRust::get_object_folder(&RepoRust::get_root().base_path);
        let (folder_name, file_name) = hash.split_at(2);
        let file_path = root_path.join(folder_name).join(file_name);
        let file = std::fs::read(file_path)?;
        let bytes_output = Self::de_compress(&file)?;

        let null_pos = bytes_output.iter().position(|&b| b == b'\0').unwrap();
        let content_bytes = bytes_output[null_pos + 1..].to_vec();
        // let contents = Blob::new_from_bytes(bytes_output)?;
        Ok(content_bytes)
    }

    // hash-object command
    pub fn encode_object(args: &ArgMatches) -> std::io::Result<Self> {
        // TODO: Check if blob already exists. Add test for it.
        let sub_arg = args.get_flag("write");
        let object = args
            .get_one::<String>("file")
            .expect("File is required.")
            .to_owned();
        let file = std::fs::read(PathBuf::from(object))?;
        let blob = Self::blob_with_sha1(&file);
        if sub_arg {
            blob.write_object_to_file(&file)?;
        }
        Ok(blob)
    }

    pub fn write_object_to_file(&self, file: &[u8]) -> std::io::Result<()> {
        let objects_path = RepoRust::get_object_folder(&RepoRust::get_root().base_path);
        let folder_path = objects_path.join(&self.folder);
        let file_path = folder_path.join(&self.file);
        if !folder_path.exists() {
            std::fs::create_dir(folder_path)?;
        }
        let new_blob = std::fs::File::create(file_path)?;
        let mut enc =
            ZlibEncoder::new_with_compress(new_blob, Compress::new(Compression::best(), true));
        let header = format!("blob {}\0", file.len());
        let full_blob = [header.as_bytes(), file].concat();

        enc.write_all(&full_blob)?;
        Ok(())
    }

    // TODO: Incorrect implementation.
    fn new_from_bytes(bytes: &[u8]) -> std::io::Result<Self> {
        let null_pos: usize = bytes.iter().position(|&b| b == b'\0').unwrap();

        let header_bytes = &bytes[..null_pos];
        let content_bytes = &bytes[null_pos + 1..];

        // TODO This is not the hash, it's the content of the file
        let hash = std::str::from_utf8(content_bytes).unwrap();
        let header = Header::from_binary(header_bytes)?;
        let (folder, file) = hash.split_at(2).to_owned();

        Ok(Self {
            header,
            hash: hash.to_owned(),
            folder: folder.to_owned(),
            file: file.to_owned(),
        })
    }

    pub fn blob_with_sha1(file: &[u8]) -> Self {
        let mut hasher = Sha1::new();
        hasher.update(format!("blob {}\0", file.len()).as_bytes());
        hasher.update(file);
        let result = hasher.finalize();
        let full_hash = result.encode_hex::<String>();
        let folder = result[..1].to_vec().encode_hex::<String>();
        let file = result[1..].to_vec().encode_hex::<String>();
        Self {
            header: Header {
                object: ObjectType::Blob,
                size: file.len(),
            },
            hash: full_hash,
            folder,
            file,
        }
    }

    pub fn de_compress(content: &[u8]) -> std::io::Result<Vec<u8>> {
        let mut decompressed = ZlibDecoder::new(content);
        let mut buffer = Vec::new();
        decompressed.read_to_end(&mut buffer)?;
        Ok(buffer)
    }

    pub fn blob_exists(hash: [u8; 20]) -> bool {
        let root = &RepoRust::get_root().base_path;
        let obj_path = RepoRust::get_object_folder(root);
        let hex_hash = hex::encode(hash);
        let (folder_name, file_name) = hex_hash.split_at(2);
        obj_path.join(folder_name).join(file_name).exists()
    }
}

impl Display for Blob {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let hash = self.hash.clone();
        write!(f, "{hash}")
    }
}
