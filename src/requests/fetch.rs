use std::io::{Cursor, Read};

use byteorder::{BigEndian, ReadBytesExt};
use flate2::read::ZlibDecoder;

use crate::requests::{
    GitObject, GitRef, UploadPack,
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
    let packfile_bytes = extract_packfile(&object_payload);

    let objects = unpack_packfile(&packfile_bytes).unwrap();
    dbg!(objects.len());
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

// Reads the upload-pack stream
// The fetch response is sent in side-band format (RFC 8484)
// Response is in this format:
// 0032 01 <packfile-bytes...>
// 000a 02 Counting objects...
// 0000
// <size> <identifier> <data>
// Idenfitied: 1 - packfile data / 2 - progress messages / 3 - error
// Extract the raw packfile -
fn extract_packfile(mut data: &[u8]) -> Vec<u8> {
    let mut packfile = Vec::new();

    while !data.is_empty() {
        if data.len() < 4 {
            break;
        }

        // Get the 4 bytes containing the size
        let len_str = str::from_utf8(&data[..4]).unwrap();
        let len = usize::from_str_radix(len_str, 16).unwrap();
        if len == 0 {
            break;
        }

        // Get the 1 byte containing the identifier (band)
        let band = data[4];
        let payload = &data[5..len];

        if band == 1 {
            packfile.extend_from_slice(payload);
        }

        data = &data[len..];
    }

    packfile
}

// Process the raw packfile
// The format of the raw packfile:
// 4 bytes - the word "PACK"
// 4 bytes - the version. Usually 0002
// 4 bytes - number of objects (big endian)
// the rest - Object entries
// last 20 bytes - SHA-1 checksum
//
// Format of object entries:
// Header + Zlib compressed data
// Header:
// 7 6 5 4 3 2 1 0
// C T T T S S S S
// bites 0-3 - Size bits (S)
// bites 4-6 - Object type (T)
// bit   7   - Continuation bit (C)
// Object types: 1 = commit / 2- tree / 3 - blob / 4 - tag / 6 - ofs-delta / 7 ref-delta
//
// Example 1 - 0b01100010
// bites 0-3 -> 0100 (2 in decimal) -> check the continuation bit
// bites 4-6 -> 011 -> (3 in decimal) Blob
// bit   7   -> 0 -> stop reading header
// When continuation bit 1 you, read another 7 bits of the size from the next byte
// Example 2 - 0b111110100
// Next byte :
// bit   7   - another continuation bit
// bites 0-6 - next 7 bits of the size value
// Calculating the size. Example:
// Byte 1: 0b10010011 -> size bits 0b0011 (3 in decimal)
// Byte 2: 0b10000101 -> size bits 0b0000101 (5 in decimal)
// Byte 3: 0b00000010 -> size bits 0b0000010 (2 in decimal)
// Shifting:
// 17 16 15 14 13 12 11 10 09 08 07 06 05 04 03 02 01 00
//  0  0  0  0  0  1  0  0  0  0  0  1  0  1  0  0  1  1 -> 4179 in decimal
// |Bytes 3              |Byte 2               |Byte 1  |
//    (0b0000010 << 11)  +   (0000101 << 4)    +  0011
fn unpack_packfile(packfile: &[u8]) -> std::io::Result<Vec<GitObject>> {
    let mut cursor = Cursor::new(packfile);

    let mut pack = [0u8; 4];
    cursor.read_exact(&mut pack)?;

    if &pack != b"PACK" {
        return Err(std::io::Error::other("Invalid packfile header"));
    }

    let version = cursor.read_u32::<BigEndian>()?;
    if version != 2 && version != 3 {
        return Err(std::io::Error::other("Invalid packfile version"));
    }

    let object_count = cursor.read_u32::<BigEndian>()?;
    dbg!(object_count);
    let mut objects = Vec::new();

    for _ in 0..object_count {
        let (object_type, size) = read_type_and_size(&mut cursor)?;

        match object_type {
            6 => {
                // If ofs-delta
                let mut offset = 0usize;
                loop {
                    let byte = cursor.read_u8()?;
                    offset = (offset << 7) | ((byte & 0x7F) as usize);
                    if byte & 0x80 == 0 {
                        break;
                    }
                }
            }
            // If ref-delta
            7 => {
                let mut base_id = [0u8; 20];
                cursor.read_exact(&mut base_id)?;
            }
            _ => {}
        };

        let start_offset = cursor.position();

        let remaining = &packfile[start_offset as usize..];
        let mut decoder = ZlibDecoder::new(remaining);

        let mut data = Vec::new();
        decoder.read_to_end(&mut data).unwrap();

        let consumed = decoder.total_in();

        cursor.set_position(start_offset + consumed);

        objects.push(GitObject {
            object_type,
            size,
            data,
        });
    }
    Ok(objects)
}

fn read_type_and_size(cursor: &mut Cursor<&[u8]>) -> std::io::Result<(u8, usize)> {
    let mut size: usize;
    let mut shift = 4;
    let mut first_byte = [0u8; 1];

    cursor.read_exact(&mut first_byte)?;
    let byte = first_byte[0];
    let object_type = (byte >> 4) & 0b111;
    size = (byte & 0x0F) as usize;

    // 0x80 = 10000000
    if byte & 0x80 != 0 {
        loop {
            // Read the continuation byte again
            let mut b = [0u8; 1];
            cursor.read_exact(&mut b)?;

            // 0x7F = 01111111
            size |= ((b[0] & 0x7F) as usize) << shift;
            shift += 7;
            if b[0] & 0x80 == 0 {
                break;
            }
        }
    }
    Ok((object_type, size))
}
