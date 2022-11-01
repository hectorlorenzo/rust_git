mod git_object;
mod object;
mod repository;

use std::{env, fs::read_to_string, path::PathBuf};

use clap::{Parser, Subcommand};
use object::{blob::Blob, serialise::Serialise};

use git_object::GitObject;
use repository::Repository;

#[derive(Parser)]
struct Cli {
    #[clap(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Add,
    CatFile {
        object_type: String,
        object: String,
    },
    Checkout,
    Commit,
    HashObject {
        #[arg(short, long)]
        r#type: Option<String>,

        #[arg(short, long, default_value_t = false)]
        write: bool,

        filepath: String,
    },
    Init {
        path: String,
    },
    Log,
    LsTree,
    Merge,
    Rebase,
    RevParse,
    Rm,
    ShowRef,
    Tag,
}

fn commit_command() {
    let repo = Repository::repo_find(env::current_dir().unwrap())
        .expect("Could not find a valid Got repository in this location.");

    let files = repo
        .list_files()
        .expect("Could not read files in repository.");

    let blobs = files
        .iter()
        .map(|path| read_to_string(path).unwrap())
        .map(|data| Blob::new(data));

    for blob in blobs {
        repo.object_write(&blob, true);
    }
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Add) => {}
        Some(Commands::CatFile {
            object_type,
            object,
        }) => {
            let repo = match Repository::repo_find(env::current_dir().unwrap()) {
                Some(repo) => repo,
                None => panic!("Could not find repository"),
            };

            let obj = repo
                .object_read(object)
                .expect("Could not find object with given hash");

            println!("{}", obj.serialise());
        }
        Some(Commands::Checkout) => {}
        Some(Commands::Commit) => {
            commit_command();
        }
        Some(Commands::HashObject {
            r#type,
            write,
            filepath,
        }) => {
            // TO BE DONE AGAIN
            // ================
            // let file_content = read_to_string(filepath).expect("Could not read file");
            // let object_type = r#type.as_deref().unwrap_or("blob");
            // let object = GitObject::new(object_type, file_content);

            // // If we are in a repo, we should offer the option of writing file in the repo.
            // // If we are not, we should just show the hash of this file.
            // match Repository::repo_find(env::current_dir().unwrap()) {
            //     Some(repo) => {
            //         if *write {
            //             println!("{}", repo.object_write(object, true));
            //         } else {
            //             println!("{}", object.hash())
            //         }
            //     }
            //     None => {
            //         println!("{}", object.hash())
            //     }
            // }
        }
        Some(Commands::Init { path }) => {
            Repository::create(PathBuf::from(path)).unwrap();
        }
        Some(Commands::Log) => {
            println!("Log");
        }
        Some(Commands::LsTree) => {
            println!("LsTree");
        }
        Some(Commands::Merge) => {
            println!("Merge");
        }
        Some(Commands::Rebase) => {
            println!("Rebase");
        }
        Some(Commands::RevParse) => {
            println!("RevParse");
        }
        Some(Commands::Rm) => {
            println!("Rm");
        }
        Some(Commands::ShowRef) => {
            println!("ShowRef");
        }
        Some(Commands::Tag) => {
            println!("Tag");
        }
        _ => {}
    }
}
