use sha1::{Digest, Sha1};

pub struct Blob {
    content: String,
}

impl Blob {
    pub fn new(data: String) -> Self {
        Blob { content: data }
    }

    fn encoded_header(&self) -> String {
        format!("{} {}\x00", "blob", self.content.len())
    }

    pub fn content_with_headers(&self) -> String {
        format!("{}{}", self.encoded_header(), self.content)
    }

    pub fn hash(&self) -> String {
        let mut sh = Sha1::default();
        sh.update(self.content_with_headers());

        let hash_result = sh.finalize();

        format!("{:x}", hash_result)
    }
}
