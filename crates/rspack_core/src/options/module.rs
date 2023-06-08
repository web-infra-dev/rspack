use std::{
  fmt::{self, Debug},
  future::Future,
};

use async_recursion::async_recursion;
use derivative::Derivative;
use futures::future::BoxFuture;
use rspack_error::Result;
use rspack_regex::RspackRegex;
use rustc_hash::FxHashMap as HashMap;

use crate::{BoxLoader, Filename, ModuleType, PublicPath, Resolve};

#[derive(Debug)]
pub struct ParserOptionsByModuleType(HashMap<ModuleType, ParserOptions>);

impl FromIterator<(ModuleType, ParserOptions)> for ParserOptionsByModuleType {
  fn from_iter<I: IntoIterator<Item = (ModuleType, ParserOptions)>>(i: I) -> Self {
    Self(HashMap::from_iter(i))
  }
}

impl ParserOptionsByModuleType {
  pub fn get(&self, module_type: &ModuleType) -> Option<&ParserOptions> {
    self.0.get(module_type)
  }
}

#[derive(Debug, Clone)]
pub enum ParserOptions {
  Asset(AssetParserOptions),
  Unknown,
}

impl ParserOptions {
  pub fn get_asset(&self, module_type: &ModuleType) -> Option<&AssetParserOptions> {
    let maybe = match self {
      ParserOptions::Asset(i) => Some(i),
      _ => None,
    };
    maybe.filter(|_| matches!(module_type, ModuleType::Asset))
  }
}

#[derive(Debug, Clone)]
pub struct AssetParserOptions {
  pub data_url_condition: Option<AssetParserDataUrl>,
}

#[derive(Debug, Clone)]
pub enum AssetParserDataUrl {
  Options(AssetParserDataUrlOptions),
  // TODO: Function
}

#[derive(Debug, Clone)]
pub struct AssetParserDataUrlOptions {
  pub max_size: Option<u32>,
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

#[derive(Debug, Clone)]
pub enum GeneratorOptions {
  Asset(AssetGeneratorOptions),
  AssetInline(AssetInlineGeneratorOptions),
  AssetResource(AssetResourceGeneratorOptions),
  Unknown,
}

impl GeneratorOptions {
  pub fn get_asset(&self, module_type: &ModuleType) -> Option<&AssetGeneratorOptions> {
    let maybe = match self {
      Self::Asset(i) => Some(i),
      _ => None,
    };
    maybe.filter(|_| matches!(module_type, ModuleType::Asset))
  }

  pub fn get_asset_inline(&self, module_type: &ModuleType) -> Option<&AssetInlineGeneratorOptions> {
    let maybe = match self {
      Self::AssetInline(i) => Some(i),
      _ => None,
    };
    maybe.filter(|_| matches!(module_type, ModuleType::AssetInline))
  }

  pub fn get_asset_resource(
    &self,
    module_type: &ModuleType,
  ) -> Option<&AssetResourceGeneratorOptions> {
    let maybe = match self {
      Self::AssetResource(i) => Some(i),
      _ => None,
    };
    maybe.filter(|_| matches!(module_type, ModuleType::AssetResource))
  }

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
}

#[derive(Debug, Clone)]
pub struct AssetInlineGeneratorOptions {
  pub data_url: Option<AssetGeneratorDataUrl>,
}

#[derive(Debug, Clone)]
pub struct AssetResourceGeneratorOptions {
  pub filename: Option<Filename>,
  pub public_path: Option<PublicPath>,
}

#[derive(Debug, Clone)]
pub struct AssetGeneratorOptions {
  pub filename: Option<Filename>,
  pub public_path: Option<PublicPath>,
  pub data_url: Option<AssetGeneratorDataUrl>,
}

#[derive(Debug, Clone)]
pub enum AssetGeneratorDataUrl {
  Options(AssetGeneratorDataUrlOptions),
  // TODO: Function
}

#[derive(Debug, Clone)]
pub struct AssetGeneratorDataUrlOptions {
  pub encoding: Option<DataUrlEncoding>,
  pub mimetype: Option<String>,
}

#[derive(Debug, Clone)]
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
    if let Some(and) = &self.and && try_any(and, |i| async { i.try_match(data).await.map(|i| !i) }).await? {
      return Ok(false)
    }
    if let Some(or) = &self.or && try_all(or, |i| async { i.try_match(data).await.map(|i| !i) }).await? {
      return Ok(false)
    }
    if let Some(not) = &self.not && not.try_match(data).await? {
      return Ok(false)
    }
    Ok(true)
  }
}

pub async fn try_any<T, Fut, F>(it: impl IntoIterator<Item = T>, f: F) -> Result<bool>
where
  Fut: Future<Output = Result<bool>>,
  F: Fn(T) -> Fut,
{
  let it = it.into_iter();
  for i in it {
    if f(i).await? {
      return Ok(true);
    }
  }
  Ok(false)
}

async fn try_all<T, Fut, F>(it: impl IntoIterator<Item = T>, f: F) -> Result<bool>
where
  Fut: Future<Output = Result<bool>>,
  F: Fn(T) -> Fut,
{
  let it = it.into_iter();
  for i in it {
    if !(f(i).await?) {
      return Ok(false);
    }
  }
  Ok(true)
}

#[derive(Derivative, Default)]
#[derivative(Debug)]
pub struct ModuleRule {
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
  pub r#use: Vec<BoxLoader>,
  pub parser: Option<ParserOptions>,
  pub generator: Option<GeneratorOptions>,
  pub resolve: Option<Resolve>,
  pub one_of: Option<Vec<ModuleRule>>,
  pub rules: Option<Vec<ModuleRule>>,
  pub enforce: ModuleRuleEnforce,
}

fn fmt_use(
  r#use: &[BoxLoader],
  f: &mut std::fmt::Formatter,
) -> std::result::Result<(), std::fmt::Error> {
  write!(
    f,
    "{}",
    r#use
      .iter()
      .map(|l| l.identifier().to_string())
      .collect::<Vec<_>>()
      .join("!")
  )
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
}
