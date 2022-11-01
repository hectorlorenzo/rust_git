use super::serialise::Serialise;
use sha1::{Digest, Sha1};

pub struct Tree {
    content: String,
}

impl Serialise for Tree {
    fn serialise(&self) -> &String {
        &self.content
    }

    fn content_with_headers(&self) -> String {
        format!("{}{}", self.encoded_header(), self.serialise())
    }

    fn encoded_header(&self) -> String {
        let content = self.serialise();

        format!("{} {}\x00", "blob", content.len())
    }

    fn hash(&self) -> String {
        let mut sh = Sha1::default();
        sh.update(self.content_with_headers());

        let hash_result = sh.finalize();

        format!("{:x}", hash_result)
    }
}
