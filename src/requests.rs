pub mod fetch;
mod protocol;

#[derive(Debug)]
#[allow(dead_code)]
pub struct UploadPack {
    pub head: Option<GitRef>,
    pub refs: Vec<GitRef>,
    pub tags: Vec<GitRef>,
    // Server specific (PR commits)
    pub pulls: Vec<GitRef>,
    // Optional
    pub symrefs: Vec<GitRef>,
    pub capabilities: Vec<String>,
}

#[allow(unused_mut)]
impl UploadPack {
    fn from_response(res: Vec<Vec<u8>>) -> Self {
        let mut head: Option<GitRef> = None;
        let mut refs: Vec<GitRef> = Vec::new();
        let mut tags: Vec<GitRef> = Vec::new();
        let mut pulls: Vec<GitRef> = Vec::new();
        let mut symrefs: Vec<GitRef> = Vec::new();
        let mut capabilities: Vec<String> = Vec::new();
        for line in res {
            let line = String::from_utf8(line).unwrap();
            let comps = line.splitn(2, ' ').collect::<Vec<&str>>();
            if comps.len() == 1 {
                dbg!("Invalid upload-pack response line");
                continue;
            }
            match comps[1] {
                s if s.starts_with("#") => {
                    continue;
                }
                s if s.starts_with("HEAD") => {
                    head = Some(GitRef::read_head(s));
                    capabilities = GitRef::read_capabilities(s);
                }
                s if s.starts_with("refs/heads") => {
                    refs = GitRef::read_refs(s);
                }
                s if s.starts_with("refs/tags") => {
                    tags = GitRef::read_refs(s);
                }
                s if s.starts_with("refs/pull") => {
                    pulls = GitRef::read_refs(s);
                }
                // Optional
                // Looks at the HEAD line, for symrefs that do not start with "symref=HEAD:"
                s if s.starts_with("refs/symref") => {}
                _ => {}
            }
        }
        Self {
            head,
            refs,
            tags,
            pulls,
            symrefs,
            capabilities,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct GitRef {
    pub name: String,
    pub hash: String,
}

// Example HEAD response:
// <SHA1> HEAD\0multi_ack thin-pack side-band side-band-64k ofs-delta shallow deepen-since deepen-not deepen-relative no-progress
// include-tag multi_ack_detailed allow-tip-sha1-in-want allow-reachable-sha1-in-want no-done symref=HEAD:refs/heads/main filter
// object-format=sha1 agent=git/github-5a2d4c40a633-Linux\n
impl GitRef {
    // Used for all refs, tags, pulls etc
    fn read_refs(res: &str) -> Vec<GitRef> {
        let mut refs = Vec::new();

        for line in res.lines() {
            // Each line is: "<hash> <refname>"
            let components: Vec<&str> = line.splitn(2, ' ').collect();
            if components.len() != 2 {
                continue; // skip invalid lines
            }

            let hash = components[0].to_string();
            let name = components[1].trim().to_string();

            // Ignore symref and capabilities lines here
            if name.starts_with("symref=") || name.starts_with('#') {
                continue;
            }

            refs.push(GitRef { name, hash });
        }

        refs
    }

    fn read_head(res: &str) -> Self {
        let components = res.splitn(2, " ").collect::<Vec<_>>();
        let hash = components[0].to_string();
        let comps = components[1]
            .split(' ')
            .filter(|x| x.starts_with("symref=HEAD:"))
            .collect::<Vec<&str>>()[0];
        dbg!(comps);
        let name = comps.strip_prefix("symref=HEAD:").unwrap().to_string();
        dbg!(&hash);
        dbg!(&name);
        Self { name, hash }
    }

    fn read_capabilities(res: &str) -> Vec<String> {
        let mut capabilities = Vec::new();

        let mut parts = res.splitn(2, '\0');
        parts.next();
        let caps_str = parts.next().expect("Invalid capabilities response");

        for cap in caps_str.split(' ') {
            if cap.is_empty() {
                continue;
            }
            if cap.starts_with("symref") {
                continue;
            }
            capabilities.push(cap.trim_end_matches('\n').to_string());
        }
        capabilities
    }
}
