pub trait Serialise {
    fn serialise(&self) -> &String;
    fn content_with_headers(&self) -> String;
    fn encoded_header(&self) -> String;
    fn hash(&self) -> String;
}
