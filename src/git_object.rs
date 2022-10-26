use sha1::{Digest, Sha1};

// Parses a Key-Value List with Message string (hence kvlm).
// This message will look something like this:
//
// ```
// tree 1660685a18e10e2a097a8627ddb75f8dab7e8a3a
// parent 52f5c83450d57f83d9d9255d96b66283d54283d8
// author Hector Lorenzo Pons <hector@hectorlorenzo.me> 1666772992 +0100
// committer Hector Lorenzo Pons <hector@hectorlorenzo.me> 1666772992 +0100
//
// Remove serialiser mod
// ```
fn kvlm_parser<'a>(
    content: &'a str,
    kvv: Option<&mut Vec<(&'a str, String)>>,
) -> Result<Vec<(&'a str, String)>, &'static str> {
    // we assume we will find a header line, so we look for its key and its value
    // key will be from beginning to first empty space, value from this point to line break
    let blank_space_maybe = content.find(' ');
    let line_break_maybe = content.find('\n');

    if line_break_maybe.is_none() {
        return Err("Could not find a new line break, content is malformed");
    }

    let line_break = line_break_maybe.unwrap();

    // kvv is an optional argument because we only pass it when doing recursion.
    // Because it is optional, we need to create a vector if None has been passed
    // (when method is called on the first recursion).
    let mut temp_kvv = match kvv {
        Some(k) => k.to_owned(),
        None => vec![],
    };

    // if there is no blank space, it means that we have reached a blank line,
    // and we can start storing the message. If there is a blank space, we have
    // a header line.
    if blank_space_maybe.is_none() || (blank_space_maybe.unwrap() > line_break) {
        temp_kvv.push(("", (&content[line_break + 1..]).to_owned()));
        return Ok(temp_kvv);
    } else {
        let blank_space = blank_space_maybe.unwrap();

        temp_kvv.push((
            &content[..blank_space],
            (&content[blank_space + 1..line_break]).to_owned(),
        ));
        return kvlm_parser(&content[line_break + 1..], Some(&mut temp_kvv));
    }
}

fn kvlm_serialize(kvv: &Vec<(&str, String)>) -> String {
    return kvv.iter().fold(String::from(""), |acc, current| {
        let key = current.0;
        let value = &current.1;

        // if key is an empty string, it means it is the content, and we should format
        // it differently (do not show they key, add a line break before it).
        if key == "" {
            acc + format!("\n{}", value).as_str()
        } else {
            acc + format!("{} {}\n", key, value).as_str()
        }
    });
}

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
