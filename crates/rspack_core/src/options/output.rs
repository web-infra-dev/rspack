use std::{
  path::{Path, PathBuf},
  str::FromStr,
  string::ParseError,
};

use once_cell::sync::Lazy;
use regex::{Captures, Regex};
use sugar_path::SugarPath;

use crate::{AssetInfo, Chunk, ChunkGroupByUkey, ChunkKind, Compilation, SourceType};

#[derive(Debug)]
pub struct OutputOptions {
  pub path: PathBuf,
  pub clean: bool,
  pub public_path: PublicPath,
  pub asset_module_filename: Filename,
  pub wasm_loading: WasmLoading,
  pub webassembly_module_filename: Filename,
  pub unique_name: String,
  pub chunk_loading_global: String,
  pub filename: Filename,
  pub chunk_filename: Filename,
  pub cross_origin_loading: CrossOriginLoading,
  pub css_filename: Filename,
  pub css_chunk_filename: Filename,
  pub library: Option<LibraryOptions>,
  pub enabled_library_types: Option<Vec<String>>,
  pub strict_module_error_handling: bool,
  pub global_object: String,
  pub import_function_name: String,
  pub iife: bool,
  pub module: bool,
  pub trusted_types: Option<TrustedTypes>,
  pub source_map_filename: Filename,
}

#[derive(Debug)]
pub struct TrustedTypes {
  pub policy_name: Option<String>,
}

#[derive(Debug)]
pub enum WasmLoading {
  Enable(WasmLoadingType),
  Disable,
}

#[derive(Debug)]
pub enum WasmLoadingType {
  Fetch,
  AsyncNode,
  AsyncNodeModule,
}

impl From<&str> for WasmLoadingType {
  fn from(value: &str) -> Self {
    match value {
      "fetch" => Self::Fetch,
      "async-node" => Self::AsyncNode,
      "async-node-module" => Self::AsyncNodeModule,
      _ => todo!(),
    }
  }
}

#[derive(Debug)]
pub enum CrossOriginLoading {
  Disable,
  Enable(String),
}

impl std::fmt::Display for CrossOriginLoading {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      CrossOriginLoading::Disable => write!(f, "false"),
      CrossOriginLoading::Enable(value) => write!(f, "'{}'", value),
    }
  }
}

pub const FILE_PLACEHOLDER: &str = "[file]";
pub const NAME_PLACEHOLDER: &str = "[name]";
pub const PATH_PLACEHOLDER: &str = "[path]";
pub const EXT_PLACEHOLDER: &str = "[ext]";
pub const ID_PLACEHOLDER: &str = "[id]";
pub static HASH_PLACEHOLDER: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"\[hash(:(\d*))?]").expect("Invalid regex"));
pub static CHUNK_HASH_PLACEHOLDER: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"\[chunkhash(:(\d*))?]").expect("Invalid regex"));
pub static CONTENT_HASH_PLACEHOLDER: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"\[contenthash(:(\d*))?]").expect("Invalid regex"));
pub const QUERY_PLACEHOLDER: &str = "[query]";

#[derive(Debug, Default)]
pub struct FilenameRenderOptions {
  pub file: Option<String>,
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
  pub fn render_with_chunk(
    &self,
    chunk: &Chunk,
    extension: &str,
    source_type: &SourceType,
    asset_info: Option<&mut AssetInfo>,
  ) -> String {
    let hash = Some(chunk.get_render_hash());
    self.render(
      FilenameRenderOptions {
        // See https://github.com/webpack/webpack/blob/4b4ca3bb53f36a5b8fc6bc1bd976ed7af161bd80/lib/TemplatedPathPlugin.js#L214
        name: chunk.name_for_filename_template(),
        extension: Some(extension.to_owned()),
        id: chunk.id.clone(),
        contenthash: chunk.content_hash.get(source_type).cloned(),
        chunkhash: hash.clone(),
        hash,
        ..Default::default()
      },
      asset_info,
    )
  }

  pub fn render_with_chunk_and_file(
    &self,
    chunk: &Chunk,
    file: &str,
    extension: &str,
    source_type: &SourceType,
    asset_info: Option<&mut AssetInfo>,
  ) -> String {
    let hash = Some(chunk.get_render_hash());
    self.render(
      FilenameRenderOptions {
        // See https://github.com/webpack/webpack/blob/4b4ca3bb53f36a5b8fc6bc1bd976ed7af161bd80/lib/TemplatedPathPlugin.js#L214
        file: Some(file.to_owned()),
        name: chunk.name_for_filename_template(),
        extension: Some(extension.to_owned()),
        id: chunk.id.clone(),
        contenthash: chunk.content_hash.get(source_type).cloned(),
        chunkhash: hash.clone(),
        hash,
        ..Default::default()
      },
      asset_info,
    )
  }

  pub fn render(
    &self,
    options: FilenameRenderOptions,
    mut asset_info: Option<&mut AssetInfo>,
  ) -> String {
    let mut filename = self.template.clone();
    if let Some(file) = options.file {
      filename = filename.replace(FILE_PLACEHOLDER, &file);
    }
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
      filename = CONTENT_HASH_PLACEHOLDER
        .replace_all(&filename, |caps: &Captures| {
          let hash_len = contenthash.len();
          let hash_len: usize = caps
            .get(2)
            .and_then(|m| m.as_str().parse().ok())
            .unwrap_or(hash_len)
            .min(hash_len);
          let contenthash = &contenthash[..hash_len.min(contenthash.len())];
          if let Some(asset_info) = asset_info.as_mut() {
            asset_info.set_content_hash(contenthash.to_owned());
          }
          contenthash
        })
        .into_owned();
    }

    if let Some(chunkhash) = options.chunkhash {
      filename = CHUNK_HASH_PLACEHOLDER
        .replace_all(&filename, |caps: &Captures| {
          let hash_len = chunkhash.len();
          let hash_len: usize = caps
            .get(2)
            .and_then(|m| m.as_str().parse().ok())
            .unwrap_or(hash_len)
            .min(hash_len);
          &chunkhash[..hash_len.min(chunkhash.len())]
        })
        .into_owned();
    }

    if let Some(hash) = options.hash {
      filename = HASH_PLACEHOLDER
        .replace_all(&filename, |caps: &Captures| {
          let hash_len = hash.len();
          let hash_len: usize = caps
            .get(2)
            .and_then(|m| m.as_str().parse().ok())
            .unwrap_or(hash_len)
            .min(hash_len);
          &hash[..hash_len]
        })
        .into_owned();
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
    let public_path = match self {
      Self::String(s) => s.clone(),
      Self::Auto => match Path::new(filename).parent() {
        None => "".to_string(),
        Some(dirname) => compilation
          .options
          .output
          .path
          .relative(compilation.options.output.path.join(dirname).absolutize())
          .to_string_lossy()
          .to_string(),
      },
    };
    Self::ensure_ends_with_slash(public_path)
  }

  pub fn ensure_ends_with_slash(public_path: String) -> String {
    if !public_path.is_empty() && !public_path.ends_with('/') {
      public_path + "/"
    } else {
      public_path
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

#[derive(Debug, Hash)]
pub struct LibraryOptions {
  pub name: Option<LibraryName>,
  pub export: Option<Vec<String>>,
  // webpack type
  pub library_type: String,
  pub umd_named_define: Option<bool>,
  pub auxiliary_comment: Option<LibraryAuxiliaryComment>,
}

#[derive(Debug, Hash)]
pub struct LibraryAuxiliaryComment {
  pub root: Option<String>,
  pub commonjs: Option<String>,
  pub commonjs2: Option<String>,
  pub amd: Option<String>,
}

#[derive(Debug, Hash)]
pub struct LibraryName {
  pub amd: Option<String>,
  pub commonjs: Option<String>,
  pub root: Option<Vec<String>>,
}
