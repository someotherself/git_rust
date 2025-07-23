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
    date: i64,
    timezone: String,
}

impl Autors {
    pub fn from_butes(bytes: &[u8]) -> Option<Self> {
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

        let date: i64 = components.next()?.parse().ok()?;
        let timezone = components.next()?.to_string();

        Some(Autors {
            name,
            email,
            date,
            timezone,
        })
    }
}

impl Commit {
    fn header(&self) -> &Header {
        &self.header
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
    pub fn encode() {}
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
                match Autors::from_butes(line) {
                    Some(a) => author = a,
                    // Should not be able to create a commit without author
                    None => {
                        return Err(std::io::Error::other("Author field missing"));
                    }
                }
            } else if line.starts_with(b"committer ") {
                match Autors::from_butes(line) {
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
            self.name, self.email, self.date, self.timezone
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
