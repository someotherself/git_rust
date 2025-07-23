use chrono::Local;

use crate::{
    git_rust::RepoRust,
    objects::{self, Header},
};

pub struct Commit {
    header: Header,
    tree_hash: String,
    parents_hash: Vec<String>,
    author: Autors,
    committer: Autors,
    message: String,
}

#[derive(Default)]
pub struct Autors {
    name: String,
    email: String,
    timestamp: i64,
    timezone: String,
}

impl Autors {
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let string = str::from_utf8(bytes).ok()?;
        let mut components = string.split(' ');
        let authors = components.next()?;
        if authors != "author" && authors != "committer" {
            return None;
        }

        let name = components.next()?.to_string();

        let email_field = components.next()?;
        let email_start = email_field.find('<')? + 1;
        let email_end = email_field.find('>')?;
        let email = email_field[email_start..email_end].to_string();

        let timestamp: i64 = components.next()?.parse().ok()?;
        let timezone = components.next()?.to_string();

        Some(Autors {
            name,
            email,
            timestamp,
            timezone,
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut contents: Vec<u8> = Vec::new();
        contents.extend_from_slice(self.name.as_bytes());
        contents.push(b' ');
        contents.push(b'<');
        contents.extend_from_slice(self.email.as_bytes());
        contents.push(b'>');
        contents.push(b' ');
        contents.extend_from_slice(self.timestamp.to_string().as_bytes());
        contents.push(b' ');
        contents.extend_from_slice(self.timezone.as_bytes());
        contents
    }
}

impl Commit {
    fn header(&self) -> &Header {
        &self.header
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut contents: Vec<u8> = vec![];
        contents.extend_from_slice("tree ".as_bytes());
        contents.extend_from_slice(self.tree_hash.as_bytes());
        contents.push(b'\n');
        if !self.parents_hash.is_empty() {
            for hash in &self.parents_hash {
                contents.extend_from_slice("parent ".as_bytes());
                contents.extend_from_slice(hash.as_bytes());
                contents.push(b'\n');
            }
        }
        contents.extend_from_slice(&self.author.to_bytes());
        contents.push(b'\n');
        contents.extend_from_slice(&self.committer.to_bytes());
        contents.push(b'\n');
        contents.push(b'\n');
        contents.extend_from_slice(self.message.as_bytes());
        contents
    }

    pub fn encode(
        tree_hash: String,
        commit: Vec<String>,
        message: String,
    ) -> std::io::Result<Self> {
        // Check if the tree is valid
        objects::get_object_path(&tree_hash).ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "Tree object not found")
        })?;

        // Check if the parents are valid
        if !commit.is_empty() {
            for hash in &commit {
                objects::get_object_path(hash).ok_or_else(|| {
                    std::io::Error::new(std::io::ErrorKind::InvalidInput, "Commit object not found")
                })?;
            }
        }

        let now = Local::now();
        let timestamp = now.timestamp();
        let offset = now.offset().utc_minus_local();
        let hours = offset / 3600;
        let minutes = (offset.abs() % 3600) / 60;
        let sign = if offset >= 0 { '-' } else { '+' };
        let timezone = format!("{}{:<02}{:02}", sign, hours.abs(), minutes);

        let git2_repo = git2::Repository::discover(".").expect("Could not find .git repo");
        let config = git2_repo.config().expect("Could not fetch git config");
        let name = config
            .get_string("user.name")
            .expect("Could not fetch git name");
        let email = config
            .get_string("user.email")
            .expect("Could not fetch git email");

        let author = Autors {
            name: name.clone(),
            email: email.clone(),
            timestamp,
            timezone: timezone.clone(),
        };
        let committer = Autors {
            name,
            email,
            timestamp,
            timezone,
        };

        let temp_header = Header {
            object: objects::ObjectType::Commit,
            size: 0,
        };

        let mut commit = Self {
            header: temp_header,
            tree_hash,
            parents_hash: commit,
            author,
            committer,
            message,
        };
        let contents = commit.to_bytes();
        let header = Header {
            object: objects::ObjectType::Commit,
            size: contents.len(),
        };
        commit.header = header;

        Ok(commit)
    }

    // Split by new line. Each line starts with these words (as bytes):
    // tree <40-char SHA>\n
    // parent <40-char SHA>\n (optional)
    // parent <40-char SHA>\n (optional)
    // author ...\n
    // committer ...\n
    // \n
    // <commit message>
    // Used by cat-file
    pub fn decode(hash: &str) -> std::io::Result<Self> {
        let root_path = RepoRust::get_object_folder(&RepoRust::get_root().absolute_path);

        let (folder_name, file_name) = hash.split_at(2);
        let file_path = root_path.join(folder_name).join(file_name);
        let file_content = std::fs::read(file_path)?;
        let bytes_output = objects::de_compress(&file_content)?;
        let header = Header::from_binary(&bytes_output)?;

        let start = header.head_length();
        let contents = bytes_output[start + 1..]
            .split(|b| *b == b'\n')
            .collect::<Vec<&[u8]>>();

        let mut tree_hash = String::from("");
        let mut parents_hash: Vec<String> = Vec::new();
        let mut author = Autors::default();
        let mut committer = Autors::default();
        let mut message = String::from("");

        for line in contents {
            if line.is_empty() {
                continue;
            }
            if line.starts_with(b"tree ") {
                let hash_bytes = &line["tree ".len()..];
                tree_hash = str::from_utf8(hash_bytes).unwrap().to_string();
            } else if line.starts_with(b"parent ") {
                let hash_bytes = &line["parent ".len()..];
                let parent_hash = str::from_utf8(hash_bytes).unwrap().to_string();
                parents_hash.push(parent_hash);
            } else if line.starts_with(b"author ") {
                match Autors::from_bytes(line) {
                    Some(a) => author = a,
                    // Should not be able to create a commit without author
                    None => {
                        return Err(std::io::Error::other("Author field missing"));
                    }
                }
            } else if line.starts_with(b"committer ") {
                match Autors::from_bytes(line) {
                    Some(a) => committer = a,
                    // Should not be able to create a commit without committer
                    None => {
                        return Err(std::io::Error::other("Comitter field missing"));
                    }
                }
            } else {
                message = str::from_utf8(line).unwrap().to_string();
            }
        }

        Ok(Commit {
            header,
            tree_hash,
            parents_hash,
            author,
            committer,
            message,
        })
    }
}

impl std::fmt::Display for Autors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} <{}> {} {}",
            self.name, self.email, self.timestamp, self.timezone
        )
    }
}

impl std::fmt::Display for Commit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "tree {}", self.tree_hash)?;

        for parent in &self.parents_hash {
            writeln!(f, "parent {parent}")?;
        }

        writeln!(f, "author {}", self.author)?;
        writeln!(f, "committer {}", self.committer)?;
        writeln!(f)?;
        writeln!(f, "{}", self.message.trim_end())
    }
}
