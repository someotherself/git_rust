use reqwest::blocking::Client;
use std::io::Read;

pub fn get_request(url: &str) -> Result<Vec<u8>, reqwest::Error> {
    let url = format!("{url}/info/refs?service=git-upload-pack");
    let client = Client::new();

    let mut res = client.get(url).header("User-Agent", "git/2.0").send()?;

    let mut body = Vec::new();
    res.read_to_end(&mut body)
        .expect("Could not write data received from git.");

    Ok(body)
}
//
pub fn post_request(url: &str, payload: Vec<u8>) -> Result<Vec<u8>, reqwest::Error> {
    let url = format!("{url}.git/git-upload-pack");
    dbg!(&url);
    let content = "application/x-git-upload-pack-request";
    let client = Client::new();

    let res = client
        .post(&url)
        .header(reqwest::header::CONTENT_TYPE, content)
        .header(
            reqwest::header::ACCEPT,
            "application/x-git-upload-pack-result",
        )
        // .header(reqwest::header::CACHE_CONTROL, "no-cache")
        .body(payload)
        .send()?;

    dbg!(res.status());

    let body = res.bytes()?.to_vec();

    dbg!(body.len());
    todo!()
}

pub fn fetch(url: &str, _dir: &str) -> Result<(), reqwest::Error> {
    let payload = get_request(url)?;
    let content = read_pkt_lines(&payload);
    for line in &content[0..10] {
        let text = String::from_utf8(line.to_vec()).unwrap();
        dbg!(text);
    }
    let head = &content[1];
    let (hash, _head) = fetch_head(head);
    let want_payload = write_pkt_lines(hash);
    post_request(url, want_payload).unwrap();

    Ok(())
}

fn fetch_head(line: &[u8]) -> (&str, &str) {
    let line_str = str::from_utf8(line).unwrap();
    let mut head_pos = line_str.splitn(2, "symref=HEAD:");

    let hash = &head_pos.next().unwrap()[..40];
    let head_ref = head_pos.next().unwrap();
    let head_ref_finish = head_ref.find(' ').unwrap();
    let head_ref = &head_ref[..head_ref_finish];
    (hash, head_ref)
}

// Parsing the Pkt-Line Format. Example:
// 001e# service=git-upload-pack\n
// 0000 -> Called a flush packet. Must be skipped
// LLLL<line1\n> -> LLLL = length of data
// LLLL<line2\n>
// LLLL<line3\n>
// LLLL<line4\n>
// 0000
//
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
        //         ↑                         ↑
        //  i + 4 ─┘                         └─ i + 0x001e (length in hex)
        let payload = data[i + 4..i + len].to_vec();
        lines.push(payload);
        i += len;
    }
    lines
}

// Example of formatting for the pakt payload
// 003fwant <hash1> multi_ack thin-pack side-band side-band-64k ofs-delta\0
// 0000
// 003fwant <hash2>
// 0000
// 0009done\n
// 0000
fn write_pkt_lines(hash: &str) -> Vec<u8> {
    let mut payload = Vec::new();
    let capabilities = "multi_ack thin-pack side-band side-band-64k ofs-delta";

    let want = format!("want {hash} {capabilities}\n");
    dbg!(&want);
    let want_len = 4 + want.len();
    dbg!(&want_len);
    payload.extend_from_slice(format!("{want_len:04x}").as_bytes());
    payload.extend_from_slice(want.as_bytes());
    payload.extend_from_slice(b"0000");

    let done = "done\n";
    let done_len = done.len() + 4;
    payload.extend_from_slice(format!("{done_len:04x}").as_bytes());
    payload.extend_from_slice(done.as_bytes());

    payload.extend_from_slice(b"0000");

    payload
}
