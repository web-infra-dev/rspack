use std::sync::LazyLock;
use std::{borrow::Cow, fmt::Debug, hash::Hash, str::FromStr, string::ParseError};

use regex::Regex;
use rspack_cacheable::cacheable;
use rspack_hash::RspackHash;
pub use rspack_hash::{HashDigest, HashFunction, HashSalt};
use rspack_macros::MergeFrom;
use rspack_paths::Utf8PathBuf;

use super::CleanOptions;
use crate::{Chunk, ChunkGroupByUkey, ChunkKind, Compilation, Filename, FilenameTemplate};

#[derive(Debug)]
pub enum PathInfo {
  Bool(bool),
  String(String),
}

// BE CAREFUL:
// Add more fields to this struct should result in adding new fields to options builder.
// `impl From<OutputOptions> for OutputOptionsBuilder` should be updated.
#[derive(Debug)]
pub struct OutputOptions {
  pub path: Utf8PathBuf,
  pub pathinfo: PathInfo,
  pub clean: CleanOptions,
  pub public_path: PublicPath,
  pub asset_module_filename: Filename,
  pub wasm_loading: WasmLoading,
  pub webassembly_module_filename: FilenameTemplate,
  pub unique_name: String,
  pub chunk_loading: ChunkLoading,
  pub chunk_loading_global: String,
  pub chunk_load_timeout: u32,
  pub charset: bool,
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
  pub import_meta_name: String,
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
  pub compare_before_emit: bool,
}

impl From<&OutputOptions> for RspackHash {
  fn from(value: &OutputOptions) -> Self {
    Self::with_salt(&value.hash_function, &value.hash_salt)
  }
}

#[derive(Debug)]
pub enum OnPolicyCreationFailure {
  Continue,
  Stop,
}

impl From<String> for OnPolicyCreationFailure {
  fn from(value: String) -> Self {
    if value == "continue" {
      Self::Continue
    } else {
      Self::Stop
    }
  }
}

#[derive(Debug)]
pub struct TrustedTypes {
  pub policy_name: Option<String>,
  pub on_policy_creation_failure: OnPolicyCreationFailure,
}

#[cacheable]
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum ChunkLoading {
  Enable(ChunkLoadingType),
  Disable,
}

impl From<ChunkLoading> for String {
  fn from(value: ChunkLoading) -> Self {
    match value {
      ChunkLoading::Enable(ty) => ty.into(),
      ChunkLoading::Disable => "false".to_string(),
    }
  }
}

impl ChunkLoading {
  pub fn as_str(&self) -> &str {
    match self {
      ChunkLoading::Enable(ty) => ty.as_str(),
      ChunkLoading::Disable => "false",
    }
  }
}

#[cacheable]
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum ChunkLoadingType {
  Jsonp,
  ImportScripts,
  Require,
  AsyncNode,
  Import,
  Custom(String),
}

impl From<&str> for ChunkLoadingType {
  fn from(value: &str) -> Self {
    match value {
      "jsonp" => Self::Jsonp,
      "import-scripts" => Self::ImportScripts,
      "require" => Self::Require,
      "async-node" => Self::AsyncNode,
      "import" => Self::Import,
      _ => Self::Custom(value.to_string()),
    }
  }
}

impl From<ChunkLoadingType> for String {
  fn from(value: ChunkLoadingType) -> Self {
    value.as_str().to_string()
  }
}

impl ChunkLoadingType {
  pub fn as_str(&self) -> &str {
    match self {
      ChunkLoadingType::Jsonp => "jsonp",
      ChunkLoadingType::ImportScripts => "import-scripts",
      ChunkLoadingType::Require => "require",
      ChunkLoadingType::AsyncNode => "async-node",
      ChunkLoadingType::Import => "import",
      ChunkLoadingType::Custom(value) => value.as_str(),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WasmLoadingType {
  Fetch,
  AsyncNode,
}

impl From<&str> for WasmLoadingType {
  fn from(value: &str) -> Self {
    match value {
      "fetch" => Self::Fetch,
      "async-node" => Self::AsyncNode,
      _ => unreachable!("invalid wasm loading type: {value}, expect one of [fetch, async-node]",),
    }
  }
}

#[derive(Debug, Clone)]
pub enum CrossOriginLoading {
  Disable,
  Enable(String),
}

impl std::fmt::Display for CrossOriginLoading {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      CrossOriginLoading::Disable => write!(f, "false"),
      CrossOriginLoading::Enable(value) => write!(f, "\"{value}\""),
    }
  }
}

#[derive(Default, Clone, Copy, Debug)]
pub struct PathData<'a> {
  pub filename: Option<&'a str>,
  pub chunk_name: Option<&'a str>,
  pub chunk_hash: Option<&'a str>,
  pub chunk_id: Option<&'a str>,
  pub module_id: Option<&'a str>,
  pub hash: Option<&'a str>,
  pub content_hash: Option<&'a str>,
  pub runtime: Option<&'a str>,
  pub url: Option<&'a str>,
  pub id: Option<&'a str>,
}

static MATCH_ID_REGEX: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r#"^"\s\+*\s*(.*)\s*\+\s*"$"#).expect("invalid Regex"));
static PREPARE_ID_REGEX: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"(^[.-]|[^a-zA-Z0-9_-])+").expect("invalid Regex"));

impl<'a> PathData<'a> {
  pub fn prepare_id(v: &str) -> Cow<str> {
    if let Some(caps) = MATCH_ID_REGEX.captures(v) {
      Cow::Owned(format!(
        "\" + ({} + \"\").replace(/(^[.-]|[^a-zA-Z0-9_-])+/g, \"_\") + \"",
        caps.get(1).expect("capture group should exist").as_str()
      ))
    } else {
      PREPARE_ID_REGEX.replace_all(v, "_")
    }
  }

  pub fn filename(mut self, v: &'a str) -> Self {
    self.filename = Some(v);
    self
  }

  pub fn chunk_hash(mut self, v: &'a str) -> Self {
    self.chunk_hash = Some(v);
    self
  }

  pub fn chunk_hash_optional(mut self, v: Option<&'a str>) -> Self {
    self.chunk_hash = v;
    self
  }

  pub fn chunk_name(mut self, v: &'a str) -> Self {
    self.chunk_name = Some(v);
    self
  }

  pub fn chunk_name_optional(mut self, v: Option<&'a str>) -> Self {
    self.chunk_name = v;
    self
  }

  pub fn chunk_id(mut self, v: &'a str) -> Self {
    self.chunk_id = Some(v);
    self
  }

  pub fn chunk_id_optional(mut self, v: Option<&'a str>) -> Self {
    self.chunk_id = v;
    self
  }

  pub fn module_id_optional(mut self, v: Option<&'a str>) -> Self {
    self.module_id = v;
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

  pub fn runtime(mut self, v: &'a str) -> Self {
    self.runtime = Some(v);
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

#[cacheable]
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, MergeFrom)]
pub enum PublicPath {
  Filename(Filename),
  Auto,
}

//https://github.com/webpack/webpack/blob/001cab14692eb9a833c6b56709edbab547e291a1/lib/util/identifier.js#L378
pub fn get_undo_path(filename: &str, output_path: String, enforce_relative: bool) -> String {
  let mut depth: i32 = -1;
  let mut append = String::new();
  let mut p = output_path;
  if p.ends_with('/') || p.ends_with('\\') {
    p.pop();
  }
  for part in filename.split(&['/', '\\']) {
    if part == ".." {
      if depth > -1 {
        depth -= 1
      } else {
        let pos = match (p.rfind('/'), p.rfind('\\')) {
          (None, None) => {
            p.push('/');
            return p;
          }
          (None, Some(j)) => j,
          (Some(i), None) => i,
          (Some(i), Some(j)) => usize::max(i, j),
        };
        append = format!("{}/{append}", &p[pos + 1..]);
        p = p[0..pos].to_string();
      }
    } else if part != "." {
      depth += 1;
    }
  }

  if depth > 0 {
    format!("{}{append}", "../".repeat(depth as usize))
  } else if enforce_relative {
    format!("./{append}")
  } else {
    append
  }
}

#[test]
fn test_get_undo_path() {
  assert_eq!(get_undo_path("a", "/a/b/c".to_string(), true), "./");
  assert_eq!(
    get_undo_path("static/js/a.js", "/a/b/c".to_string(), false),
    "../../"
  );
}

impl PublicPath {
  pub fn render(&self, compilation: &Compilation, filename: &str) -> String {
    match self {
      Self::Filename(f) => Self::ensure_ends_with_slash(Self::render_filename(compilation, f)),
      Self::Auto => Self::render_auto_public_path(compilation, filename),
    }
  }

  pub fn render_filename(compilation: &Compilation, template: &Filename) -> String {
    compilation
      .get_path(
        template,
        // @{link https://github.com/webpack/webpack/blob/a642809846deefdb9db05214718af5ab78c0ab94/lib/runtime/PublicPathRuntimeModule.js#L30-L32}
        PathData::default().hash(compilation.get_hash().unwrap_or("XXXX")),
      )
      .expect("failed to render public")
  }

  pub fn ensure_ends_with_slash(public_path: String) -> String {
    if !public_path.is_empty() && !public_path.ends_with('/') {
      public_path + "/"
    } else {
      public_path
    }
  }

  pub fn render_auto_public_path(compilation: &Compilation, filename: &str) -> String {
    let public_path = get_undo_path(filename, compilation.options.output.path.to_string(), false);
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
      Ok(Self::Auto)
    } else {
      Ok(Self::Filename(Filename::from_str(s)?))
    }
  }
}

impl From<String> for PublicPath {
  fn from(value: String) -> Self {
    if value == "auto" {
      Self::Auto
    } else {
      Self::Filename(value.into())
    }
  }
}

pub fn get_css_chunk_filename_template<'filename>(
  chunk: &'filename Chunk,
  output_options: &'filename OutputOptions,
  chunk_group_by_ukey: &ChunkGroupByUkey,
) -> &'filename Filename {
  // Align with https://github.com/webpack/webpack/blob/8241da7f1e75c5581ba535d127fa66aeb9eb2ac8/lib/css/CssModulesPlugin.js#L444
  if let Some(css_filename_template) = chunk.css_filename_template() {
    css_filename_template
  } else if chunk.can_be_initial(chunk_group_by_ukey) {
    &output_options.css_filename
  } else {
    &output_options.css_chunk_filename
  }
}

pub fn get_js_chunk_filename_template(
  chunk: &Chunk,
  output_options: &OutputOptions,
  chunk_group_by_ukey: &ChunkGroupByUkey,
) -> Filename {
  // Align with https://github.com/webpack/webpack/blob/8241da7f1e75c5581ba535d127fa66aeb9eb2ac8/lib/javascript/JavascriptModulesPlugin.js#L480
  if let Some(filename_template) = chunk.filename_template() {
    filename_template.clone()
  } else if matches!(chunk.kind(), ChunkKind::HotUpdate) {
    output_options.hot_update_chunk_filename.clone().into()
  } else if chunk.can_be_initial(chunk_group_by_ukey) {
    output_options.filename.clone()
  } else {
    output_options.chunk_filename.clone()
  }
}

#[cacheable]
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

#[cacheable]
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct LibraryAuxiliaryComment {
  pub root: Option<String>,
  pub commonjs: Option<String>,
  pub commonjs2: Option<String>,
  pub amd: Option<String>,
}

#[cacheable]
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum LibraryName {
  NonUmdObject(LibraryNonUmdObject),
  UmdObject(LibraryCustomUmdObject),
}

#[cacheable]
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum LibraryNonUmdObject {
  Array(Vec<String>),
  String(String),
}

#[cacheable]
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct LibraryCustomUmdObject {
  pub amd: Option<String>,
  pub commonjs: Option<String>,
  pub root: Option<Vec<String>>,
}

#[derive(Debug, Default, Copy, Clone)]
pub struct Environment {
  pub r#const: Option<bool>,
  pub arrow_function: Option<bool>,
  pub node_prefix_for_core_modules: Option<bool>,
  pub async_function: Option<bool>,
  pub big_int_literal: Option<bool>,
  pub destructuring: Option<bool>,
  pub document: Option<bool>,
  pub dynamic_import: Option<bool>,
  pub for_of: Option<bool>,
  pub global_this: Option<bool>,
  pub module: Option<bool>,
  pub optional_chaining: Option<bool>,
  pub template_literal: Option<bool>,
  pub dynamic_import_in_worker: Option<bool>,
}

impl Environment {
  pub fn supports_const(&self) -> bool {
    self.r#const.unwrap_or_default()
  }

  pub fn supports_arrow_function(&self) -> bool {
    self.arrow_function.unwrap_or_default()
  }

  pub fn supports_node_prefix_for_core_modules(&self) -> bool {
    self.node_prefix_for_core_modules.unwrap_or_default()
  }

  pub fn supports_async_function(&self) -> bool {
    self.async_function.unwrap_or_default()
  }

  pub fn supports_big_int_literal(&self) -> bool {
    self.big_int_literal.unwrap_or_default()
  }

  pub fn supports_destructuring(&self) -> bool {
    self.destructuring.unwrap_or_default()
  }

  pub fn supports_document(&self) -> bool {
    self.document.unwrap_or_default()
  }

  pub fn supports_dynamic_import(&self) -> bool {
    self.dynamic_import.unwrap_or_default()
  }

  pub fn supports_dynamic_import_in_worker(&self) -> bool {
    self.dynamic_import_in_worker.unwrap_or_default()
  }

  pub fn supports_for_of(&self) -> bool {
    self.for_of.unwrap_or_default()
  }

  pub fn supports_global_this(&self) -> bool {
    self.global_this.unwrap_or_default()
  }

  pub fn supports_module(&self) -> bool {
    self.module.unwrap_or_default()
  }

  pub fn supports_optional_chaining(&self) -> bool {
    self.optional_chaining.unwrap_or_default()
  }

  pub fn supports_template_literal(&self) -> bool {
    self.template_literal.unwrap_or_default()
  }
}
