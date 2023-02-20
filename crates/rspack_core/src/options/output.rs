use std::{
  path::{Path, PathBuf},
  str::FromStr,
  string::ParseError,
};

use sugar_path::SugarPath;

use crate::{Chunk, ChunkGroupByUkey, ChunkKind, Compilation};

#[derive(Debug)]
pub struct OutputOptions {
  pub path: PathBuf,
  pub public_path: PublicPath,
  pub asset_module_filename: Filename,
  pub unique_name: String,
  //todo we are not going to support file_name & chunk_file_name as function in the near feature
  pub filename: Filename,
  pub chunk_filename: Filename,
  pub css_filename: Filename,
  pub css_chunk_filename: Filename,
  pub library: Option<String>,
  pub strict_module_error_handling: bool,
}

pub const NAME_PLACEHOLDER: &str = "[name]";
pub const PATH_PLACEHOLDER: &str = "[path]";
pub const EXT_PLACEHOLDER: &str = "[ext]";
pub const ID_PLACEHOLDER: &str = "[id]";
pub const HASH_PLACEHOLDER: &str = "[hash]";
pub const CHUNK_HASH_PLACEHOLDER: &str = "[chunkhash]";
pub const CONTENT_HASH_PLACEHOLDER: &str = "[contenthash]";
pub const QUERY_PLACEHOLDER: &str = "[query]";

#[derive(Debug, Default)]
pub struct FilenameRenderOptions {
  pub name: Option<String>,
  pub path: Option<String>,
  pub extension: Option<String>,
  pub id: Option<String>,
  pub contenthash: Option<String>,
  pub chunkhash: Option<String>,
  pub hash: Option<String>,
  pub query: Option<String>,
}
#[derive(Debug, Clone)]
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

impl From<String> for Filename {
  fn from(value: String) -> Self {
    Self { template: value }
  }
}

impl Filename {
  pub fn render(&self, options: FilenameRenderOptions) -> String {
    let mut filename = self.template.clone();
    if let Some(name) = options.name {
      filename = filename.replace(NAME_PLACEHOLDER, &name);
    }

    if let Some(path) = options.path {
      filename = filename.replace(PATH_PLACEHOLDER, &path);
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

    if let Some(query) = options.query {
      filename = filename.replace(QUERY_PLACEHOLDER, &query);
    }
    filename
  }
}

#[derive(Debug)]
pub enum PublicPath {
  String(String),
  Auto,
}

impl PublicPath {
  pub fn render(&self, compilation: &Compilation, filename: &str) -> String {
    match self {
      Self::String(s) => s.clone(),
      Self::Auto => match Path::new(filename).parent() {
        None => "".to_string(),
        Some(dirname) => compilation
          .options
          .output
          .path
          .join(dirname)
          .resolve()
          .relative(&compilation.options.output.path)
          .to_string_lossy()
          .to_string(),
      },
    }
  }
}

impl Default for PublicPath {
  fn default() -> Self {
    Self::from_str("/").expect("TODO:")
  }
}

impl FromStr for PublicPath {
  type Err = ParseError;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    if s.eq("auto") {
      Ok(PublicPath::Auto)
    } else {
      Ok(PublicPath::String(s.to_string()))
    }
  }
}

impl From<String> for PublicPath {
  fn from(value: String) -> Self {
    if value == "auto" {
      Self::Auto
    } else {
      Self::String(value)
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
      hash_length = hash_length_string.parse().expect("TODO:")
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

#[allow(clippy::if_same_then_else)]
pub fn get_css_chunk_filename_template<'filename>(
  chunk: &Chunk,
  output_options: &'filename OutputOptions,
  chunk_group_by_ukey: &ChunkGroupByUkey,
) -> &'filename Filename {
  // Align with https://github.com/webpack/webpack/blob/8241da7f1e75c5581ba535d127fa66aeb9eb2ac8/lib/css/CssModulesPlugin.js#L444
  if chunk.can_be_initial(chunk_group_by_ukey) {
    &output_options.css_filename
  } else {
    &output_options.css_chunk_filename
  }
}

#[allow(clippy::if_same_then_else)]
pub fn get_js_chunk_filename_template<'filename>(
  chunk: &Chunk,
  output_options: &'filename OutputOptions,
  chunk_group_by_ukey: &ChunkGroupByUkey,
) -> &'filename Filename {
  // Align with https://github.com/webpack/webpack/blob/8241da7f1e75c5581ba535d127fa66aeb9eb2ac8/lib/javascript/JavascriptModulesPlugin.js#L480
  if chunk.can_be_initial(chunk_group_by_ukey) {
    &output_options.filename
  } else if matches!(chunk.kind, ChunkKind::HotUpdate) {
    // TODO: Should return output_options.hotUpdateChunkFilename
    // See https://github.com/webpack/webpack/blob/8241da7f1e75c5581ba535d127fa66aeb9eb2ac8/lib/javascript/JavascriptModulesPlugin.js#L484
    &output_options.chunk_filename
  } else {
    &output_options.chunk_filename
  }
}
