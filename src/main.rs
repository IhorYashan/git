use git_starter_rust::git;
use std::env;
use std::fs;

// CLI

fn main() {
    let args: Vec<String> = env::args().collect();

    if args[1] == "init" {
        git::do_git_init();
    }

    if args[1] == "cat-file" && args[2] == "-p" {
        if let Err(err) = git::read_git_object(&args[3]) {
            eprintln!("Error: {}", err);
        }
    }

    if args[1] == "hash-object" && args[2] == "-w" {
        let content_file = fs::read(&args[3].to_string()).unwrap();
        match git::write_git_object(content_file, "blob", "./") {
            Ok(hash_blob_file) => {
                println!("{}", hash_blob_file);
            }
            Err(err) => {
                eprintln!("Error: {}", err);
            }
        }
    }

    if args[1] == "ls-tree" && args[2] == "--name-only" {
        if let Err(err) = git::read_tree_object(args[3].to_string()) {
            eprintln!("Error: {}", err);
        }
    }

    if args[1] == "write-tree" {
        match git::write_tree_object(&".".to_string()) {
            Ok(result) => println!("{}", result),
            Err(err) => eprintln!("Error writing tree object: {}", err),
        }
    }

    if args[1] == "commit-tree" && args[3] == "-p" && args[5] == "-m" {
        match git::do_commit(
            args[2].to_string(),
            args[4].to_string(),
            args[6].to_string(),
        ) {
            Ok(sha_commit) => println!("{}", sha_commit),
            Err(err) => eprintln!("Error committing: {}", err),
        }
    }

    if args[1] == "clone" {
        if let Err(err) = git::clone_repo(args[3].to_string(), args[2].to_string()) {
            eprintln!("Error: {}", err);
        }
    }
}
