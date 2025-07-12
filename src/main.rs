mod git_rust;
mod index;
mod objects;

#[cfg(test)]
mod test_common;

use clap::{Arg, ArgAction, Command, command};
use git_rust::RepoRust;

use std::env;
fn main() -> std::io::Result<()> {
    let matches = command!()
        // git init
        .subcommand(Command::new("init").about("Create an empty git directory"))
        // git cat-file
        .subcommand(
            Command::new("cat-file")
                .arg(
                    Arg::new("pretty")
                        .short('p')
                        .help("Pretty print the object's contents")
                        .action(ArgAction::SetTrue),
                )
                .arg(Arg::new("hash").required(true).value_name("HASH")),
        )
        // git hash-object
        .subcommand(
            Command::new("hash-object")
                .about("Compute object ID and optionally create an object from a file")
                .arg(
                    Arg::new("write")
                        .short('w')
                        .help("Actually write the object into the object database.")
                        .action(ArgAction::SetTrue),
                )
                .arg(Arg::new("file").required(true).value_name("FILE")),
        )
        // git ls-tree
        .subcommand(
            Command::new("ls-tree")
                .about("List the contents of a tree object")
                .arg(Arg::new("hash").required(true).value_name("HASH")),
        )
        // git add
        .subcommand(
            Command::new("add")
                .about("Update the index using the content found in the working tree.")
                .arg(Arg::new("path").required(true).value_name("path")),
        )
        // git ls-files
        .subcommand(
            Command::new("ls-files")
                .about("Show information about files in the index and the working tree"),
        )
        // git write-tree
        .subcommand(Command::new("write-tree").about("Create a tree object from the current index"))
        .get_matches();

    match matches.subcommand() {
        Some(("init", _)) => RepoRust::init()?,
        Some(("cat-file", args)) => RepoRust::cat_file(args)?,
        Some(("hash-object", args)) => RepoRust::hash_object(args)?,
        Some(("ls-tree", args)) => RepoRust::ls_tree(args)?,
        Some(("add", args)) => RepoRust::add(args)?,
        Some(("ls-files", args)) => RepoRust::ls_files(args)?,
        Some(("write-tree", args)) => RepoRust::write_tree(args)?,
        Some((_, _)) => {}
        None => {}
    }
    Ok(())
}
