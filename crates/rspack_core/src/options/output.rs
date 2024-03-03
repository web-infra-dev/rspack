use std::{
  path::{Path, PathBuf},
  str::FromStr,
  string::ParseError,
};

use derivative::Derivative;
use once_cell::sync::Lazy;
use regex::{Captures, Regex};
use rspack_hash::RspackHash;
pub use rspack_hash::{HashDigest, HashFunction, HashSalt};
use sugar_path::SugarPath;

use crate::{
  parse_resource, AssetInfo, Chunk, ChunkGraph, ChunkGroupByUkey, ChunkKind, Compilation, Module,
  ResourceParsedData, RuntimeSpec,
};

#[derive(Debug)]
pub struct OutputOptions {
  pub path: PathBuf,
  pub pathinfo: bool,
  pub clean: bool,
  pub public_path: PublicPath,
  pub asset_module_filename: Filename,
  pub wasm_loading: WasmLoading,
  pub webassembly_module_filename: Filename,
  pub unique_name: String,
  pub chunk_loading: ChunkLoading,
  pub chunk_loading_global: String,
  pub filename: Filename,
  pub chunk_filename: Filename,
  pub cross_origin_loading: CrossOriginLoading,
  pub css_filename: Filename,
  pub css_chunk_filename: Filename,
  pub hot_update_main_filename: Filename,
  pub hot_update_chunk_filename: Filename,
  pub hot_update_global: String,
  pub library: Option<LibraryOptions>,
  pub enabled_library_types: Option<Vec<String>>,
  pub strict_module_error_handling: bool,
  pub global_object: String,
  pub import_function_name: String,
  pub iife: bool,
  pub module: bool,
  pub trusted_types: Option<TrustedTypes>,
  pub source_map_filename: Filename,
  pub hash_function: HashFunction,
  pub hash_digest: HashDigest,
  pub hash_digest_length: usize,
  pub hash_salt: HashSalt,
  pub async_chunks: bool,
  pub worker_chunk_loading: ChunkLoading,
  pub worker_wasm_loading: WasmLoading,
  pub worker_public_path: String,
  pub script_type: String,
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

pub const FILE_PLACEHOLDER: &str = "[file]";
pub const BASE_PLACEHOLDER: &str = "[base]";
pub const NAME_PLACEHOLDER: &str = "[name]";
pub const PATH_PLACEHOLDER: &str = "[path]";
pub const EXT_PLACEHOLDER: &str = "[ext]";
pub const QUERY_PLACEHOLDER: &str = "[query]";
pub const FRAGMENT_PLACEHOLDER: &str = "[fragment]";
pub const ID_PLACEHOLDER: &str = "[id]";
pub const RUNTIME_PLACEHOLDER: &str = "[runtime]";
pub const URL_PLACEHOLDER: &str = "[url]";
pub static HASH_PLACEHOLDER: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"\[hash(:(\d*))?]").expect("Invalid regex"));
pub static CHUNK_HASH_PLACEHOLDER: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"\[chunkhash(:(\d*))?]").expect("Invalid regex"));
pub static CONTENT_HASH_PLACEHOLDER: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"\[contenthash(:(\d*))?]").expect("Invalid regex"));
pub static FULL_HASH_PLACEHOLDER: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"\[fullhash(:(\d*))?]").expect("Invalid regex"));

static DATA_URI_REGEX: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"^data:([^;,]+)").expect("Invalid regex"));

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

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
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
  pub fn template(&self) -> &str {
    &self.template
  }

  pub fn has_hash_placeholder(&self) -> bool {
    HASH_PLACEHOLDER.is_match(&self.template) || FULL_HASH_PLACEHOLDER.is_match(&self.template)
  }

  pub fn render(&self, options: PathData, mut asset_info: Option<&mut AssetInfo>) -> String {
    let mut template = self.template.clone();
    if let Some(filename) = options.filename {
      if let Some(caps) = DATA_URI_REGEX.captures(filename) {
        let ext = mime_guess::get_mime_extensions_str(
          caps
            .get(1)
            .expect("should match mime for data uri")
            .as_str(),
        )
        .map(|exts| exts[0]);
        template = template.replace(FILE_PLACEHOLDER, "");
        template = template.replace(QUERY_PLACEHOLDER, "");
        template = template.replace(FRAGMENT_PLACEHOLDER, "");
        template = template.replace(PATH_PLACEHOLDER, "");
        template = template.replace(BASE_PLACEHOLDER, "");
        template = template.replace(NAME_PLACEHOLDER, "");
        template = template.replace(
          EXT_PLACEHOLDER,
          &ext.map(|ext| format!(".{}", ext)).unwrap_or_default(),
        );
      } else if let Some(ResourceParsedData {
        path: file,
        query,
        fragment,
      }) = parse_resource(filename)
      {
        template = template.replace(FILE_PLACEHOLDER, &file.to_string_lossy());
        template = template.replace(
          EXT_PLACEHOLDER,
          &file
            .extension()
            .map(|p| format!(".{}", p.to_string_lossy()))
            .unwrap_or_default(),
        );
        if let Some(base) = file.file_name().map(|p| p.to_string_lossy()) {
          template = template.replace(BASE_PLACEHOLDER, &base);
        }
        if let Some(name) = file.file_stem().map(|p| p.to_string_lossy()) {
          template = template.replace(NAME_PLACEHOLDER, &name);
        }
        template = template.replace(
          PATH_PLACEHOLDER,
          &file
            .parent()
            .map(|p| p.to_string_lossy())
            // "" -> "", "folder" -> "folder/"
            .filter(|p| !p.is_empty())
            .map(|p| p + "/")
            .unwrap_or_default(),
        );
        template = template.replace(QUERY_PLACEHOLDER, &query.unwrap_or_default());
        template = template.replace(FRAGMENT_PLACEHOLDER, &fragment.unwrap_or_default());
      }
    }
    if let Some(content_hash) = options.content_hash {
      if let Some(asset_info) = asset_info.as_mut() {
        // set version as content hash
        asset_info.version = content_hash.to_string();
      }
      template = CONTENT_HASH_PLACEHOLDER
        .replace_all(&template, |caps: &Captures| {
          let content_hash = &content_hash[..hash_len(content_hash, caps)];
          if let Some(asset_info) = asset_info.as_mut() {
            asset_info.set_immutable(true);
            asset_info.set_content_hash(content_hash.to_owned());
          }
          content_hash
        })
        .into_owned();
    }
    if let Some(hash) = options.hash {
      for reg in [&HASH_PLACEHOLDER, &FULL_HASH_PLACEHOLDER] {
        template = reg
          .replace_all(&template, |caps: &Captures| {
            let hash = &hash[..hash_len(hash, caps)];
            if let Some(asset_info) = asset_info.as_mut() {
              asset_info.set_immutable(true);
              asset_info.set_content_hash(hash.to_owned());
            }
            hash
          })
          .into_owned();
      }
    }
    if let Some(chunk) = options.chunk {
      if let Some(id) = &options.id {
        template = template.replace(ID_PLACEHOLDER, id);
      } else if let Some(id) = &chunk.id {
        template = template.replace(ID_PLACEHOLDER, &id.to_string());
      }
      if let Some(name) = chunk.name_for_filename_template() {
        template = template.replace(NAME_PLACEHOLDER, name);
      }
      if let Some(d) = chunk.rendered_hash.as_ref() {
        template = CHUNK_HASH_PLACEHOLDER
          .replace_all(&template, |caps: &Captures| {
            let hash = &**d;
            let hash = &hash[..hash_len(hash, caps)];
            if let Some(asset_info) = asset_info.as_mut() {
              asset_info.set_immutable(true);
              asset_info.set_chunk_hash(hash.to_owned());
            }
            hash
          })
          .into_owned();
      }
    }

    if let Some(id) = &options.id {
      template = template.replace(ID_PLACEHOLDER, id);
    } else if let Some(module) = options.module {
      if let Some(chunk_graph) = options.chunk_graph {
        if let Some(id) = chunk_graph.get_module_id(module.identifier()) {
          template = template.replace(ID_PLACEHOLDER, id);
        }
      }
    }
    template = template.replace(RUNTIME_PLACEHOLDER, options.runtime.unwrap_or("_"));
    if let Some(url) = options.url {
      template = template.replace(URL_PLACEHOLDER, url);
    }

    template
  }
}

fn hash_len(hash: &str, caps: &Captures) -> usize {
  let hash_len = hash.len();
  caps
    .get(2)
    .and_then(|m| m.as_str().parse().ok())
    .unwrap_or(hash_len)
    .min(hash_len)
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
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
