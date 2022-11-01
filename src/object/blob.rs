use sha1::{Digest, Sha1};

use super::serialise::Serialise;

pub struct Blob {
    content: String,
}

impl Blob {
    pub fn new(content: String) -> Self {
        Blob { content }
    }
}

impl Serialise for Blob {
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
