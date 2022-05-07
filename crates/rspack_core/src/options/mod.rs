use std::collections::HashMap;

#[derive(Debug)]
pub enum Loader {
  DataURI,
}

pub type LoaderOptions = HashMap<String, Loader>;
