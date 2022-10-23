use std::{
    fs::{canonicalize, create_dir_all, read, read_to_string, write, File},
    io::{Read, Write},
    path::PathBuf,
    str::{from_utf8, FromStr},
};

use clap::{Parser, Subcommand};
use configparser::ini::Ini;
use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};
use sha1::{Digest, Sha1};

mod serializer;

const GOT_DIR: &str = ".got";

#[derive(Parser)]
struct Cli {
    #[clap(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Add,
    CatFile { object_type: String, object: String },
    Checkout,
    Commit,
    HashObject,
    Init { path: String },
    Log,
    LsTree,
    Merge,
    Rebase,
    RevParse,
    Rm,
    ShowRef,
    Tag,
}

trait GitObject {
    fn object_type(&self) -> &str;
    fn content(&self) -> &str;
    fn serialise(&self) -> &str {
        return self.content();
    }
    fn deserialise(&mut self, content: &str);
}

struct GitCommit {
    content: String,
}
impl GitObject for GitCommit {
    fn object_type(&self) -> &str {
        "commit"
    }
    fn content(&self) -> &str {
        &self.content[..]
    }
    fn deserialise(&mut self, content: &str) {
        self.content = content.to_owned();
    }
}

struct GitTree {
    content: String,
}
impl GitObject for GitTree {
    fn object_type(&self) -> &str {
        "tree"
    }
    fn content(&self) -> &str {
        &self.content[..]
    }
    fn deserialise(&mut self, content: &str) {
        self.content = content.to_owned();
    }
}

struct GitTag {
    content: String,
}
impl GitObject for GitTag {
    fn object_type(&self) -> &str {
        "tag"
    }
    fn content(&self) -> &str {
        &self.content[..]
    }
    fn deserialise(&mut self, content: &str) {
        self.content = content.to_owned();
    }
}

struct GitBlob {
    content: String,
}
impl GitObject for GitBlob {
    fn object_type(&self) -> &str {
        "blob"
    }
    fn content(&self) -> &str {
        &self.content[..]
    }
    fn deserialise(&mut self, content: &str) {
        self.content = content.to_owned();
    }
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

    fn generate_default_config() -> Ini {
        let mut config = Ini::new();

        config.set("core", "repositoryformatversion", Some(String::from("0")));
        config.set("core", "filemode", Some(String::from("false")));
        config.set("core", "bare", Some(String::from("false")));

        return config;
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
            create_dir_all(&path).expect("Could not create repository");
            path
        };

        let repo = Repository::new(working_dir, true);

        repo.repo_dir("branches", true);
        repo.repo_dir("objects", true);
        repo.repo_dir("refs/tags", true);
        repo.repo_dir("refs/heads", true);

        // Create description file
        write(
            repo.repo_file("description", false),
            "Unnamed repository; edit this file 'description' to name the repository.\n",
        )
        .expect("Could not write description file");

        // Write HEAD file
        write(repo.repo_file("HEAD", false), "ref: refs/heads/master\n")
            .expect("Could not write HEAD file");

        // Write configuration file
        let config = Repository::generate_default_config();
        config
            .write(repo.repo_file("config", false))
            .expect("Could not write configuration file on repo creation");

        Ok(repo)
    }

    // From current repository, return a parent directory that is an active repository.
    // We identify an active repository because it contains a ".got" directory.
    // Useful when we want to execute commands when inside child directories.
    fn repo_find(path: &PathBuf) -> Option<Self> {
        let canonical_path = canonicalize(&path).expect("Could not convert path to canonical");
        let parent_path = canonical_path.parent();
        let maybe_got_path = canonical_path.join(GOT_DIR).is_dir();

        if parent_path.is_none() {
            return None;
        } else if maybe_got_path {
            return Some(Repository::new(canonical_path, false));
        } else {
            return Repository::repo_find(&parent_path.unwrap().to_owned());
        }
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

        if !path.parent().is_none() {
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
                match create_dir_all(&path) {
                    Ok(()) => Some(path),
                    Err(error) => panic!("Could not create directory: {:?}", error),
                }
            } else {
                return None;
            }
        }
    }

    fn object_find(&self, name: &'static str, _format: &str, _follow: bool) -> &'static str {
        return name;
    }

    fn object_write(&self, object: &dyn GitObject) {
        let data = object.serialise();
        let content_with_headers = format!("{} {}\x00{}", object.object_type(), data.len(), data);
        let mut hasher = Sha1::new();
        hasher.update(&content_with_headers);

        let hash = &hasher.finalize()[..];
        let folder_name = from_utf8(&hash[..2]).unwrap();
        let filename = from_utf8(&hash[2..]).unwrap();

        let file_path = self.repo_file(&format!("{}/{}", folder_name, filename)[..], true);

        let file_writer = File::create(file_path).expect("Could not create file.");

        let mut file_contents_encoder = ZlibEncoder::new(file_writer, Compression::fast());
        file_contents_encoder
            .write(content_with_headers.as_bytes())
            .expect("Could not compress object contents.");
    }

    fn object_read(&self, sha: &str) -> Result<Box<dyn GitObject>, &'static str> {
        let file_relative_path = format!("objects/{}/{}", &sha[..2], &sha[2..]);
        let file_relative_path_str = file_relative_path.as_str();
        let file_path = self.repo_file(file_relative_path_str, false);

        let compressed_file_contents = read(file_path).expect("File does not exist");
        let mut file_contents_decoder = ZlibDecoder::new(&compressed_file_contents[..]);
        let mut file_contents = String::new();
        file_contents_decoder
            .read_to_string(&mut file_contents)
            .unwrap();

        let object_type_index = match file_contents.find(' ') {
            Some(index) => index,
            None => return Err("File is malformed"),
        };
        let object_type = &file_contents[0..object_type_index];

        let object_size_index = match file_contents.find('\x00') {
            Some(index) => index,
            None => return Err("File is malformed"),
        };
        let object_size = &file_contents[object_type_index + 1..object_size_index];
        let object_content = &file_contents[object_size_index..];
        let real_object_size = object_content.len() - 1;

        if object_size.parse::<usize>().unwrap() != real_object_size {
            return Err("Could not read object because sizes mismatch (object is malformed).");
        } else {
            let content = object_content.to_string();

            match object_type {
                "commit" => Ok(Box::new(GitCommit { content })),
                "tree" => Ok(Box::new(GitTree { content })),
                "tag" => Ok(Box::new(GitTag { content })),
                "blob" => Ok(Box::new(GitBlob { content })),
                _ => Err("Object type does not match any known types."),
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
        Some(Commands::CatFile {
            object_type,
            object,
        }) => {
            let repo = match Repository::repo_find(&PathBuf::from("./test")) {
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
            println!("Commit");
        }
        Some(Commands::HashObject) => {
            println!("HashObject");
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
