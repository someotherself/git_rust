🦀 git-rust — A Git CLI clone Written in Rust

git-rust is a simplified Git implementation built from scratch in Rust. It mimics core Git functionality such as init, hash-object, and cat-file, and serves both as a learning project and a functional Git-like CLI.

This project explores how Git works under the hood — from creating repositories and hashing objects, to writing Git-compliant blob files and reading them back from the .git/objects directory.
✨ Current Features

    git-rust init — Initialize a new Git-like repository

    git-rust hash-object -w <file> — Hash and write blob objects to .git/objects

    git-rust cat-file -p <hash> — Decode and print stored objects

    Fully compatible object format (blob <size>\0<content>) using zlib compression

    Built with safety, performance, and testability in mind using idiomatic Rust

🛠️ Why This Project?

This project is ideal if you want to:

    Understand Git internals by building them yourself

    Learn about SHA-1 hashing, zlib compression, and Git’s object model

    Practice building command-line tools and file-based data structures in Rust