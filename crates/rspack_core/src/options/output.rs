use std::{
  fmt::Debug,
  hash::Hash,
  path::{Path, PathBuf},
  str::FromStr,
  string::ParseError,
};

use derivative::Derivative;
use rspack_hash::RspackHash;
pub use rspack_hash::{HashDigest, HashFunction, HashSalt};
use rspack_macros::MergeFrom;
use sugar_path::SugarPath;

use crate::{
  Chunk, ChunkGraph, ChunkGroupByUkey, ChunkKind, Compilation, Filename, FilenameTemplate, Module,
  RuntimeSpec,
};

#[derive(Debug)]
pub enum PathInfo {
  Bool(bool),
  String(String),
}

#[derive(Debug)]
pub struct OutputOptions {
  pub path: PathBuf,
  pub pathinfo: PathInfo,
  pub clean: bool,
  pub public_path: PublicPath,
  pub asset_module_filename: Filename,
  pub wasm_loading: WasmLoading,
  pub webassembly_module_filename: FilenameTemplate,
  pub unique_name: String,
  pub chunk_loading: ChunkLoading,
  pub chunk_loading_global: String,
  pub filename: Filename,
  pub chunk_filename: Filename,
  pub cross_origin_loading: CrossOriginLoading,
  pub css_filename: Filename,
  pub css_chunk_filename: Filename,
  pub hot_update_main_filename: FilenameTemplate,
  pub hot_update_chunk_filename: FilenameTemplate,
  pub hot_update_global: String,
  pub library: Option<LibraryOptions>,
  pub enabled_library_types: Option<Vec<String>>,
  pub strict_module_error_handling: bool,
  pub global_object: String,
  pub import_function_name: String,
  pub iife: bool,
  pub module: bool,
  pub trusted_types: Option<TrustedTypes>,
  pub source_map_filename: FilenameTemplate,
  pub hash_function: HashFunction,
  pub hash_digest: HashDigest,
  pub hash_digest_length: usize,
  pub hash_salt: HashSalt,
  pub async_chunks: bool,
  pub worker_chunk_loading: ChunkLoading,
  pub worker_wasm_loading: WasmLoading,
  pub worker_public_path: String,
  pub script_type: String,
  pub environment: Environment,
}

impl From<&OutputOptions> for RspackHash {
  fn from(value: &OutputOptions) -> Self {
    Self::with_salt(&value.hash_function, &value.hash_salt)
  }
}

#[derive(Debug)]
pub struct TrustedTypes {
  pub policy_name: Option<String>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum ChunkLoading {
  Enable(ChunkLoadingType),
  Disable,
}

impl From<&str> for ChunkLoading {
  fn from(value: &str) -> Self {
    match value {
      "false" => ChunkLoading::Disable,
      v => ChunkLoading::Enable(v.into()),
    }
  }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum ChunkLoadingType {
  Jsonp,
  ImportScripts,
  Require,
  AsyncNode,
  Import,
  // TODO: Custom
}

impl From<&str> for ChunkLoadingType {
  fn from(value: &str) -> Self {
    match value {
      "jsonp" => Self::Jsonp,
      "import-scripts" => Self::ImportScripts,
      "require" => Self::Require,
      "async-node" => Self::AsyncNode,
      "import" => Self::Import,
      _ => unimplemented!("custom chunkLoading in not supported yet"),
    }
  }
}

#[derive(Debug, Clone)]
pub enum WasmLoading {
  Enable(WasmLoadingType),
  Disable,
}

impl From<&str> for WasmLoading {
  fn from(value: &str) -> Self {
    match value {
      "false" => Self::Disable,
      v => Self::Enable(v.into()),
    }
  }
}

#[derive(Debug, Clone)]
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
      CrossOriginLoading::Enable(value) => write!(f, "\"{}\"", value),
    }
  }
}

#[derive(Derivative, Default, Clone, Copy)]
#[derivative(Debug)]
pub struct PathData<'a> {
  pub filename: Option<&'a str>,
  #[derivative(Debug = "ignore")]
  pub chunk: Option<&'a Chunk>,
  #[derivative(Debug = "ignore")]
  pub module: Option<&'a dyn Module>,
  pub hash: Option<&'a str>,
  pub content_hash: Option<&'a str>,
  #[derivative(Debug = "ignore")]
  pub chunk_graph: Option<&'a ChunkGraph>,
  pub runtime: Option<&'a str>,
  pub url: Option<&'a str>,
  pub id: Option<&'a str>,
}

impl<'a> PathData<'a> {
  pub fn filename(mut self, v: &'a str) -> Self {
    self.filename = Some(v);
    self
  }

  pub fn chunk(mut self, v: &'a Chunk) -> Self {
    self.chunk = Some(v);
    self
  }

  pub fn module(mut self, v: &'a dyn Module) -> Self {
    self.module = Some(v);
    self
  }

  pub fn hash(mut self, v: &'a str) -> Self {
    self.hash = Some(v);
    self
  }

  pub fn hash_optional(mut self, v: Option<&'a str>) -> Self {
    self.hash = v;
    self
  }

  pub fn content_hash(mut self, v: &'a str) -> Self {
    self.content_hash = Some(v);
    self
  }

  pub fn content_hash_optional(mut self, v: Option<&'a str>) -> Self {
    self.content_hash = v;
    self
  }

  pub fn chunk_graph(mut self, v: &'a ChunkGraph) -> Self {
    self.chunk_graph = Some(v);
    self
  }

  pub fn runtime(mut self, v: &'a RuntimeSpec) -> Self {
    self.runtime = if v.len() == 1 {
      v.iter().next().map(|v| v.as_ref())
    } else {
      None
    };
    self
  }

  pub fn url(mut self, v: &'a str) -> Self {
    self.url = Some(v);
    self
  }

  pub fn id(mut self, id: &'a str) -> Self {
    self.id = Some(id);
    self
  }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, MergeFrom)]
pub enum PublicPath {
  // TODO: should be RawPublicPath(Filename)
  String(String),
  Auto,
}

impl PublicPath {
  pub fn render(&self, compilation: &Compilation, filename: &str) -> String {
    match self {
      Self::String(s) => Self::ensure_ends_with_slash(s.to_string()),
      Self::Auto => Self::render_auto_public_path(compilation, filename),
    }
  }

  pub fn ensure_ends_with_slash(public_path: String) -> String {
    if !public_path.is_empty() && !public_path.ends_with('/') {
      public_path + "/"
    } else {
      public_path
    }
  }

  pub fn render_auto_public_path(compilation: &Compilation, filename: &str) -> String {
    let public_path = match Path::new(filename).parent() {
      None => "".to_string(),
      Some(dirname) => compilation
        .options
        .output
        .path
        .relative(compilation.options.output.path.join(dirname).absolutize())
        .to_string_lossy()
        .to_string(),
    };
    Self::ensure_ends_with_slash(public_path)
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
  chunk: &'filename Chunk,
  output_options: &'filename OutputOptions,
  chunk_group_by_ukey: &ChunkGroupByUkey,
) -> &'filename Filename {
  // Align with https://github.com/webpack/webpack/blob/8241da7f1e75c5581ba535d127fa66aeb9eb2ac8/lib/css/CssModulesPlugin.js#L444
  if let Some(css_filename_template) = &chunk.css_filename_template {
    css_filename_template
  } else if chunk.can_be_initial(chunk_group_by_ukey) {
    &output_options.css_filename
  } else {
    &output_options.css_chunk_filename
  }
}

#[allow(clippy::if_same_then_else)]
pub fn get_js_chunk_filename_template<'filename>(
  chunk: &'filename Chunk,
  output_options: &'filename OutputOptions,
  chunk_group_by_ukey: &ChunkGroupByUkey,
) -> &'filename Filename {
  // Align with https://github.com/webpack/webpack/blob/8241da7f1e75c5581ba535d127fa66aeb9eb2ac8/lib/javascript/JavascriptModulesPlugin.js#L480
  if let Some(filename_template) = &chunk.filename_template {
    filename_template
  } else if chunk.can_be_initial(chunk_group_by_ukey) {
    &output_options.filename
  } else if matches!(chunk.kind, ChunkKind::HotUpdate) {
    // TODO: Should return output_options.hotUpdateChunkFilename
    // See https://github.com/webpack/webpack/blob/8241da7f1e75c5581ba535d127fa66aeb9eb2ac8/lib/javascript/JavascriptModulesPlugin.js#L484
    &output_options.chunk_filename
  } else {
    &output_options.chunk_filename
  }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct LibraryOptions {
  pub name: Option<LibraryName>,
  pub export: Option<LibraryExport>,
  // webpack type
  pub library_type: LibraryType,
  pub umd_named_define: Option<bool>,
  pub auxiliary_comment: Option<LibraryAuxiliaryComment>,
  pub amd_container: Option<String>,
}

pub type LibraryType = String;

pub type LibraryExport = Vec<String>;

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct LibraryAuxiliaryComment {
  pub root: Option<String>,
  pub commonjs: Option<String>,
  pub commonjs2: Option<String>,
  pub amd: Option<String>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum LibraryName {
  NonUmdObject(LibraryNonUmdObject),
  UmdObject(LibraryCustomUmdObject),
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum LibraryNonUmdObject {
  Array(Vec<String>),
  String(String),
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct LibraryCustomUmdObject {
  pub amd: Option<String>,
  pub commonjs: Option<String>,
  pub root: Option<Vec<String>>,
}

#[derive(Debug)]
pub struct Environment {
  pub arrow_function: Option<bool>,
}

impl Environment {
  pub fn supports_arrow_function(&self) -> bool {
    self.arrow_function.unwrap_or_default()
  }
}
