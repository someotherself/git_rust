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
    objects::{GitObject, Header, ObjectType, blob},
};

pub(crate) struct Blob {
    pub(crate) header: Header,
    pub(crate) hash: String,
    pub(crate) folder: String,
    pub(crate) file: String,
}

impl Blob {
    pub fn get_folder(&self) -> &str {
        &self.folder
    }

    pub fn get_file(&self) -> &str {
        &self.file
    }

    fn new_from_bytes(bytes: Vec<u8>) -> std::io::Result<Self> {
        let null_pos = bytes.iter().position(|&b| b == b'\0').unwrap();

        let header_bytes = &bytes[..null_pos];
        let content_bytes = &bytes[null_pos + 1..];

        let hash = std::str::from_utf8(content_bytes).unwrap().to_owned();
        let header = Header::from_binary(header_bytes.to_vec())?;
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

    pub fn de_compress(content: Vec<u8>) -> std::io::Result<Vec<u8>> {
        let mut decompressed = ZlibDecoder::new(&content[..]);
        let mut buffer = Vec::new();
        decompressed.read_to_end(&mut buffer)?;
        Ok(buffer)
    }
}

impl GitObject for Blob {
    const TYPE: ObjectType = ObjectType::Blob;
    type Output = Blob;

    fn header(&self) -> &Header {
        &self.header
    }

    fn write_object_to_file(&self, file: Vec<u8>) -> std::io::Result<()> {
        let root_path = RepoRust::get_object_folder(RepoRust::get_root()?.base_path.clone())?;
        let folder_path = root_path.join(&self.folder);
        let file_path = folder_path.join(&self.file);
        if !folder_path.exists() {
            std::fs::create_dir(folder_path)?;
        }
        let new_blob = std::fs::File::create(file_path)?;
        let mut enc =
            ZlibEncoder::new_with_compress(new_blob, Compress::new(Compression::best(), true));
        let header = format!("blob {}\0", file.len());
        let full_blob = [header.as_bytes(), &file[..]].concat();

        enc.write_all(&full_blob)?;
        Ok(())
    }

    // hash-object command
    fn encode_object(args: &ArgMatches) -> std::io::Result<Blob> {
        let sub_arg = args.get_flag("write");
        let object = args
            .get_one::<String>("file")
            .expect("File is required.")
            .to_owned();
        let file = std::fs::read(PathBuf::from(object))?;
        let blob = blob::Blob::blob_with_sha1(file.clone())?;
        if sub_arg {
            blob.write_object_to_file(file)?;
        }
        Ok(blob)
    }

    // cat-file command
    fn decode_object(args: &ArgMatches) -> std::io::Result<Blob> {
        let root_path = RepoRust::get_object_folder(RepoRust::get_root()?.base_path.clone())?;
        let _sub_arg = args.get_flag("pretty");
        let hash = args
            .get_one::<String>("hash")
            .expect("Hash is required.")
            .to_owned();

        let (folder_name, file_name) = hash.split_at(2);
        let file_path = root_path.join(folder_name).join(file_name);
        let file = std::fs::read(file_path)?;
        let bytes_output = Blob::de_compress(file)?;
        let blob = Blob::new_from_bytes(bytes_output)?;
        Ok(blob)
    }
}

impl Display for Blob {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let hash = self.hash.clone();
        write!(f, "{hash}")
    }
}
