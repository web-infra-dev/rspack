use std::{str::FromStr, string::ParseError};

#[derive(Debug)]
pub struct OutputOptions {
  pub path: String,
  pub public_path: PublicPath,
  pub asset_module_filename: Filename,
  pub unique_name: String,
  //todo we are not going to support file_name & chunk_file_name as function in the near feature
  pub filename: Filename,
  pub chunk_filename: Filename,
}

pub const NAME_PLACEHOLDER: &str = "[name]";
pub const EXT_PLACEHOLDER: &str = "[ext]";
pub const ID_PLACEHOLDER: &str = "[id]";
pub const HASH_PLACEHOLDER: &str = "[hash]";
pub const CHUNK_HASH_PLACEHOLDER: &str = "[chunkhash]";
pub const CONTENT_PLACEHOLDER: &str = "[contenthash]";

pub struct FilenameRenderOptions {
  pub filename: Option<String>,
  pub extension: Option<String>,
  pub id: Option<String>,
}
#[derive(Debug)]
pub struct Filename {
  template: String,
}

impl FromStr for Filename {
  type Err = ParseError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Ok(Self {
      template: s.to_string(),
    })
  }
}

impl Filename {
  pub fn render(&self, options: FilenameRenderOptions) -> String {
    let mut filename = self.template.clone();
    if let Some(name) = options.filename {
      filename = filename.replace(NAME_PLACEHOLDER, &name);
    }

    if let Some(ext) = options.extension {
      filename = filename.replace(EXT_PLACEHOLDER, &ext);
    }

    if let Some(id) = options.id {
      filename = filename.replace(ID_PLACEHOLDER, &id);
    }

    filename
  }
}

#[derive(Debug)]
pub enum PublicPathType {
  String(String),
  Auto,
}
#[derive(Debug)]
pub struct PublicPath {
  content: PublicPathType,
}

impl Default for PublicPath {
  fn default() -> Self {
    Self::from_str("/").unwrap()
  }
}

impl FromStr for PublicPath {
  type Err = ParseError;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    if s.eq("auto") {
      Ok(PublicPath {
        content: PublicPathType::Auto,
      })
    } else {
      Ok(PublicPath {
        content: PublicPathType::String(s.to_string()),
      })
    }
  }
}

impl PublicPath {
  pub fn public_path(&self) -> &str {
    match &self.content {
      PublicPathType::String(s) => s,
      PublicPathType::Auto => "__rspack_auto_public_path__ +",
    }
  }
}
