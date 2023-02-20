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
pub enum ModuleRuleCondition {
  String(String),
  Regexp(RspackRegex),
  // TODO: support logical conditions
  // LogicalConditions
}

#[derive(Debug, Default)]
pub struct IssuerOptions {
  pub not: Option<Vec<ModuleRuleCondition>>,
}

#[derive(Default)]
pub struct ModuleRule {
  /// A condition matcher matching an absolute path.
  /// - String: To match the input must start with the provided string. I. e. an absolute directory path, or absolute path to the file.
  /// - Regexp: It's tested with the input.
  pub test: Option<ModuleRuleCondition>,
  pub include: Option<Vec<ModuleRuleCondition>>,
  pub exclude: Option<Vec<ModuleRuleCondition>>,
  /// A condition matcher matching an absolute path.
  /// See `test` above
  pub resource: Option<ModuleRuleCondition>,
  /// A condition matcher against the resource query.
  /// TODO: align with webpack's `?` prefixed `resourceQuery`
  pub resource_query: Option<ModuleRuleCondition>,
  pub side_effects: Option<bool>,
  /// The `ModuleType` to use for the matched resource.
  pub r#type: Option<ModuleType>,
  pub r#use: Vec<BoxLoader>,
  pub parser: Option<AssetParserOptions>,
  pub generator: Option<AssetGeneratorOptions>,
  pub resolve: Option<Resolve>,
  pub issuer: Option<IssuerOptions>,
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
