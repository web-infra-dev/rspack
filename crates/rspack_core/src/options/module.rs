use std::{
  fmt::{self, Debug},
  ops::{Deref, DerefMut},
  str::FromStr,
  sync::Arc,
};

use async_recursion::async_recursion;
use bitflags::bitflags;
use futures::future::BoxFuture;
use rspack_cacheable::{cacheable, with::Unsupported};
use rspack_error::Result;
use rspack_macros::MergeFrom;
use rspack_regex::RspackRegex;
use rspack_util::{try_all, try_any, MergeFrom};
use rustc_hash::FxHashMap as HashMap;
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
  Relative,
}

impl From<&str> for JavascriptParserUrl {
  fn from(value: &str) -> Self {
    match value {
      "false" => Self::Disable,
      "relative" => Self::Relative,
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
#[derive(Debug, Clone, MergeFrom, Default)]
pub struct JavascriptParserOptions {
  pub dynamic_import_mode: Option<DynamicImportMode>,
  pub dynamic_import_preload: Option<JavascriptParserOrder>,
  pub dynamic_import_prefetch: Option<JavascriptParserOrder>,
  pub dynamic_import_fetch_priority: Option<DynamicImportFetchPriority>,
  pub url: Option<JavascriptParserUrl>,
  pub expr_context_critical: Option<bool>,
  pub wrapped_context_critical: Option<bool>,
  pub wrapped_context_reg_exp: Option<RspackRegex>,
  pub exports_presence: Option<ExportPresenceMode>,
  pub import_exports_presence: Option<ExportPresenceMode>,
  pub reexport_exports_presence: Option<ExportPresenceMode>,
  pub strict_export_presence: Option<bool>,
  pub worker: Option<Vec<String>>,
  pub override_strict: Option<OverrideStrict>,
  pub import_meta: Option<bool>,
  pub require_as_expression: Option<bool>,
  pub require_dynamic: Option<bool>,
  pub require_resolve: Option<bool>,
  pub import_dynamic: Option<bool>,
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

#[cacheable]
#[derive(Debug, Clone, MergeFrom)]
pub struct CssParserOptions {
  pub named_exports: Option<bool>,
}

#[cacheable]
#[derive(Debug, Clone, MergeFrom)]
pub struct CssAutoParserOptions {
  pub named_exports: Option<bool>,
}

impl From<CssParserOptions> for CssAutoParserOptions {
  fn from(value: CssParserOptions) -> Self {
    Self {
      named_exports: value.named_exports,
    }
  }
}

#[cacheable]
#[derive(Debug, Clone, MergeFrom)]
pub struct CssModuleParserOptions {
  pub named_exports: Option<bool>,
}

impl From<CssParserOptions> for CssModuleParserOptions {
  fn from(value: CssParserOptions) -> Self {
    Self {
      named_exports: value.named_exports,
    }
  }
}

pub type JsonParseFn = Arc<dyn Fn(String) -> Result<String> + Sync + Send>;

#[cacheable]
pub enum ParseOption {
  Func(#[cacheable(with=Unsupported)] JsonParseFn),
  None,
}

impl Debug for ParseOption {
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

  pub fn asset_emit(&self) -> Option<bool> {
    self
      .get_asset()
      .and_then(|x| x.emit)
      .or_else(|| self.get_asset_resource().and_then(|x| x.emit))
  }
}

#[cacheable]
#[derive(Debug, Clone, MergeFrom)]
pub struct AssetInlineGeneratorOptions {
  pub data_url: Option<AssetGeneratorDataUrl>,
}

impl From<AssetGeneratorOptions> for AssetInlineGeneratorOptions {
  fn from(value: AssetGeneratorOptions) -> Self {
    Self {
      data_url: value.data_url,
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
}

impl From<AssetGeneratorOptions> for AssetResourceGeneratorOptions {
  fn from(value: AssetGeneratorOptions) -> Self {
    Self {
      emit: value.emit,
      filename: value.filename,
      output_path: value.output_path,
      public_path: value.public_path,
      import_mode: value.import_mode,
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
}

pub struct AssetGeneratorDataUrlFnCtx<'a> {
  pub filename: String,
  pub module: &'a dyn Module,
  pub compilation: &'a Compilation,
}

pub type AssetGeneratorDataUrlFn =
  Arc<dyn Fn(Vec<u8>, AssetGeneratorDataUrlFnCtx) -> Result<String> + Sync + Send>;

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
  pub template: crate::FilenameTemplate,
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
      template: crate::FilenameTemplate::from_str(value).expect("should be infalliable"),
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

impl RuleSetCondition {
  #[async_recursion]
  pub async fn try_match(&self, data: DataRef<'async_recursion>) -> Result<bool> {
    match self {
      Self::String(s) => Ok(
        data
          .as_str()
          .map(|data| data.starts_with(s))
          .unwrap_or_default(),
      ),
      Self::Regexp(r) => Ok(data.as_str().map(|data| r.test(data)).unwrap_or_default()),
      Self::Logical(g) => g.try_match(data).await,
      Self::Array(l) => try_any(l, |i| async { i.try_match(data).await }).await,
      Self::Func(f) => f(data).await,
    }
  }

  #[async_recursion]
  async fn match_when_empty(&self) -> Result<bool> {
    let res = match self {
      RuleSetCondition::String(s) => s.is_empty(),
      RuleSetCondition::Regexp(rspack_regex) => rspack_regex.test(""),
      RuleSetCondition::Logical(logical) => logical.match_when_empty().await?,
      RuleSetCondition::Array(arr) => {
        arr.is_empty() && try_any(arr, |c| async move { c.match_when_empty().await }).await?
      }
      RuleSetCondition::Func(func) => func("".into()).await?,
    };
    Ok(res)
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
  #[async_recursion]
  pub async fn try_match(&self, data: DataRef<'async_recursion>) -> Result<bool> {
    if let Some(and) = &self.and
      && try_any(and, |i| async { i.try_match(data).await.map(|i| !i) }).await?
    {
      return Ok(false);
    }
    if let Some(or) = &self.or
      && try_all(or, |i| async { i.try_match(data).await.map(|i| !i) }).await?
    {
      return Ok(false);
    }
    if let Some(not) = &self.not
      && not.try_match(data).await?
    {
      return Ok(false);
    }
    Ok(true)
  }

  pub async fn match_when_empty(&self) -> Result<bool> {
    let mut has_condition = false;
    let mut match_when_empty = true;
    if let Some(and) = &self.and {
      has_condition = true;
      match_when_empty &= try_all(and, |i| async { i.match_when_empty().await }).await?;
    }
    if let Some(or) = &self.or {
      has_condition = true;
      match_when_empty &= try_any(or, |i| async { i.match_when_empty().await }).await?;
    }
    if let Some(not) = &self.not {
      has_condition = true;
      match_when_empty &= !not.match_when_empty().await?;
    }
    Ok(has_condition && match_when_empty)
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

impl Debug for ModuleRuleUse {
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

impl Debug for ModuleNoParseRule {
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
