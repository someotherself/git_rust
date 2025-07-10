mod git_rust;
mod objects;

#[cfg(test)]
mod test_common;

use clap::{Arg, ArgAction, Command, command};
use git_rust::Repo_Rust;

use std::env;

fn main() -> std::io::Result<()> {
    let matches = command!()
        .subcommand(Command::new("init").about("Create an empty git directory"))
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
        .subcommand(
            Command::new("hash-object")
                .about("Compute object ID and optionally create an object from a file")
                .arg(
                    Arg::new("pretty")
                        .short('w')
                        .help("Actually write the object into the object database.")
                        .action(ArgAction::SetTrue),
                )
                .arg(Arg::new("file").required(true).value_name("FILE")),
        )
        .subcommand(
            Command::new("ls-tree")
                .about("List the contents of a tree object")
                .arg(Arg::new("hash").required(true).value_name("HASH")),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("init", _)) => Repo_Rust::init()?,
        Some(("cat-file", args)) => Repo_Rust::cat_file(args)?,
        Some(("hash-object", args)) => Repo_Rust::hash_object(args)?,
        Some(("ls-tree", args)) => Repo_Rust::ls_tree(args)?,
        Some((_, _)) => {}
        None => {}
    }
    Ok(())
}
