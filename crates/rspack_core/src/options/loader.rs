use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub enum Loader {
  Js,
  Jsx,
  Ts,
  Tsx,
  DataURI,
  Json,
  Text,
  Null,
}

pub type LoaderOptions = HashMap<String, Loader>;

pub struct LoadedFile {
  pub content: String,
  pub loader: Option<Loader>,
}

pub struct ResolvedLoadedFile {
  pub content: String,
  pub loader: Loader,
}

impl ResolvedLoadedFile {
  pub fn new(content: String) -> Self {
    Self {
      content,
      loader: Loader::Null,
    }
  }

  pub fn with_loader(content: String, loader: Loader) -> Self {
    Self { content, loader }
  }
}

impl LoadedFile {
  pub fn new(content: String) -> Self {
    Self {
      content,
      loader: None,
    }
  }

  pub fn with_loader(content: String, loader: Loader) -> Self {
    Self {
      content,
      loader: Some(loader),
    }
  }
}
