use std::fmt::Debug;

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

#[derive(Debug)]
pub enum RuleSetCondition {
  String(String),
  Regexp(RspackRegex),
  Logical(Box<RuleSetLogicalConditions>),
  Array(Vec<RuleSetCondition>),
}

impl RuleSetCondition {
  pub fn is_match(&self, data: &str) -> bool {
    match self {
      Self::String(s) => data.starts_with(s),
      Self::Regexp(r) => r.test(data),
      Self::Logical(g) => g.is_match(data),
      Self::Array(l) => l.iter().any(|i| i.is_match(data)),
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
  pub fn is_match(&self, data: &str) -> bool {
    if let Some(and) = &self.and && and.iter().any(|i| !i.is_match(data)) {
      return false
    }
    if let Some(or) = &self.or && or.iter().all(|i| !i.is_match(data)) {
      return false
    }
    if let Some(not) = &self.not && not.is_match(data) {
      return false
    }
    true
  }
}

#[derive(Default)]
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
  /// The `ModuleType` to use for the matched resource.
  pub r#type: Option<ModuleType>,
  pub r#use: Vec<BoxLoader>,
  pub parser: Option<AssetParserOptions>,
  pub generator: Option<AssetGeneratorOptions>,
  pub resolve: Option<Resolve>,
  pub issuer: Option<RuleSetCondition>,
  pub one_of: Option<Vec<ModuleRule>>,
}

impl Debug for ModuleRule {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("ModuleRule")
      .field("test", &self.test)
      .field("include", &self.include)
      .field("exclude", &self.exclude)
      .field("resource", &self.resource)
      .field("resource_query", &self.resource_query)
      .field("type", &self.r#type)
      .field("resolve", &self.resolve)
      .field("parser", &self.parser)
      .field("generator", &self.generator)
      .field("use", &self.r#use)
      .field("side_effects", &self.side_effects)
      .field("issuer", &self.issuer)
      .field("one_of", &self.one_of)
      .finish()
  }
}

#[derive(Debug, Default)]
pub struct ModuleOptions {
  pub rules: Vec<ModuleRule>,
  pub parser: Option<ParserOptions>,
}
