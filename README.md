# 🦀 git-rust — A Git CLI clone Written in Rust

git-rust is a simplified Git implementation built from scratch in Rust. It mimics core Git functionality such as init, hash-object, and cat-file, and serves both as a learning project and a functional Git-like CLI.

This project explores how Git works under the hood — from creating repositories and hashing objects, to writing Git-compliant blob files and reading them back from the .git/objects directory.

# ✨ Current Features

    git-rust init — Initialize a new Git-like repository. Does not re-initialize a repo.

    git-rust hash-object -w <file> — Hash and write blob objects to .git/objects

    git-rust cat-file -p <hash> — Decode and print stored objects

    Fully compatible object format (blob <size>\0<content>) using zlib compression

    Built with safety, performance, and testability in mind using idiomatic Rust

# 🛠️ Why This Project?

This project is ideal if you want to:

    Understand Git internals by building them yourself

    Learn about SHA-1 hashing, zlib compression, and Git’s object model

    Practice building command-line tools and file-based data structures in Rust




# 🛠️ Formatting helper 

## A Blob file
Todo

## A Tree file
Todo

## The INDEX file (staging area)
It uses a binary layour (raw bytes) and always big-endian format.
The index has a header that is 12 bytes, a record of entries (files/blobs) added to the staging area and a checksum (SHA-1) of all the content (header + entries).
The entries are not a fixed length, but a multiple of 8 bytes:
- 62 bytes of fixed-size metadata (ctime, mtime, mode, sha1, etc.).
- A null terminated, variable-length path (the filename or relative path), which follows the metadata. 
- 1–8 bytes of padding to ensure the total size of the entry is a multiple of 8 bytes.


Example of the Header:
| **Field**     | **Size (Bytes)** | **Description**                                       | **Example (Hex)**                    |
| ------------- | ---------------- | ----------------------------------------------------- | ------------------------------------ |
| `dircache DIRC`       | 4                | Always set as "DIRC"                     | `44 49 52 43`                        |
| `Version` | 4                | Version 2, 3 or 4. 2 is most common                            | `00 00 00 00`                        |
| `Entries`       | 4                | Number of entries                    | `00 00 00 20`                        |




Example of an entry:
| **Field**     | **Size (Bytes)** | **Description**                                       | **Example (Hex)**                    |
| ------------- | ---------------- | ----------------------------------------------------- | ------------------------------------ |
| `ctime`       | 4                | Created time (seconds)                     | `5E 2D 5A 80`                        |
| `ctime_nanos` | 4                | Created time (nanoseconds)                            | `00 00 00 00`                        |
| `mtime`       | 4                | Modified time (seconds)                    | `5E 2D 5A 90`                        |
| `mtime_nanos` | 4                | Modified time (nanoseconds)                           | `00 00 00 00`                        |
| `dev`         | 4                | Device ID (from `stat(2)`)                            | `00 00 00 15`                        |
| `ino`         | 4                | Inode number                                          | `00 00 00 01`                        |
| `mode`        | 4                | File mode (includes type and permissions)             | `00 00 81 A4`                        |
| `uid`         | 4                | User ID                                               | `00 00 03 E8`                        |
| `gid`         | 4                | Group ID                                              | `00 00 03 E8`                        |
| `file_size`   | 4                | File size in bytes                                    | `00 00 00 04`                        |
| `sha1`        | 20               | SHA-1 hash of the file contents                       | `12 34 ... EF`                       |
| `flags`       | 2                | Bitfield with name length, stage, and flags           | `00 0A`                              |
| `path`        | N (variable)     | File path (UTF-8 bytes, not null-terminated)          | `"main.rs"` = `6D 61 69 6E 2E 72 73` |
| `padding`     | 0–7              | Null bytes to align total entry size to multiple of 8 | `00 00 00` (example)                 |
