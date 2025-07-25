# 🦀 git-rust — A Git CLI clone Written in Rust

git-rust is a simplified Git implementation built from scratch in Rust. It mimics core Git functionality of basic commands, listed below. It serves as a learning project.

This project explores how Git works under the hood — from creating repositories and hashing objects, to writing Git-compliant blob files and reading them back from the .git/objects directory.

# ✨ Current Features and flags implemented

    cargo run init          
                            — Initialize a new Git-like repository. Does not re-initialize a repo.

    cargo run add <file>    
                            - Add files to staging area.
                            - Index file compatible with git

    cargo run hash-object -w <hash>
                            - Hash and write blob objects to .git/objects
                            - flag -w (optional) - works as expected in git

    cargo run cat-file -p <hash>      
                            - Decode and print stored objects
                            - flag -p (optional) - different from git
                                Blobs:      Always prints raw bytes to stdout
                                Trees:      Without pretty pring, will send raw bytes to stdout
                                Commits:    Always pretty print

    cargo run ls-files
                            - Show the files in the index

    cargo run ls-tree <hash>
                            - List the contents of a tree object.

    cargo run write-tree
                            - Reads the index and creates tree objects.

    cargo run commit-tree <hash> -p <hash> -m <message>
                            - Creates a new commit object
                            - flag -p can be used multiple times for parrent commits
                            - flag -m can be used only once

    cargo run commit -a -m <message>
                            - Record changes to the repository
                            - flag -a not yet implemented. Changes need to be staged separately.
                            - flag -m can be used only once.


# 🛠️ Formatting helper 

## Object files (blobs, tree and commits)
All blob, tree and commit files are tested to be compatible with git.
Below is meant to be as documentation. Should be matching git to the best of my research.

All git object files are preceded by a header. They are all compressed with zlib.
A header contains the name of the object, followed by the size of te content (number of bytes) and null terminated.
Example: "blob [size]\0
Example: "tree [size]\0
Example: "commit [size]\0

## A Blob file
The content is simply the contents of the original file.
Contains no metadata.

## A Tree file
The content is list of all the objects under that tree.  
Each entry contains the mode, file name and the hash of that object.  
The mode is terminated with a space and the filename is null terminated.  
The hash is always 20 bytes.  

```text
example object 1    -> mode+b' '+file name+b'\0'+hash [u8; 20]
example object 1    -> 100644+b' '+test.txt+b'\0'+63aa9936a393155f43c2b03d42d79b1c83290f41
example output 1    -> 100644 blob 63aa9936a393155f43c2b03d42d79b1c83290f41 file.txt
```
Mode is specific to each object
| **Object**     | **Mode** |
|---------------|------------------|
|blob|100644                  |
|tree|40000                  |
|commit|160000                  |


## A Commit file
The information in the commit file is separated by new line characters and all but the message start with a specific word.  
Between the committer and message, there is an extra line.  
Lack of a parent SHA indicates it is the initial commit.  
Multiple SHA's indicate it is a merge commit  

```text
tree <40-character SHA>\n
parent <40-character SHA>\n      (optional, can appear multiple times)
parent <40-character SHA>\n      (optional, for merge commits)
author <name> <<email>> <timestamp> <timezone>\n
committer <name> <<email>> <timestamp> <timezone>\n
\n
<commit message>
```

#### Logic of cargo run commit:
```text
1.  run write-tree                  -> get root tree and the hash of the root tree
    if the index does not exist     -> STOP
2.  read the HEAD                   -> get the branch
    if no branch file               -> initial commit. Create the branch file.
3.  read the branch file            -> get the parent commit
4.  read the tree hash from the last commit
5.  evaluate if new tree hash is the same as tree in the last commit
    if the same                     -> STOP
6.  write the tree to file
7.  use commit-tree to create the commit
8.  write commit to file
9.  update the branch file
10. update reflog
11. print summary
```

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
