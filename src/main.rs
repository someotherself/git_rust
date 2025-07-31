mod git_rust;
mod index;
mod objects;
mod requests;

#[cfg(test)]
mod test_common;

use clap::{Arg, ArgAction, Command, command};
use git_rust::RepoRust;
use tracing_subscriber::EnvFilter;

use std::env;
fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env().add_directive("git_rust=info".parse().unwrap()),
        )
        .init();
    tracing::info!("Starting git-rust CLI");
    let matches = command!()
        // git init
        .subcommand(Command::new("init").about("Create an empty git directory"))
        // git cat-file
        .subcommand(
            Command::new("cat-file").about("Provide contents or details of repository objects")
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
        .subcommand(
            Command::new("commit-tree")
                .about("Create a new commit object")
                .arg(
                    Arg::new("hash")
                        .required(true)
                        .value_name("HASH")
                        .help("The tree object hash to commit"),
                )
                .arg(
                    Arg::new("commit")
                        .short('p')
                        .value_name("COMMIT")
                        .num_args(1)
                        .action(clap::ArgAction::Append)
                        .help("Optional parent commit hash (for non-root commits)"),
                )
                .arg(
                    Arg::new("message")
                        .short('m')
                        .value_name("MESSAGE")
                        .default_value("")
                        .help("Commit message (if not provided, reads from stdin)"),
                ),
        )
        // commit
        .subcommand(
            Command::new("commit").about("Record changes to the repository")
            .arg(
                Arg::new("add")
                .short('a')
                .action(ArgAction::SetTrue)
                .help("Automatically stage files that have been modified and deleted, but new files you have not told Git about are not affected."))
            .arg(Arg::new("message")
                .short('m')
                .value_name("MESSAGE")
                .help("Add a commit message.")))
        .subcommand(
            Command::new("clone")
                .about("")
                .arg(
                    Arg::new("url")
                        .required(true)
                        .value_name("URL")
                        .help("The URL for the reposity."),
                )
                .arg(
                    Arg::new("directory")
                        .required(true)
                        .value_name("DIR")
                        .help("The local directory you wish the clone into."),
                ),
        )
                .subcommand(
            Command::new("fetch")
                .about("Download objects and refs from a repository")
                .arg(
                    Arg::new("url")
                        .required(true)
                        .value_name("URL")
                        .help("The URL for the reposity."),
                )
                .arg(
                    Arg::new("branch")
                        .required(true)
                        .value_name("BRANCH")
                        .help("The branch you wish to fetch from."),
                )
                .arg(
                    Arg::new("directory")
                        .required(true)
                        .value_name("DIR")
                        .help("The local directory you wish the clone into."),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("init", _)) => RepoRust::init()?,
        Some(("cat-file", args)) => {
            RepoRust::cat_file(args)?;
        }
        Some(("hash-object", args)) => RepoRust::hash_object(args)?,
        Some(("ls-tree", args)) => RepoRust::ls_tree(args)?,
        Some(("add", args)) => RepoRust::add(args)?,
        Some(("ls-files", args)) => RepoRust::ls_files(args)?,
        Some(("write-tree", args)) => RepoRust::write_tree(args)?,
        Some(("commit-tree", args)) => RepoRust::commit_tree(args)?,
        Some(("commit", args)) => RepoRust::commit(args)?,
        Some(("fetch", args)) => RepoRust::fetch(args)?,
        Some(("clone", args)) => RepoRust::clone(args)?,
        Some((_, _)) | None => {}
    }
    tracing::info!("Shutting down git-rust");
    Ok(())
}
