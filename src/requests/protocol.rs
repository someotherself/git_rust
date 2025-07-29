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

// Response will probably start with `0008NAK` for the initial request
// `The 0008NAK is part of the Git protocol's negotiation phase.
// It indicates that the server did not find any common commits between
// the client and the server for the requested references.
// This often happens during an initial clone or when the client doesn't have any objects yet.`
#[allow(dead_code)]
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
        .header(reqwest::header::CACHE_CONTROL, "no-cache")
        .body(payload)
        .send()?;

    let body = res.bytes()?.to_vec();

    Ok(body)
}
