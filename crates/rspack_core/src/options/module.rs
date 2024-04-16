use std::{
  fmt::{self, Debug},
  sync::Arc,
};

use async_recursion::async_recursion;
use bitflags::bitflags;
use derivative::Derivative;
use futures::future::BoxFuture;
use rspack_error::Result;
use rspack_macros::MergeFrom;
use rspack_regex::RspackRegex;
use rspack_util::{try_all, try_any, MergeFrom};
use rustc_hash::FxHashMap as HashMap;

use crate::{Filename, ModuleType, PublicPath, Resolve};

#[derive(Debug)]
pub struct ParserOptionsByModuleType(HashMap<ModuleType, ParserOptions>);

impl FromIterator<(ModuleType, ParserOptions)> for ParserOptionsByModuleType {
  fn from_iter<I: IntoIterator<Item = (ModuleType, ParserOptions)>>(i: I) -> Self {
    Self(HashMap::from_iter(i))
  }
}

impl ParserOptionsByModuleType {
  pub fn get<'a>(&'a self, module_type: &'a ModuleType) -> Option<&'a ParserOptions> {
    self.0.get(module_type)
  }
}

#[derive(Debug, Clone, MergeFrom)]
pub enum ParserOptions {
  Asset(AssetParserOptions),
  Css(CssParserOptions),
  CssAuto(CssAutoParserOptions),
  CssModule(CssModuleParserOptions),
  Javascript(JavascriptParserOptions),
  Unknown,
}

macro_rules! get_variant {
  ($fn_name:ident, $variant:ident, $module_variant:ident, $ret_ty:ident) => {
    pub fn $fn_name(&self, module_type: &ModuleType) -> Option<&$ret_ty> {
      match self {
        Self::$variant(value) if *module_type == ModuleType::$module_variant => Some(value),
        _ => None,
      }
    }
  };
}

impl ParserOptions {
  get_variant!(get_asset, Asset, Asset, AssetParserOptions);
  get_variant!(get_css, Css, Css, CssParserOptions);
  get_variant!(get_css_auto, CssAuto, CssAuto, CssAutoParserOptions);
  get_variant!(get_css_module, CssModule, CssModule, CssModuleParserOptions);
  get_variant!(get_javascript, Javascript, Js, JavascriptParserOptions);
}

#[derive(Debug, Clone, Copy, Default, MergeFrom)]
pub enum DynamicImportMode {
  #[default]
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
      "lazyOnce" => DynamicImportMode::LazyOnce,
      _ => {
        // TODO: warning
        DynamicImportMode::default()
      }
    }
  }
}

#[derive(Debug, Clone, Copy, Default, MergeFrom)]
pub enum JavascriptParserUrl {
  #[default]
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

#[derive(Debug, Clone, Copy, Default, MergeFrom)]
pub enum JavascriptParserOrder {
  #[default]
  Disable,
  Order(u32),
}

impl JavascriptParserOrder {
  pub fn get_order(&self) -> Option<u32> {
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
        if let Ok(order) = value.parse::<u32>() {
          Self::Order(order)
        } else {
          Self::Order(0)
        }
      }
    }
  }
}

#[derive(Debug, Clone, Default, MergeFrom)]
pub struct JavascriptParserOptions {
  pub dynamic_import_mode: DynamicImportMode,
  pub dynamic_import_preload: JavascriptParserOrder,
  pub dynamic_import_prefetch: JavascriptParserOrder,
  pub url: JavascriptParserUrl,
}

#[derive(Debug, Clone, MergeFrom)]
pub struct AssetParserOptions {
  pub data_url_condition: Option<AssetParserDataUrl>,
}

#[derive(Debug, Clone, MergeFrom)]
pub enum AssetParserDataUrl {
  Options(AssetParserDataUrlOptions),
  // TODO: Function
}

#[derive(Debug, Clone, MergeFrom)]
pub struct AssetParserDataUrlOptions {
  pub max_size: Option<u32>,
}

#[derive(Debug, Clone, MergeFrom)]
pub struct CssParserOptions {
  pub named_exports: Option<bool>,
}

#[derive(Debug, Clone, MergeFrom)]
pub struct CssAutoParserOptions {
  pub named_exports: Option<bool>,
}

#[derive(Debug, Clone, MergeFrom)]
pub struct CssModuleParserOptions {
  pub named_exports: Option<bool>,
}

#[derive(Debug)]
pub struct GeneratorOptionsByModuleType(HashMap<ModuleType, GeneratorOptions>);

impl FromIterator<(ModuleType, GeneratorOptions)> for GeneratorOptionsByModuleType {
  fn from_iter<I: IntoIterator<Item = (ModuleType, GeneratorOptions)>>(i: I) -> Self {
    Self(HashMap::from_iter(i))
  }
}

impl GeneratorOptionsByModuleType {
  pub fn get(&self, module_type: &ModuleType) -> Option<&GeneratorOptions> {
    self.0.get(module_type)
  }
}

#[derive(Debug, Clone, MergeFrom)]
pub enum GeneratorOptions {
  Asset(AssetGeneratorOptions),
  AssetInline(AssetInlineGeneratorOptions),
  AssetResource(AssetResourceGeneratorOptions),
  Css(CssGeneratorOptions),
  CssAuto(CssAutoGeneratorOptions),
  CssModule(CssModuleGeneratorOptions),
  Unknown,
}

impl GeneratorOptions {
  get_variant!(get_asset, Asset, Asset, AssetGeneratorOptions);
  get_variant!(
    get_asset_inline,
    AssetInline,
    AssetInline,
    AssetInlineGeneratorOptions
  );
  get_variant!(
    get_asset_resource,
    AssetResource,
    AssetResource,
    AssetResourceGeneratorOptions
  );
  get_variant!(get_css, Css, Css, CssGeneratorOptions);
  get_variant!(get_css_auto, CssAuto, CssAuto, CssAutoGeneratorOptions);
  get_variant!(
    get_css_module,
    CssModule,
    CssModule,
    CssModuleGeneratorOptions
  );

  pub fn asset_filename(&self, module_type: &ModuleType) -> Option<&Filename> {
    self
      .get_asset(module_type)
      .and_then(|x| x.filename.as_ref())
      .or_else(|| {
        self
          .get_asset_resource(module_type)
          .and_then(|x| x.filename.as_ref())
      })
  }

  pub fn asset_public_path(&self, module_type: &ModuleType) -> Option<&PublicPath> {
    self
      .get_asset(module_type)
      .and_then(|x| x.public_path.as_ref())
      .or_else(|| {
        self
          .get_asset_resource(module_type)
          .and_then(|x| x.public_path.as_ref())
      })
  }

  pub fn asset_data_url(&self, module_type: &ModuleType) -> Option<&AssetGeneratorDataUrl> {
    self
      .get_asset(module_type)
      .and_then(|x| x.data_url.as_ref())
      .or_else(|| {
        self
          .get_asset_inline(module_type)
          .and_then(|x| x.data_url.as_ref())
      })
  }

  pub fn asset_emit(&self, module_type: &ModuleType) -> Option<bool> {
    self
      .get_asset(module_type)
      .and_then(|x| x.emit)
      .or_else(|| self.get_asset_resource(module_type).and_then(|x| x.emit))
  }
}

#[derive(Debug, Clone, MergeFrom)]
pub struct AssetInlineGeneratorOptions {
  pub data_url: Option<AssetGeneratorDataUrl>,
}

#[derive(Debug, Clone, MergeFrom)]
pub struct AssetResourceGeneratorOptions {
  pub emit: Option<bool>,
  pub filename: Option<Filename>,
  pub public_path: Option<PublicPath>,
}

#[derive(Debug, Clone, MergeFrom)]
pub struct AssetGeneratorOptions {
  pub emit: Option<bool>,
  pub filename: Option<Filename>,
  pub public_path: Option<PublicPath>,
  pub data_url: Option<AssetGeneratorDataUrl>,
}

pub struct AssetGeneratorDataUrlFnArgs {
  pub filename: String,
  pub content: String,
}

pub type AssetGeneratorDataUrlFn =
  Arc<dyn Fn(AssetGeneratorDataUrlFnArgs) -> Result<String> + Sync + Send>;

pub enum AssetGeneratorDataUrl {
  Options(AssetGeneratorDataUrlOptions),
  Func(AssetGeneratorDataUrlFn),
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

#[derive(Debug, Clone, MergeFrom)]
pub struct AssetGeneratorDataUrlOptions {
  pub encoding: Option<DataUrlEncoding>,
  pub mimetype: Option<String>,
}

#[derive(Debug, Clone, MergeFrom)]
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

#[derive(Debug, Clone, MergeFrom)]
pub struct CssGeneratorOptions {
  pub exports_convention: Option<CssExportsConvention>,
  pub exports_only: Option<bool>,
}

#[derive(Debug, Clone, MergeFrom)]
pub struct CssAutoGeneratorOptions {
  pub exports_convention: Option<CssExportsConvention>,
  pub exports_only: Option<bool>,
  pub local_ident_name: Option<LocalIdentName>,
}

#[derive(Debug, Clone, MergeFrom)]
pub struct CssModuleGeneratorOptions {
  pub exports_convention: Option<CssExportsConvention>,
  pub exports_only: Option<bool>,
  pub local_ident_name: Option<LocalIdentName>,
}

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

bitflags! {
  #[derive(Debug, Clone, Copy)]
  struct ExportsConventionFlags: u8 {
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

pub type DescriptionData = HashMap<String, RuleSetCondition>;

pub type RuleSetConditionFnMatcher =
  Box<dyn Fn(&str) -> BoxFuture<'static, Result<bool>> + Sync + Send>;

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

impl RuleSetCondition {
  #[async_recursion]
  pub async fn try_match(&self, data: &str) -> Result<bool> {
    match self {
      Self::String(s) => Ok(data.starts_with(s)),
      Self::Regexp(r) => Ok(r.test(data)),
      Self::Logical(g) => g.try_match(data).await,
      Self::Array(l) => try_any(l, |i| async { i.try_match(data).await }).await,
      Self::Func(f) => f(data).await,
    }
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
  pub async fn try_match(&self, data: &str) -> Result<bool> {
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
}

pub struct FuncUseCtx {
  pub resource: Option<String>,
  pub real_resource: Option<String>,
  pub resource_query: Option<String>,
  pub issuer: Option<Box<str>>,
}

#[derive(Debug, Clone)]
pub struct ModuleRuleUseLoader {
  /// Loader identifier with query and fragments
  pub loader: String,
  /// Loader options
  pub options: Option<String>,
}

pub type FnUse =
  Box<dyn Fn(FuncUseCtx) -> BoxFuture<'static, Result<Vec<ModuleRuleUseLoader>>> + Sync + Send>;

#[derive(Derivative, Default)]
#[derivative(Debug)]
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
  pub resource_query: Option<RuleSetCondition>,
  pub resource_fragment: Option<RuleSetCondition>,
  pub dependency: Option<RuleSetCondition>,
  pub issuer: Option<RuleSetCondition>,
  pub scheme: Option<RuleSetCondition>,
  pub mimetype: Option<RuleSetCondition>,
  pub description_data: Option<DescriptionData>,
  pub side_effects: Option<bool>,
  /// The `ModuleType` to use for the matched resource.
  pub r#type: Option<ModuleType>,
  #[derivative(Debug(format_with = "fmt_use"))]
  pub r#use: ModuleRuleUse,
  pub parser: Option<ParserOptions>,
  pub generator: Option<GeneratorOptions>,
  pub resolve: Option<Resolve>,
  pub one_of: Option<Vec<ModuleRule>>,
  pub rules: Option<Vec<ModuleRule>>,
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

fn fmt_use(
  r#use: &ModuleRuleUse,
  f: &mut std::fmt::Formatter,
) -> std::result::Result<(), std::fmt::Error> {
  match r#use {
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

#[derive(Debug, Default)]
pub struct ModuleOptions {
  pub rules: Vec<ModuleRule>,
  pub parser: Option<ParserOptionsByModuleType>,
  pub generator: Option<GeneratorOptionsByModuleType>,
  pub no_parse: Option<ModuleNoParseRules>,
}
