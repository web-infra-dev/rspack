use std::{
  fmt::{self, Debug},
  future::Future,
};

use async_recursion::async_recursion;
use futures::future::BoxFuture;
use rspack_error::Result;
use rspack_regex::RspackRegex;

use crate::{BoxLoader, Filename, ModuleType, Resolve};

#[derive(Debug, Clone, Default)]
pub struct AssetParserDataUrlOption {
  pub max_size: Option<u32>,
}
#[derive(Debug, Clone, Default)]
pub struct AssetParserOptions {
  pub data_url_condition: Option<AssetParserDataUrlOption>,
}
#[derive(Debug, Clone, Default)]
pub struct ParserOptions {
  pub asset: Option<AssetParserOptions>,
}

#[derive(Debug, Clone, Default)]
pub struct AssetGeneratorOptions {
  /// Same as webpack's Rule.generator.filename, see: [Rule.generator.filename](https://webpack.js.org/configuration/module/#rulegeneratorfilename)
  pub filename: Option<Filename>,
}

pub type RuleSetConditionFnMatcher =
  Box<dyn Fn(&str) -> BoxFuture<'static, Result<bool>> + Sync + Send>;

// #[async_trait::async_trait]
// pub trait RuleSetConditionFn: Send + Sync {
//   async fn try_match(&self, data: &str) -> Result<bool>;
// }

// #[async_trait::async_trait]
// impl<F> RuleSetConditionFn for F
// where
//   F: Fn(String) -> BoxFuture<'static, Result<bool>>,
//   F: Send + Sync,
// {
//   async fn try_match(&self, data: &str) -> Result<bool> {
//     self(data.to_string()).await
//   }
// }

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

async fn try_any<T, Fut, F>(it: impl IntoIterator<Item = T>, f: F) -> Result<bool>
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

#[derive(Debug, Default)]
pub struct ModuleRule {
  /// A condition matcher matching an absolute path.
  pub test: Option<RuleSetCondition>,
  pub include: Option<RuleSetCondition>,
  pub exclude: Option<RuleSetCondition>,
  /// A condition matcher matching an absolute path.
  pub resource: Option<RuleSetCondition>,
  /// A condition matcher against the resource query.
  pub resource_query: Option<RuleSetCondition>,
  pub side_effects: Option<bool>,
  pub dependency: Option<RuleSetCondition>,
  /// The `ModuleType` to use for the matched resource.
  pub r#type: Option<ModuleType>,
  pub r#use: Vec<BoxLoader>,
  pub parser: Option<AssetParserOptions>,
  pub generator: Option<AssetGeneratorOptions>,
  pub resolve: Option<Resolve>,
  pub issuer: Option<RuleSetCondition>,
  pub one_of: Option<Vec<ModuleRule>>,
}

#[derive(Debug, Default)]
pub struct ModuleOptions {
  pub rules: Vec<ModuleRule>,
  pub parser: Option<ParserOptions>,
}
