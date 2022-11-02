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
pub const CONTENT_HASH_PLACEHOLDER: &str = "[contenthash]";

#[derive(Debug, Clone, Default)]
pub struct FilenameRenderOptions {
  pub filename: Option<String>,
  pub extension: Option<String>,
  pub id: Option<String>,
  pub contenthash: Option<String>,
  pub chunkhash: Option<String>,
  pub hash: Option<String>,
}
#[derive(Debug, Clone)]
pub struct Filename {
  pub template: String,
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

    if let Some(contenthash) = options.contenthash {
      let hash_placeholder = get_hash_placeholder(&filename, CONTENT_HASH_PLACEHOLDER);
      let hash_length: usize = get_hash_length(&hash_placeholder, CONTENT_HASH_PLACEHOLDER);

      filename = filename.replace(&hash_placeholder, &contenthash[..hash_length]);
    }

    if let Some(chunkhash) = options.chunkhash {
      let hash_placeholder = get_hash_placeholder(&filename, CHUNK_HASH_PLACEHOLDER);
      let hash_length: usize = get_hash_length(&hash_placeholder, CHUNK_HASH_PLACEHOLDER);

      filename = filename.replace(&hash_placeholder, &chunkhash[..hash_length]);
    }

    if let Some(hash) = options.hash {
      let hash_placeholder = get_hash_placeholder(&filename, HASH_PLACEHOLDER);
      let hash_length: usize = get_hash_length(&hash_placeholder, HASH_PLACEHOLDER);

      filename = filename.replace(&hash_placeholder, &hash[..hash_length]);
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

fn get_hash_placeholder(filename: &str, placeholder: &str) -> String {
  let filename_split = filename.split('.');

  let mut return_placeholder: String = String::from(placeholder);
  for sub_filename in filename_split {
    if sub_filename.contains(&placeholder[..placeholder.len() - 1]) {
      return_placeholder = String::from(sub_filename);
    }
  }

  return_placeholder
}

fn get_hash_length(placeholder_with_length: &str, placeholder: &str) -> usize {
  let mut hash_length: usize = placeholder_with_length.len();
  let start_index: usize = placeholder.len() - 1;
  let end_index: usize = placeholder_with_length.len() - 1;
  if start_index < end_index {
    let hash_length_string = String::from(&placeholder_with_length[start_index + 1..end_index]);
    if !hash_length_string.is_empty() && is_string_numeric(&hash_length_string) {
      hash_length = hash_length_string.parse().unwrap()
    }
  }

  hash_length
}

fn is_string_numeric(str: &str) -> bool {
  for c in str.chars() {
    if !c.is_numeric() {
      return false;
    }
  }
  true
}
