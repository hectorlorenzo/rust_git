use std::{
    error::Error,
    fmt,
    fs::{create_dir, create_dir_all, read_to_string},
    path::{Path, PathBuf},
    str::FromStr,
};

use clap::{Parser, Subcommand};
use configparser::ini::Ini;

const GOT_DIR: &str = ".got";

#[derive(Parser)]
struct Cli {
    #[clap(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Add,
    CatFile,
    Checkout { path: String },
    Commit,
    HashObject,
    Init,
    Log,
    LsTree,
    Merge,
    Rebase,
    RevParse,
    Rm,
    ShowRef,
    Tag,
}

struct Repository {
    worktree: PathBuf,
    gotdir: PathBuf,
}

impl Repository {
    // Returns a new Repository.
    // path is the system absolute path for this repository.
    // force ensures that a new Repository is created even if gotdir (.got)
    // does not exist. Useful for creating new repositories.
    fn new(path: PathBuf, force: bool) -> Self {
        // 1. Panic if <path>/.got does not exist and force is false
        // 2. Read configuration file from /.got/config, panic if missing and force is false
        // 3. Read repositoryformatversion from config
        let gotdir = path.join(GOT_DIR);

        if !gotdir.is_dir() && !force {
            panic!("Not a valid Got repository");
        }

        let config_file_path = gotdir.join("config");
        let mut config_parser = Ini::new();

        let config = match read_to_string(config_file_path) {
            Ok(config_content) => Some(
                config_parser
                    .read(config_content)
                    .expect("Configuration file has wrong format"),
            ),
            Err(_) => None,
        };

        if config.is_none() && !force {
            panic!("Configuration file not found");
        }

        return Repository {
            worktree: path,
            gotdir,
        };
    }

    fn create(path: PathBuf) -> Result<Self, String> {
        // Does the path exist and it is a dir? Create subdirs and return Repo
        // Does the path exist and it is not a dir? Return Error
        // Does the path not exist? Create the path
        let working_dir = if path.exists() {
            if path.is_dir() {
                path
            } else {
                return Err(format!(
                    "Could not create repository because {:?} is not a directory",
                    path,
                ));
            }
        } else {
            create_dir_all(path.clone()).expect("Could not create repository");
            path
        };

        let repo = Repository::new(working_dir, true);

        repo.repo_dir("branches", true);
        repo.repo_dir("objects", true);
        repo.repo_dir("refs/tags", true);
        repo.repo_dir("refs/heads", true);

        Ok(repo)
    }

    // Returns a new path that is relative to .got dir
    fn repo_path(&self, rel_path_str: &str) -> PathBuf {
        let rel_path = PathBuf::from_str(rel_path_str).expect("Invalid path");
        self.gotdir.join(rel_path)
    }

    // Returns a new file path that is relative to .got dir and it maybe created
    // its dir along the way.
    fn repo_file(&self, rel_path_str: &str, should_create_dir: bool) -> PathBuf {
        let path = self.repo_path(rel_path_str);

        if !path.is_file() {
            panic!("{} is not a file path", rel_path_str);
        } else if !path.parent().is_none() {
            return path;
        } else {
            let mut dir_path = path.clone();
            dir_path.pop();

            if self.repo_dir(rel_path_str, should_create_dir).is_none() {
                panic!("Could not create file because path does not exist");
            } else {
                return path;
            }
        }
    }

    // Returns a dir path that is relative to .got dir and maybe creates it if it does
    // not exist.
    fn repo_dir(&self, rel_path_str: &str, should_create_dir: bool) -> Option<PathBuf> {
        let path = self.repo_path(rel_path_str);

        if path.exists() {
            if path.is_dir() {
                return Some(path);
            } else {
                panic!("Provided path is not a directory")
            }
        } else {
            if should_create_dir {
                match create_dir_all(path.clone()) {
                    Ok(()) => Some(path),
                    Err(error) => panic!("Could not create directory: {:?}", error),
                }
            } else {
                return None;
            }
        }
    }
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Add) => {
            println!("Add");
        }
        Some(Commands::CatFile) => {
            println!("Init");
        }
        Some(Commands::Checkout { path }) => {
            println!("Checkout path: {:?}", path);
            Repository::create(PathBuf::from(path));
        }
        Some(Commands::Commit) => {
            println!("Commit");
        }
        Some(Commands::HashObject) => {
            println!("HashObject");
        }
        Some(Commands::Init) => {
            println!("Init");
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
