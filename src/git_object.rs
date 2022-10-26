use sha1::{Digest, Sha1};

pub enum GitObject {
    Commit(String),
    Blob(String),
    Tag(String),
    Tree(String),
}

impl GitObject {
    pub fn new(type_str: &str, content: String) -> Self {
        match type_str {
            "commit" => GitObject::Commit(content),
            "blob" => GitObject::Blob(content),
            "tag" => GitObject::Tag(content),
            "tree" => GitObject::Tree(content),
            _ => panic!("Incorrect type to initialise an object."),
        }
    }

    pub fn type_string(&self) -> String {
        match self {
            GitObject::Commit(_) => String::from("commit"),
            GitObject::Blob(_) => String::from("blob"),
            GitObject::Tag(_) => String::from("tag"),
            GitObject::Tree(_) => String::from("tree"),
        }
    }

    pub fn serialise(&self) -> &String {
        match self {
            GitObject::Commit(content) => content,
            GitObject::Blob(content) => content,
            GitObject::Tag(content) => content,
            GitObject::Tree(content) => content,
        }
    }

    pub fn content_with_headers(&self) -> String {
        format!("{}{}", self.encoded_header(), self.serialise())
    }

    pub fn encoded_header(&self) -> String {
        let content = self.serialise();

        format!("{} {}\x00", self.type_string(), content.len())
    }

    pub fn hash(&self) -> String {
        let mut sh = Sha1::default();
        sh.update(self.content_with_headers());

        let hash_result = sh.finalize();

        format!("{:x}", hash_result)
    }
}
