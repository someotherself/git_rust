use crate::requests::{
    GitRef, UploadPack,
    protocol::{get_request, post_request},
};

// Process of making a fetch request
// 1. GET /info/refs?service=git-upload-pack
//      -> Receive the Git reference advertisement with all the refs and and capabilities
// 2. Parse the refs and capabilities
// 3. Create a POST payload with want and do
// 4. POST the payload to /git-upload-pack (The actual fetch request)
//      -> Receive the commit and objects
// 5. Decode the response
// 6. Extract and process .pack file contents (objects, commits, etc.)
pub fn fetch(url: &str, _dir: &str) -> std::io::Result<()> {
    let payload = get_request(url)
        .map_err(|_| std::io::Error::other("Error fetching the git-upload-pack"))?;
    let content = read_pkt_lines(&payload);
    // for line in &content[..5] {
    //     let text = String::from_utf8(line.to_vec()).unwrap();
    //     dbg!(text);
    // }

    let uploadpack = UploadPack::from_response(content);

    if uploadpack.head.is_none() {
        return Err(std::io::Error::other("missing HEAD"));
    }
    let want_commits = vec![uploadpack.head.unwrap()];

    let want_payload = write_pkt_lines(want_commits);
    let object_payload = post_request(url, want_payload).unwrap();
    let _packfile_bytes = extract_packfile(&object_payload);
    Ok(())
}

// Parsing the Pkt-Line Format. Example:
// 001e# service=git-upload-pack\n
// 0000 -> Called a flush packet. Must be skipped
// LLLL<line1\n> -> LLLL = length of data
// LLLL<line2\n>
// LLLL<line3\n>
// LLLL<line4\n>
// 0000
// The length of the data includes the 4 bytes that hold the size
// Example: "001e# service=git-upload-pack\n".len() = 30 / 001e = 30
fn read_pkt_lines(data: &[u8]) -> Vec<Vec<u8>> {
    let mut i = 0;
    let mut lines = Vec::new();

    // Skip reading the 4 bytes containing the length
    while i + 4 <= data.len() {
        let data_length = std::str::from_utf8(&data[i..i + 4]).unwrap();
        // Check for flush packets
        if data_length == "0000" {
            i += 4;
            continue;
        }

        // Converts the data_length(as str) to a usize
        let len = usize::from_str_radix(data_length, 16).unwrap();
        if len < 4 || i + len > data.len() {
            break;
        }

        //      001e# service=git-upload-pack\n
        //         ↑                          ↑
        //  i + 4 ─┘                          └─ i + 0x001e (length in hex)
        let payload = data[i + 4..i + len].to_vec();
        lines.push(payload);
        i += len;
    }
    lines
}

// Example of formatting for the pakt payload
// 003fwant <hash1> multi_ack thin-pack side-band side-band-64k ofs-delta\0
// 003fwant <hash2>
// 0000
// 0009done\n
// 0000
#[allow(dead_code)]
fn write_pkt_lines(commits: Vec<GitRef>) -> Vec<u8> {
    let mut payload = Vec::new();
    let capabilities = "multi_ack thin-pack no-progress side-band side-band-64k ofs-delta";

    for (i, commit) in commits.iter().enumerate() {
        let want = if i == 0 {
            format!("want {} {capabilities}\n", commit.hash)
        } else {
            format!("want {}\n", commit.hash)
        };

        let want_len = 4 + want.len();
        payload.extend_from_slice(format!("{want_len:04x}").as_bytes());
        payload.extend_from_slice(want.as_bytes());
    }

    payload.extend_from_slice(b"0000");

    let done = "done\n";
    let done_len = done.len() + 4;
    payload.extend_from_slice(format!("{done_len:04x}").as_bytes());
    payload.extend_from_slice(done.as_bytes());

    payload.extend_from_slice(b"0000");

    payload
}

fn extract_packfile(mut data: &[u8]) -> Vec<u8> {
    let mut packfile = Vec::new();

    while !data.is_empty() {
        if data.len() < 4 {
            break;
        }

        let len_str = str::from_utf8(&data[..4]).unwrap();
        let len = usize::from_str_radix(len_str, 16).unwrap();
        if len == 0 {
            break;
        }

        let band = data[4];
        let payload = &data[5..len];

        if band == 1 {
            packfile.extend_from_slice(payload);
        }

        data = &data[len..];
    }

    packfile
}
