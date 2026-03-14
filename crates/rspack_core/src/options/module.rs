use std::{
  fmt,
  ops::{Deref, DerefMut},
  sync::Arc,
};

use async_recursion::async_recursion;
use bitflags::bitflags;
use derive_more::Debug;
use futures::future::BoxFuture;
use rspack_cacheable::{cacheable, with::Unsupported};
use rspack_error::Result;
use rspack_macros::MergeFrom;
use rspack_regex::RspackRegex;
use rspack_util::{MergeFrom, try_any};
use rustc_hash::FxHashMap as HashMap;
use smallvec::SmallVec;
use tokio::sync::OnceCell;

use crate::{Compilation, Filename, Module, ModuleType, PublicPath, Resolve};

#[derive(Debug, Default)]
pub struct ParserOptionsMap(HashMap<String, ParserOptions>);

impl Deref for ParserOptionsMap {
  type Target = HashMap<String, ParserOptions>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl DerefMut for ParserOptionsMap {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

impl FromIterator<(String, ParserOptions)> for ParserOptionsMap {
  fn from_iter<I: IntoIterator<Item = (String, ParserOptions)>>(i: I) -> Self {
    Self(HashMap::from_iter(i))
  }
}

impl ParserOptionsMap {
  pub fn get<'a>(&'a self, key: &'a str) -> Option<&'a ParserOptions> {
    self.0.get(key)
  }
}

#[cacheable]
#[derive(Debug, Clone, MergeFrom)]
pub enum ParserOptions {
  Asset(AssetParserOptions),
  Css(CssParserOptions),
  CssAuto(CssAutoParserOptions),
  CssModule(CssModuleParserOptions),
  Javascript(JavascriptParserOptions),
  JavascriptAuto(JavascriptParserOptions),
  JavascriptEsm(JavascriptParserOptions),
  JavascriptDynamic(JavascriptParserOptions),
  Json(JsonParserOptions),
  Unknown,
}

macro_rules! get_variant {
  ($fn_name:ident, $variant:ident, $ret_ty:ident) => {
    pub fn $fn_name(&self) -> Option<&$ret_ty> {
      match self {
        Self::$variant(value) => Some(value),
        _ => None,
      }
    }
  };
}

impl ParserOptions {
  get_variant!(get_asset, Asset, AssetParserOptions);
  get_variant!(get_css, Css, CssParserOptions);
  get_variant!(get_css_auto, CssAuto, CssAutoParserOptions);
  get_variant!(get_css_module, CssModule, CssModuleParserOptions);
  get_variant!(get_javascript, Javascript, JavascriptParserOptions);
  get_variant!(get_javascript_auto, JavascriptAuto, JavascriptParserOptions);
  get_variant!(get_javascript_esm, JavascriptEsm, JavascriptParserOptions);
  get_variant!(
    get_javascript_dynamic,
    JavascriptDynamic,
    JavascriptParserOptions
  );
  get_variant!(get_json, Json, JsonParserOptions);
}

#[cacheable]
#[derive(Debug, Clone, Copy, MergeFrom)]
pub enum DynamicImportMode {
  Lazy,
  Weak,
  Eager,
  LazyOnce,
}

impl From<&str> for DynamicImportMode {
  fn from(value: &str) -> Self {
    match value {
      "weak" => DynamicImportMode::Weak,
      "eager" => DynamicImportMode::Eager,
      "lazy" => DynamicImportMode::Lazy,
      "lazy-once" => DynamicImportMode::LazyOnce,
      _ => {
        // TODO: warning
        DynamicImportMode::Lazy
      }
    }
  }
}

#[cacheable]
#[derive(Debug, Clone, Copy, MergeFrom, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DynamicImportFetchPriority {
  Low,
  High,
  Auto,
}

impl From<&str> for DynamicImportFetchPriority {
  fn from(value: &str) -> Self {
    match value {
      "low" => DynamicImportFetchPriority::Low,
      "high" => DynamicImportFetchPriority::High,
      "auto" => DynamicImportFetchPriority::Auto,
      _ => DynamicImportFetchPriority::Auto,
    }
  }
}

impl fmt::Display for DynamicImportFetchPriority {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      DynamicImportFetchPriority::Low => write!(f, "low"),
      DynamicImportFetchPriority::High => write!(f, "high"),
      DynamicImportFetchPriority::Auto => write!(f, "auto"),
    }
  }
}

#[cacheable]
#[derive(Debug, Clone, Copy, MergeFrom)]
pub enum JavascriptParserUrl {
  Enable,
  Disable,
  NewUrlRelative,
  Relative,
}

impl From<&str> for JavascriptParserUrl {
  fn from(value: &str) -> Self {
    match value {
      "false" => Self::Disable,
      "relative" => Self::Relative,
      "new-url-relative" => Self::NewUrlRelative,
      _ => Self::Enable,
    }
  }
}

#[cacheable]
#[derive(Debug, Clone, Copy, MergeFrom)]
pub enum JavascriptParserOrder {
  Disable,
  Order(i32),
}

impl JavascriptParserOrder {
  pub fn get_order(&self) -> Option<i32> {
    match self {
      Self::Disable => None,
      Self::Order(o) => Some(*o),
    }
  }
}

impl From<&str> for JavascriptParserOrder {
  fn from(value: &str) -> Self {
    match value {
      "false" => Self::Disable,
      "true" => Self::Order(0),
      _ => {
        if let Ok(order) = value.parse::<i32>() {
          Self::Order(order)
        } else {
          Self::Order(0)
        }
      }
    }
  }
}

#[cacheable]
#[derive(Debug, Clone, Copy, MergeFrom, PartialEq, Eq)]
pub enum JavascriptParserCommonjsExportsOption {
  Enable,
  Disable,
  SkipInEsm,
}

impl From<bool> for JavascriptParserCommonjsExportsOption {
  fn from(value: bool) -> Self {
    if value { Self::Enable } else { Self::Disable }
  }
}

#[cacheable]
#[derive(Debug, Clone, MergeFrom)]
pub struct JavascriptParserCommonjsOptions {
  pub exports: JavascriptParserCommonjsExportsOption,
}

#[cacheable]
#[derive(Debug, Clone, Copy, MergeFrom)]
pub enum ExportPresenceMode {
  None,
  Warn,
  Auto,
  Error,
}

impl From<&str> for ExportPresenceMode {
  fn from(value: &str) -> Self {
    match value {
      "false" => Self::None,
      "warn" => Self::Warn,
      "error" => Self::Error,
      _ => Self::Auto,
    }
  }
}

impl ExportPresenceMode {
  pub fn get_effective_export_presence(&self, module: &dyn Module) -> Option<bool> {
    match self {
      ExportPresenceMode::None => None,
      ExportPresenceMode::Warn => Some(false),
      ExportPresenceMode::Error => Some(true),
      ExportPresenceMode::Auto => Some(module.build_meta().strict_esm_module),
    }
  }
}

#[cacheable]
#[derive(Debug, Default, Clone, Copy, MergeFrom)]
pub enum TypeReexportPresenceMode {
  #[default]
  NoTolerant,
  Tolerant,
  TolerantNoCheck,
}

impl From<&str> for TypeReexportPresenceMode {
  fn from(value: &str) -> Self {
    match value {
      "tolerant" => Self::Tolerant,
      "tolerant-no-check" => Self::TolerantNoCheck,
      _ => Self::NoTolerant,
    }
  }
}

#[cacheable]
#[derive(Debug, Clone, Copy, MergeFrom)]
pub enum OverrideStrict {
  Strict,
  NoneStrict,
}

impl From<&str> for OverrideStrict {
  fn from(value: &str) -> Self {
    match value {
      "strict" => Self::Strict,
      "non-strict" => Self::NoneStrict,
      _ => unreachable!("parser.overrideStrict should be 'strict' or 'non-strict'"),
    }
  }
}

#[cacheable]
#[derive(Debug, Clone, Copy, MergeFrom)]
pub enum ImportMeta {
  PreserveUnknown,
  Enabled,
  Disabled,
}

impl From<&str> for ImportMeta {
  fn from(value: &str) -> Self {
    match value {
      "preserve-unknown" => Self::PreserveUnknown,
      "false" => Self::Disabled,
      _ => Self::Enabled,
    }
  }
}

#[cacheable]
#[derive(Debug, Clone, MergeFrom, Default)]
pub struct JavascriptParserOptions {
  pub dynamic_import_mode: Option<DynamicImportMode>,
  pub dynamic_import_preload: Option<JavascriptParserOrder>,
  pub dynamic_import_prefetch: Option<JavascriptParserOrder>,
  pub dynamic_import_fetch_priority: Option<DynamicImportFetchPriority>,
  pub url: Option<JavascriptParserUrl>,
  pub unknown_context_critical: Option<bool>,
  pub expr_context_critical: Option<bool>,
  pub wrapped_context_critical: Option<bool>,
  pub strict_this_context_on_imports: Option<bool>,
  pub wrapped_context_reg_exp: Option<RspackRegex>,
  pub exports_presence: Option<ExportPresenceMode>,
  pub import_exports_presence: Option<ExportPresenceMode>,
  pub reexport_exports_presence: Option<ExportPresenceMode>,
  pub type_reexports_presence: Option<TypeReexportPresenceMode>,
  pub worker: Option<Vec<String>>,
  pub override_strict: Option<OverrideStrict>,
  pub import_meta: Option<ImportMeta>,
  pub require_alias: Option<bool>,
  pub require_as_expression: Option<bool>,
  pub require_dynamic: Option<bool>,
  pub require_resolve: Option<bool>,
  pub commonjs: Option<JavascriptParserCommonjsOptions>,
  pub import_dynamic: Option<bool>,
  pub commonjs_magic_comments: Option<bool>,
  pub jsx: Option<bool>,
  pub defer_import: Option<bool>,
}

#[cacheable]
#[derive(Debug, Clone, MergeFrom)]
pub struct AssetParserOptions {
  pub data_url_condition: Option<AssetParserDataUrl>,
}

#[cacheable]
#[derive(Debug, Clone, MergeFrom)]
pub enum AssetParserDataUrl {
  Options(AssetParserDataUrlOptions),
  // TODO: Function
}

#[cacheable]
#[derive(Debug, Clone, MergeFrom)]
pub struct AssetParserDataUrlOptions {
  pub max_size: Option<f64>,
}

/// Context passed to the CSS parser import filter function
pub struct CssParserImportContext {
  pub url: String,
  pub media: Option<String>,
  pub resource_path: String,
  pub supports: Option<String>,
  pub layer: Option<String>,
}

pub type CssParserImportFn =
  Arc<dyn Fn(CssParserImportContext) -> BoxFuture<'static, Result<bool>> + Sync + Send>;

#[cacheable]
pub enum CssParserImport {
  Bool(bool),
  Func(#[cacheable(with=Unsupported)] CssParserImportFn),
}

impl Default for CssParserImport {
  fn default() -> Self {
    Self::Bool(true)
  }
}

impl fmt::Debug for CssParserImport {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Bool(b) => write!(f, "CssParserImport::Bool({b})"),
      Self::Func(_) => write!(f, "CssParserImport::Func(...)"),
    }
  }
}

impl Clone for CssParserImport {
  fn clone(&self) -> Self {
    match self {
      Self::Bool(b) => Self::Bool(*b),
      Self::Func(f) => Self::Func(f.clone()),
    }
  }
}

impl MergeFrom for CssParserImport {
  fn merge_from(self, other: &Self) -> Self {
    other.clone()
  }
}

#[cacheable]
#[derive(Debug, Clone, MergeFrom)]
pub struct CssParserOptions {
  pub named_exports: Option<bool>,
  pub url: Option<bool>,
  pub resolve_import: Option<CssParserImport>,
}

#[cacheable]
#[derive(Debug, Clone, MergeFrom)]
pub struct CssAutoParserOptions {
  pub named_exports: Option<bool>,
  pub url: Option<bool>,
  pub resolve_import: Option<CssParserImport>,
}

impl From<CssParserOptions> for CssAutoParserOptions {
  fn from(value: CssParserOptions) -> Self {
    Self {
      named_exports: value.named_exports,
      url: value.url,
      resolve_import: value.resolve_import,
    }
  }
}

#[cacheable]
#[derive(Debug, Clone, MergeFrom)]
pub struct CssModuleParserOptions {
  pub named_exports: Option<bool>,
  pub url: Option<bool>,
  pub resolve_import: Option<CssParserImport>,
}

impl From<CssParserOptions> for CssModuleParserOptions {
  fn from(value: CssParserOptions) -> Self {
    Self {
      named_exports: value.named_exports,
      url: value.url,
      resolve_import: value.resolve_import,
    }
  }
}

pub type JsonParseFn = Arc<dyn Fn(String) -> BoxFuture<'static, Result<String>> + Sync + Send>;

#[cacheable]
pub enum ParseOption {
  Func(#[cacheable(with=Unsupported)] JsonParseFn),
  None,
}

impl fmt::Debug for ParseOption {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Func(_) => write!(f, "ParseOption::Func(...)"),
      _ => write!(f, "ParseOption::None"),
    }
  }
}

impl Clone for ParseOption {
  fn clone(&self) -> Self {
    match self {
      Self::Func(f) => Self::Func(f.clone()),
      Self::None => Self::None,
    }
  }
}

impl MergeFrom for ParseOption {
  fn merge_from(self, other: &Self) -> Self {
    other.clone()
  }
}

#[cacheable]
#[derive(Debug, Clone, MergeFrom)]
pub struct JsonParserOptions {
  pub exports_depth: Option<u32>,
  pub parse: ParseOption,
}

#[derive(Debug, Default)]
pub struct GeneratorOptionsMap(HashMap<String, GeneratorOptions>);

impl Deref for GeneratorOptionsMap {
  type Target = HashMap<String, GeneratorOptions>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl DerefMut for GeneratorOptionsMap {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

impl FromIterator<(String, GeneratorOptions)> for GeneratorOptionsMap {
  fn from_iter<I: IntoIterator<Item = (String, GeneratorOptions)>>(i: I) -> Self {
    Self(HashMap::from_iter(i))
  }
}

impl GeneratorOptionsMap {
  pub fn get(&self, key: &str) -> Option<&GeneratorOptions> {
    self.0.get(key)
  }
}

#[cacheable]
#[derive(Debug, Clone, MergeFrom)]
pub enum GeneratorOptions {
  Asset(AssetGeneratorOptions),
  AssetInline(AssetInlineGeneratorOptions),
  AssetResource(AssetResourceGeneratorOptions),
  Css(CssGeneratorOptions),
  CssAuto(CssAutoGeneratorOptions),
  CssModule(CssModuleGeneratorOptions),
  Json(JsonGeneratorOptions),
  Unknown,
}

impl GeneratorOptions {
  get_variant!(get_asset, Asset, AssetGeneratorOptions);
  get_variant!(get_asset_inline, AssetInline, AssetInlineGeneratorOptions);
  get_variant!(
    get_asset_resource,
    AssetResource,
    AssetResourceGeneratorOptions
  );
  get_variant!(get_css, Css, CssGeneratorOptions);
  get_variant!(get_css_auto, CssAuto, CssAutoGeneratorOptions);
  get_variant!(get_css_module, CssModule, CssModuleGeneratorOptions);
  get_variant!(get_json, Json, JsonGeneratorOptions);

  pub fn asset_filename(&self) -> Option<&Filename> {
    self
      .get_asset()
      .and_then(|x| x.filename.as_ref())
      .or_else(|| self.get_asset_resource().and_then(|x| x.filename.as_ref()))
  }

  pub fn asset_output_path(&self) -> Option<&Filename> {
    self
      .get_asset()
      .and_then(|x| x.output_path.as_ref())
      .or_else(|| {
        self
          .get_asset_resource()
          .and_then(|x| x.output_path.as_ref())
      })
  }

  pub fn asset_public_path(&self) -> Option<&PublicPath> {
    self
      .get_asset()
      .and_then(|x| x.public_path.as_ref())
      .or_else(|| {
        self
          .get_asset_resource()
          .and_then(|x| x.public_path.as_ref())
      })
  }

  pub fn asset_data_url(&self) -> Option<&AssetGeneratorDataUrl> {
    self
      .get_asset()
      .and_then(|x| x.data_url.as_ref())
      .or_else(|| self.get_asset_inline().and_then(|x| x.data_url.as_ref()))
  }
}

#[cacheable]
#[derive(Debug, Clone, MergeFrom)]
pub struct AssetInlineGeneratorOptions {
  pub data_url: Option<AssetGeneratorDataUrl>,
  pub binary: Option<bool>,
}

impl From<AssetGeneratorOptions> for AssetInlineGeneratorOptions {
  fn from(value: AssetGeneratorOptions) -> Self {
    Self {
      data_url: value.data_url,
      binary: value.binary,
    }
  }
}

#[cacheable]
#[derive(Debug, Clone, Copy, MergeFrom)]
struct AssetGeneratorImportModeFlags(u8);
bitflags! {
  impl AssetGeneratorImportModeFlags: u8 {
    const URL = 1 << 0;
    const PRESERVE = 1 << 1;
  }
}

#[cacheable]
#[derive(Debug, Clone, Copy, MergeFrom)]
pub struct AssetGeneratorImportMode(AssetGeneratorImportModeFlags);

impl AssetGeneratorImportMode {
  pub fn is_url(&self) -> bool {
    self.0.contains(AssetGeneratorImportModeFlags::URL)
  }
  pub fn is_preserve(&self) -> bool {
    self.0.contains(AssetGeneratorImportModeFlags::PRESERVE)
  }
}

impl From<String> for AssetGeneratorImportMode {
  fn from(s: String) -> Self {
    match s.as_str() {
      "url" => Self(AssetGeneratorImportModeFlags::URL),
      "preserve" => Self(AssetGeneratorImportModeFlags::PRESERVE),
      _ => unreachable!("AssetGeneratorImportMode error"),
    }
  }
}

impl Default for AssetGeneratorImportMode {
  fn default() -> Self {
    Self(AssetGeneratorImportModeFlags::URL)
  }
}

#[cacheable]
#[derive(Debug, Clone, MergeFrom)]
pub struct AssetResourceGeneratorOptions {
  pub emit: Option<bool>,
  pub filename: Option<Filename>,
  pub output_path: Option<Filename>,
  pub public_path: Option<PublicPath>,
  pub import_mode: Option<AssetGeneratorImportMode>,
  pub binary: Option<bool>,
}

impl From<AssetGeneratorOptions> for AssetResourceGeneratorOptions {
  fn from(value: AssetGeneratorOptions) -> Self {
    Self {
      emit: value.emit,
      filename: value.filename,
      output_path: value.output_path,
      public_path: value.public_path,
      import_mode: value.import_mode,
      binary: value.binary,
    }
  }
}

#[cacheable]
#[derive(Debug, Clone, MergeFrom)]
pub struct AssetGeneratorOptions {
  pub emit: Option<bool>,
  pub filename: Option<Filename>,
  pub output_path: Option<Filename>,
  pub public_path: Option<PublicPath>,
  pub data_url: Option<AssetGeneratorDataUrl>,
  pub import_mode: Option<AssetGeneratorImportMode>,
  pub binary: Option<bool>,
}

pub struct AssetGeneratorDataUrlFnCtx<'a> {
  pub filename: String,
  pub module: &'a dyn Module,
  pub compilation: &'a Compilation,
}

pub type AssetGeneratorDataUrlFn = Arc<
  dyn Fn(Vec<u8>, AssetGeneratorDataUrlFnCtx) -> BoxFuture<'static, Result<String>> + Sync + Send,
>;

#[cacheable]
pub enum AssetGeneratorDataUrl {
  Options(AssetGeneratorDataUrlOptions),
  Func(#[cacheable(with=Unsupported)] AssetGeneratorDataUrlFn),
}

impl fmt::Debug for AssetGeneratorDataUrl {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Options(i) => i.fmt(f),
      Self::Func(_) => "Func(...)".fmt(f),
    }
  }
}

impl Clone for AssetGeneratorDataUrl {
  fn clone(&self) -> Self {
    match self {
      Self::Options(i) => Self::Options(i.clone()),
      Self::Func(i) => Self::Func(i.clone()),
    }
  }
}

impl MergeFrom for AssetGeneratorDataUrl {
  fn merge_from(self, other: &Self) -> Self {
    other.clone()
  }
}

#[cacheable]
#[derive(Debug, Clone, MergeFrom, Hash)]
pub struct AssetGeneratorDataUrlOptions {
  pub encoding: Option<DataUrlEncoding>,
  pub mimetype: Option<String>,
}

#[cacheable]
#[derive(Debug, Clone, MergeFrom, Hash)]
pub enum DataUrlEncoding {
  None,
  Base64,
}

impl fmt::Display for DataUrlEncoding {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      DataUrlEncoding::None => write!(f, ""),
      DataUrlEncoding::Base64 => write!(f, "base64"),
    }
  }
}

impl From<String> for DataUrlEncoding {
  fn from(value: String) -> Self {
    match value.as_str() {
      "base64" => Self::Base64,
      "false" => Self::None,
      _ => unreachable!("DataUrlEncoding should be base64 or false"),
    }
  }
}

#[cacheable]
#[derive(Debug, Clone, MergeFrom)]
pub struct CssGeneratorOptions {
  pub exports_only: Option<bool>,
  pub es_module: Option<bool>,
}

#[cacheable]
#[derive(Default, Debug, Clone, MergeFrom)]
pub struct CssAutoGeneratorOptions {
  pub exports_convention: Option<CssExportsConvention>,
  pub exports_only: Option<bool>,
  pub local_ident_name: Option<LocalIdentName>,
  pub es_module: Option<bool>,
}

impl From<CssGeneratorOptions> for CssAutoGeneratorOptions {
  fn from(value: CssGeneratorOptions) -> Self {
    Self {
      exports_only: value.exports_only,
      es_module: value.es_module,
      ..Default::default()
    }
  }
}

#[cacheable]
#[derive(Default, Debug, Clone, MergeFrom)]
pub struct CssModuleGeneratorOptions {
  pub exports_convention: Option<CssExportsConvention>,
  pub exports_only: Option<bool>,
  pub local_ident_name: Option<LocalIdentName>,
  pub es_module: Option<bool>,
}

impl From<CssGeneratorOptions> for CssModuleGeneratorOptions {
  fn from(value: CssGeneratorOptions) -> Self {
    Self {
      exports_only: value.exports_only,
      es_module: value.es_module,
      ..Default::default()
    }
  }
}

#[cacheable]
#[derive(Default, Debug, Clone, MergeFrom)]
pub struct JsonGeneratorOptions {
  pub json_parse: Option<bool>,
}

#[cacheable]
#[derive(Debug, Clone, MergeFrom)]
pub struct LocalIdentName {
  pub template: Filename,
}

impl From<String> for LocalIdentName {
  fn from(value: String) -> Self {
    Self {
      template: value.into(),
    }
  }
}

impl From<&str> for LocalIdentName {
  fn from(value: &str) -> Self {
    Self {
      template: crate::Filename::from(value),
    }
  }
}

#[cacheable]
#[derive(Debug, Clone, Copy)]
struct ExportsConventionFlags(u8);
bitflags! {
  impl ExportsConventionFlags: u8 {
    const ASIS = 1 << 0;
    const CAMELCASE = 1 << 1;
    const DASHES = 1 << 2;
  }
}

impl MergeFrom for ExportsConventionFlags {
  fn merge_from(self, other: &Self) -> Self {
    *other
  }
}

#[cacheable]
#[derive(Debug, Clone, Copy, MergeFrom)]
pub struct CssExportsConvention(ExportsConventionFlags);

impl CssExportsConvention {
  pub fn as_is(&self) -> bool {
    self.0.contains(ExportsConventionFlags::ASIS)
  }

  pub fn camel_case(&self) -> bool {
    self.0.contains(ExportsConventionFlags::CAMELCASE)
  }

  pub fn dashes(&self) -> bool {
    self.0.contains(ExportsConventionFlags::DASHES)
  }
}

impl From<String> for CssExportsConvention {
  fn from(s: String) -> Self {
    match s.as_str() {
      "as-is" => Self(ExportsConventionFlags::ASIS),
      "camel-case" => Self(ExportsConventionFlags::ASIS | ExportsConventionFlags::CAMELCASE),
      "camel-case-only" => Self(ExportsConventionFlags::CAMELCASE),
      "dashes" => Self(ExportsConventionFlags::ASIS | ExportsConventionFlags::DASHES),
      "dashes-only" => Self(ExportsConventionFlags::DASHES),
      _ => unreachable!("css exportsConventions error"),
    }
  }
}

impl Default for CssExportsConvention {
  fn default() -> Self {
    Self(ExportsConventionFlags::ASIS)
  }
}

pub type DescriptionData = HashMap<String, RuleSetConditionWithEmpty>;
pub type With = HashMap<String, RuleSetConditionWithEmpty>;

pub type RuleSetConditionFnMatcher =
  Box<dyn Fn(DataRef) -> BoxFuture<'static, Result<bool>> + Sync + Send>;

pub enum RuleSetCondition {
  String(String),
  Regexp(RspackRegex),
  Logical(Box<RuleSetLogicalConditions>),
  Array(Vec<RuleSetCondition>),
  Func(RuleSetConditionFnMatcher),
}

impl fmt::Debug for RuleSetCondition {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::String(i) => i.fmt(f),
      Self::Regexp(i) => i.fmt(f),
      Self::Logical(i) => i.fmt(f),
      Self::Array(i) => i.fmt(f),
      Self::Func(_) => "Func(...)".fmt(f),
    }
  }
}

#[derive(Copy, Clone)]
pub enum DataRef<'a> {
  Str(&'a str),
  Value(&'a serde_json::Value),
}

impl<'s> From<&'s str> for DataRef<'s> {
  fn from(value: &'s str) -> Self {
    Self::Str(value)
  }
}

impl<'s> From<&'s serde_json::Value> for DataRef<'s> {
  fn from(value: &'s serde_json::Value) -> Self {
    Self::Value(value)
  }
}

impl DataRef<'_> {
  pub fn as_str(&self) -> Option<&str> {
    match self {
      Self::Str(s) => Some(s),
      Self::Value(v) => v.as_str(),
    }
  }

  pub fn to_value(&self) -> serde_json::Value {
    match self {
      Self::Str(s) => serde_json::Value::String((*s).to_owned()),
      Self::Value(v) => (*v).to_owned(),
    }
  }
}

enum ConditionMatchRoot<'a> {
  Condition(&'a RuleSetCondition),
  Logical(&'a RuleSetLogicalConditions),
}

enum ConditionMatchState<'a> {
  Condition(&'a RuleSetCondition),
  Array {
    conditions: &'a [RuleSetCondition],
    index: usize,
    matched_any: bool,
  },
  Logical {
    logical: &'a RuleSetLogicalConditions,
    stage: LogicalMatchStage,
  },
}

enum LogicalMatchStage {
  Start,
  And { index: usize },
  OrStart,
  Or { index: usize, matched_any: bool },
  NotStart,
  NotDone,
}

async fn try_match_condition_impl<'a, 'data>(
  root: ConditionMatchRoot<'a>,
  data: DataRef<'data>,
) -> Result<bool> {
  let mut stack = SmallVec::<[ConditionMatchState<'a>; 8]>::with_capacity(8);
  let mut last_result = None;

  match root {
    ConditionMatchRoot::Condition(condition) => {
      stack.push(ConditionMatchState::Condition(condition))
    }
    ConditionMatchRoot::Logical(logical) => stack.push(ConditionMatchState::Logical {
      logical,
      stage: LogicalMatchStage::Start,
    }),
  }

  while let Some(state) = stack.pop() {
    match state {
      ConditionMatchState::Condition(condition) => match condition {
        RuleSetCondition::String(s) => {
          last_result = Some(
            data
              .as_str()
              .map(|data| data.starts_with(s))
              .unwrap_or_default(),
          );
        }
        RuleSetCondition::Regexp(r) => {
          last_result = Some(data.as_str().map(|data| r.test(data)).unwrap_or_default());
        }
        RuleSetCondition::Logical(logical) => stack.push(ConditionMatchState::Logical {
          logical,
          stage: LogicalMatchStage::Start,
        }),
        RuleSetCondition::Array(conditions) => {
          if conditions.is_empty() {
            last_result = Some(false);
          } else {
            stack.push(ConditionMatchState::Array {
              conditions,
              index: 0,
              matched_any: false,
            });
          }
        }
        RuleSetCondition::Func(func) => {
          last_result = Some(func(data).await?);
        }
      },
      ConditionMatchState::Array {
        conditions,
        index,
        matched_any,
      } => {
        let matched_any = if index == 0 {
          matched_any
        } else {
          matched_any
            || last_result
              .take()
              .expect("condition array evaluation should produce a result")
        };

        if matched_any {
          last_result = Some(true);
          continue;
        }

        if let Some(condition) = conditions.get(index) {
          stack.push(ConditionMatchState::Array {
            conditions,
            index: index + 1,
            matched_any,
          });
          stack.push(ConditionMatchState::Condition(condition));
        } else {
          last_result = Some(false);
        }
      }
      ConditionMatchState::Logical { logical, stage } => match stage {
        LogicalMatchStage::Start => {
          if let Some(and) = logical.and.as_deref()
            && let Some(condition) = and.first()
          {
            stack.push(ConditionMatchState::Logical {
              logical,
              stage: LogicalMatchStage::And { index: 1 },
            });
            stack.push(ConditionMatchState::Condition(condition));
          } else {
            stack.push(ConditionMatchState::Logical {
              logical,
              stage: LogicalMatchStage::OrStart,
            });
          }
        }
        LogicalMatchStage::And { index } => {
          if !last_result
            .take()
            .expect("logical and evaluation should produce a result")
          {
            last_result = Some(false);
            continue;
          }

          if let Some(condition) = logical.and.as_deref().and_then(|and| and.get(index)) {
            stack.push(ConditionMatchState::Logical {
              logical,
              stage: LogicalMatchStage::And { index: index + 1 },
            });
            stack.push(ConditionMatchState::Condition(condition));
          } else {
            stack.push(ConditionMatchState::Logical {
              logical,
              stage: LogicalMatchStage::OrStart,
            });
          }
        }
        LogicalMatchStage::OrStart => {
          if let Some(or) = logical.or.as_deref() {
            if let Some(condition) = or.first() {
              stack.push(ConditionMatchState::Logical {
                logical,
                stage: LogicalMatchStage::Or {
                  index: 1,
                  matched_any: false,
                },
              });
              stack.push(ConditionMatchState::Condition(condition));
            } else {
              last_result = Some(false);
            }
          } else {
            stack.push(ConditionMatchState::Logical {
              logical,
              stage: LogicalMatchStage::NotStart,
            });
          }
        }
        LogicalMatchStage::Or { index, matched_any } => {
          let matched_any = matched_any
            || last_result
              .take()
              .expect("logical or evaluation should produce a result");

          if matched_any {
            stack.push(ConditionMatchState::Logical {
              logical,
              stage: LogicalMatchStage::NotStart,
            });
            continue;
          }

          if let Some(condition) = logical.or.as_deref().and_then(|or| or.get(index)) {
            stack.push(ConditionMatchState::Logical {
              logical,
              stage: LogicalMatchStage::Or {
                index: index + 1,
                matched_any,
              },
            });
            stack.push(ConditionMatchState::Condition(condition));
          } else {
            last_result = Some(false);
          }
        }
        LogicalMatchStage::NotStart => {
          if let Some(not) = logical.not.as_ref() {
            stack.push(ConditionMatchState::Logical {
              logical,
              stage: LogicalMatchStage::NotDone,
            });
            stack.push(ConditionMatchState::Condition(not));
          } else {
            last_result = Some(true);
          }
        }
        LogicalMatchStage::NotDone => {
          last_result = Some(
            !last_result
              .take()
              .expect("logical not evaluation should produce a result"),
          );
        }
      },
    }
  }

  Ok(last_result.expect("condition evaluation should always finish with a result"))
}

enum ConditionEmptyRoot<'a> {
  Condition(&'a RuleSetCondition),
  Logical(&'a RuleSetLogicalConditions),
}

enum ConditionEmptyState<'a> {
  Condition(&'a RuleSetCondition),
  Logical {
    logical: &'a RuleSetLogicalConditions,
    stage: LogicalEmptyStage,
  },
}

enum LogicalEmptyStage {
  Start,
  And {
    index: usize,
    has_condition: bool,
    matched: bool,
  },
  OrStart {
    has_condition: bool,
    matched: bool,
  },
  Or {
    index: usize,
    has_condition: bool,
    matched: bool,
    matched_any: bool,
  },
  NotStart {
    has_condition: bool,
    matched: bool,
  },
  NotDone {
    has_condition: bool,
    matched: bool,
  },
}

async fn match_when_empty_condition_impl(root: ConditionEmptyRoot<'_>) -> Result<bool> {
  let mut stack = SmallVec::<[ConditionEmptyState<'_>; 8]>::with_capacity(8);
  let mut last_result = None;

  match root {
    ConditionEmptyRoot::Condition(condition) => {
      stack.push(ConditionEmptyState::Condition(condition))
    }
    ConditionEmptyRoot::Logical(logical) => stack.push(ConditionEmptyState::Logical {
      logical,
      stage: LogicalEmptyStage::Start,
    }),
  }

  while let Some(state) = stack.pop() {
    match state {
      ConditionEmptyState::Condition(condition) => match condition {
        RuleSetCondition::String(s) => last_result = Some(s.is_empty()),
        RuleSetCondition::Regexp(regex) => last_result = Some(regex.test("")),
        RuleSetCondition::Logical(logical) => stack.push(ConditionEmptyState::Logical {
          logical,
          stage: LogicalEmptyStage::Start,
        }),
        RuleSetCondition::Array(_) => last_result = Some(false),
        RuleSetCondition::Func(func) => last_result = Some(func("".into()).await?),
      },
      ConditionEmptyState::Logical { logical, stage } => match stage {
        LogicalEmptyStage::Start => {
          let has_condition = logical.and.is_some();
          if let Some(and) = logical.and.as_deref()
            && let Some(condition) = and.first()
          {
            stack.push(ConditionEmptyState::Logical {
              logical,
              stage: LogicalEmptyStage::And {
                index: 1,
                has_condition,
                matched: true,
              },
            });
            stack.push(ConditionEmptyState::Condition(condition));
          } else {
            stack.push(ConditionEmptyState::Logical {
              logical,
              stage: LogicalEmptyStage::OrStart {
                has_condition,
                matched: true,
              },
            });
          }
        }
        LogicalEmptyStage::And {
          index,
          has_condition,
          matched,
        } => {
          let matched = matched
            && last_result
              .take()
              .expect("logical empty and evaluation should produce a result");

          if matched {
            if let Some(condition) = logical.and.as_deref().and_then(|and| and.get(index)) {
              stack.push(ConditionEmptyState::Logical {
                logical,
                stage: LogicalEmptyStage::And {
                  index: index + 1,
                  has_condition,
                  matched,
                },
              });
              stack.push(ConditionEmptyState::Condition(condition));
            } else {
              stack.push(ConditionEmptyState::Logical {
                logical,
                stage: LogicalEmptyStage::OrStart {
                  has_condition,
                  matched,
                },
              });
            }
          } else {
            stack.push(ConditionEmptyState::Logical {
              logical,
              stage: LogicalEmptyStage::OrStart {
                has_condition,
                matched,
              },
            });
          }
        }
        LogicalEmptyStage::OrStart {
          has_condition,
          matched,
        } => {
          if let Some(or) = logical.or.as_deref() {
            let has_condition = true;
            if let Some(condition) = or.first() {
              stack.push(ConditionEmptyState::Logical {
                logical,
                stage: LogicalEmptyStage::Or {
                  index: 1,
                  has_condition,
                  matched,
                  matched_any: false,
                },
              });
              stack.push(ConditionEmptyState::Condition(condition));
            } else {
              stack.push(ConditionEmptyState::Logical {
                logical,
                stage: LogicalEmptyStage::NotStart {
                  has_condition,
                  matched: false,
                },
              });
            }
          } else {
            stack.push(ConditionEmptyState::Logical {
              logical,
              stage: LogicalEmptyStage::NotStart {
                has_condition,
                matched,
              },
            });
          }
        }
        LogicalEmptyStage::Or {
          index,
          has_condition,
          matched,
          matched_any,
        } => {
          let matched_any = matched_any
            || last_result
              .take()
              .expect("logical empty or evaluation should produce a result");

          if matched_any {
            stack.push(ConditionEmptyState::Logical {
              logical,
              stage: LogicalEmptyStage::NotStart {
                has_condition,
                matched,
              },
            });
            continue;
          }

          if let Some(condition) = logical.or.as_deref().and_then(|or| or.get(index)) {
            stack.push(ConditionEmptyState::Logical {
              logical,
              stage: LogicalEmptyStage::Or {
                index: index + 1,
                has_condition,
                matched,
                matched_any,
              },
            });
            stack.push(ConditionEmptyState::Condition(condition));
          } else {
            stack.push(ConditionEmptyState::Logical {
              logical,
              stage: LogicalEmptyStage::NotStart {
                has_condition,
                matched: false,
              },
            });
          }
        }
        LogicalEmptyStage::NotStart {
          has_condition,
          matched,
        } => {
          if let Some(not) = logical.not.as_ref() {
            stack.push(ConditionEmptyState::Logical {
              logical,
              stage: LogicalEmptyStage::NotDone {
                has_condition: true,
                matched,
              },
            });
            stack.push(ConditionEmptyState::Condition(not));
          } else {
            last_result = Some(has_condition && matched);
          }
        }
        LogicalEmptyStage::NotDone {
          has_condition,
          matched,
        } => {
          last_result = Some(
            has_condition
              && matched
              && !last_result
                .take()
                .expect("logical empty not evaluation should produce a result"),
          );
        }
      },
    }
  }

  Ok(last_result.expect("empty condition evaluation should always finish with a result"))
}

impl RuleSetCondition {
  pub async fn try_match(&self, data: DataRef<'_>) -> Result<bool> {
    try_match_condition_impl(ConditionMatchRoot::Condition(self), data).await
  }

  async fn match_when_empty(&self) -> Result<bool> {
    match_when_empty_condition_impl(ConditionEmptyRoot::Condition(self)).await
  }
}

#[derive(Debug)]
pub struct RuleSetConditionWithEmpty {
  condition: RuleSetCondition,
  match_when_empty: OnceCell<bool>,
}

impl RuleSetConditionWithEmpty {
  pub fn new(condition: RuleSetCondition) -> Self {
    Self {
      condition,
      match_when_empty: OnceCell::new(),
    }
  }

  pub async fn try_match(&self, data: DataRef<'_>) -> Result<bool> {
    self.condition.try_match(data).await
  }

  pub async fn match_when_empty(&self) -> Result<bool> {
    self
      .match_when_empty
      .get_or_try_init(|| async { self.condition.match_when_empty().await })
      .await
      .copied()
  }
}

impl From<RuleSetCondition> for RuleSetConditionWithEmpty {
  fn from(condition: RuleSetCondition) -> Self {
    Self::new(condition)
  }
}

#[derive(Debug, Default)]
pub struct RuleSetLogicalConditions {
  pub and: Option<Vec<RuleSetCondition>>,
  pub or: Option<Vec<RuleSetCondition>>,
  pub not: Option<RuleSetCondition>,
}

impl RuleSetLogicalConditions {
  pub async fn try_match(&self, data: DataRef<'_>) -> Result<bool> {
    try_match_condition_impl(ConditionMatchRoot::Logical(self), data).await
  }

  pub async fn match_when_empty(&self) -> Result<bool> {
    match_when_empty_condition_impl(ConditionEmptyRoot::Logical(self)).await
  }
}

pub struct FuncUseCtx {
  pub resource: Option<String>,
  pub real_resource: Option<String>,
  pub resource_query: Option<String>,
  pub resource_fragment: Option<String>,
  pub issuer: Option<Box<str>>,
  pub issuer_layer: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ModuleRuleUseLoader {
  /// Loader identifier with query and fragments
  /// Loader ident or query will be appended if it exists.
  pub loader: String,
  /// Loader options
  /// This only exists if the loader is a built-in loader.
  pub options: Option<String>,
}

pub type FnUse =
  Box<dyn Fn(FuncUseCtx) -> BoxFuture<'static, Result<Vec<ModuleRuleUseLoader>>> + Sync + Send>;

#[derive(Debug, Default)]
pub struct ModuleRule {
  /// A conditional match matching an absolute path + query + fragment.
  /// Note:
  ///   This is a custom matching rule not initially designed by webpack.
  ///   Only for single-threaded environment interoperation purpose.
  pub rspack_resource: Option<RuleSetCondition>,
  /// A condition matcher matching an absolute path.
  pub test: Option<RuleSetCondition>,
  pub include: Option<RuleSetCondition>,
  pub exclude: Option<RuleSetCondition>,
  /// A condition matcher matching an absolute path.
  pub resource: Option<RuleSetCondition>,
  /// A condition matcher against the resource query.
  pub resource_query: Option<RuleSetConditionWithEmpty>,
  pub resource_fragment: Option<RuleSetConditionWithEmpty>,
  pub dependency: Option<RuleSetCondition>,
  pub issuer: Option<RuleSetConditionWithEmpty>,
  pub issuer_layer: Option<RuleSetConditionWithEmpty>,
  pub scheme: Option<RuleSetConditionWithEmpty>,
  pub mimetype: Option<RuleSetConditionWithEmpty>,
  pub description_data: Option<DescriptionData>,
  pub with: Option<With>,
  pub one_of: Option<Vec<ModuleRule>>,
  pub rules: Option<Vec<ModuleRule>>,
  pub effect: ModuleRuleEffect,
  pub extract_source_map: Option<bool>,
}

#[derive(Debug, Default)]
pub struct ModuleRuleEffect {
  pub side_effects: Option<bool>,
  /// The `ModuleType` to use for the matched resource.
  pub r#type: Option<ModuleType>,
  pub layer: Option<String>,
  pub r#use: ModuleRuleUse,
  pub parser: Option<ParserOptions>,
  pub generator: Option<GeneratorOptions>,
  pub resolve: Option<Resolve>,
  pub enforce: ModuleRuleEnforce,
  pub extract_source_map: Option<bool>,
}

pub enum ModuleRuleUse {
  Array(Vec<ModuleRuleUseLoader>),
  Func(FnUse),
}

impl Default for ModuleRuleUse {
  fn default() -> Self {
    Self::Array(vec![])
  }
}

impl fmt::Debug for ModuleRuleUse {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
    match self {
      ModuleRuleUse::Array(array_use) => write!(
        f,
        "{}",
        array_use
          .iter()
          .map(|l| &*l.loader)
          .collect::<Vec<_>>()
          .join("!")
      ),
      ModuleRuleUse::Func(_) => write!(f, "Fn(...)"),
    }
  }
}

pub type ModuleNoParseTestFn =
  Box<dyn Fn(String) -> BoxFuture<'static, Result<bool>> + Sync + Send>;

pub enum ModuleNoParseRule {
  AbsPathPrefix(String),
  Regexp(RspackRegex),
  TestFn(ModuleNoParseTestFn),
}

impl fmt::Debug for ModuleNoParseRule {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::TestFn(_) => "Fn(...)".fmt(f),
      _ => self.fmt(f),
    }
  }
}

impl ModuleNoParseRule {
  pub async fn try_match(&self, request: &str) -> Result<bool> {
    match self {
      Self::AbsPathPrefix(s) => Ok(request.starts_with(s)),
      Self::Regexp(reg) => Ok(reg.test(request)),
      Self::TestFn(func) => func(request.to_string()).await,
    }
  }
}

#[derive(Debug)]
pub enum ModuleNoParseRules {
  Rule(ModuleNoParseRule),
  Rules(Vec<ModuleNoParseRule>),
}

impl ModuleNoParseRules {
  #[async_recursion]
  pub async fn try_match(&self, request: &str) -> Result<bool> {
    match self {
      Self::Rule(r) => r.try_match(request).await,
      Self::Rules(list) => try_any(list, |r| r.try_match(request)).await,
    }
  }
}

#[derive(Debug, Default)]
pub enum ModuleRuleEnforce {
  Post,
  #[default]
  Normal,
  Pre,
}

// BE CAREFUL:
// Add more fields to this struct should result in adding new fields to options builder.
// `impl From<ModuleOptions> for ModuleOptionsBuilder` should be updated.
#[derive(Debug, Default)]
pub struct ModuleOptions {
  pub rules: Vec<ModuleRule>,
  pub parser: Option<ParserOptionsMap>,
  pub generator: Option<GeneratorOptionsMap>,
  pub no_parse: Option<ModuleNoParseRules>,
}

#[cfg(test)]
mod tests {
  use super::{DataRef, RuleSetCondition, RuleSetLogicalConditions};

  #[tokio::test]
  async fn should_match_nested_rule_set_conditions_without_recursion() {
    let condition = RuleSetCondition::Array(vec![
      RuleSetCondition::Logical(Box::new(RuleSetLogicalConditions {
        and: Some(vec![
          RuleSetCondition::String("/src".to_string()),
          RuleSetCondition::Array(vec![
            RuleSetCondition::String("/nope".to_string()),
            RuleSetCondition::String("/src/app".to_string()),
          ]),
        ]),
        or: Some(vec![
          RuleSetCondition::String("/miss".to_string()),
          RuleSetCondition::String("/src/app".to_string()),
        ]),
        not: Some(RuleSetCondition::String("/src/app/blocked".to_string())),
      })),
      RuleSetCondition::String("/fallback".to_string()),
    ]);

    assert!(
      condition
        .try_match(DataRef::from("/src/app/index.js"))
        .await
        .expect("nested condition should match")
    );

    assert!(
      !condition
        .try_match(DataRef::from("/src/app/blocked.js"))
        .await
        .expect("blocked path should not match")
    );
  }

  #[tokio::test]
  async fn should_preserve_rule_set_logical_short_circuit_semantics() {
    let logical = RuleSetLogicalConditions {
      and: Some(vec![
        RuleSetCondition::String("/src".to_string()),
        RuleSetCondition::String("/src/feature".to_string()),
      ]),
      or: Some(vec![
        RuleSetCondition::String("/miss".to_string()),
        RuleSetCondition::String("/src/feature".to_string()),
      ]),
      not: Some(RuleSetCondition::String(
        "/src/feature/internal".to_string(),
      )),
    };

    assert!(
      logical
        .try_match(DataRef::from("/src/feature/index.js"))
        .await
        .expect("expected logical condition to match")
    );

    assert!(
      !logical
        .try_match(DataRef::from("/src/feature/internal.js"))
        .await
        .expect("expected not condition to reject")
    );

    assert!(
      !logical
        .try_match(DataRef::from("/src/other/index.js"))
        .await
        .expect("expected and condition to reject")
    );
  }

  #[tokio::test]
  async fn should_preserve_match_when_empty_semantics() {
    let array = RuleSetCondition::Array(vec![RuleSetCondition::String(String::new())]);
    assert!(
      !array
        .match_when_empty()
        .await
        .expect("array should not match empty input")
    );

    let logical = RuleSetLogicalConditions {
      and: Some(vec![RuleSetCondition::String(String::new())]),
      or: Some(vec![RuleSetCondition::String(String::new())]),
      not: Some(RuleSetCondition::String("/blocked".to_string())),
    };

    assert!(
      logical
        .match_when_empty()
        .await
        .expect("logical condition should match empty input")
    );

    assert!(
      !RuleSetLogicalConditions::default()
        .match_when_empty()
        .await
        .expect("empty logical condition should not match empty input")
    );

    assert!(
      !RuleSetLogicalConditions {
        or: Some(vec![]),
        ..Default::default()
      }
      .match_when_empty()
      .await
      .expect("empty or list should remain false")
    );
  }
}
