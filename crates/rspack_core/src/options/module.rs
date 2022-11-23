use std::fmt::Debug;

use crate::{BoxedLoader, ModuleType, ResourceData};

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

type ModuleRuleFunc = Box<dyn Fn(&ResourceData) -> anyhow::Result<bool> + Send + Sync>;

#[derive(Debug)]
pub enum ModuleRuleCondition {
  String(String),
  Regexp(regex::Regex),
  // TODO: support logical conditions
  // LogicalConditions
}

#[derive(Default)]
pub struct ModuleRule {
  /// A condition matcher matching an absolute path.
  /// - String: To match the input must start with the provided string. I. e. an absolute directory path, or absolute path to the file.
  /// - Regexp: It's tested with the input.
  pub test: Option<ModuleRuleCondition>,
  /// A condition matcher matching an absolute path.
  /// See `test` above
  pub resource: Option<ModuleRuleCondition>,
  /// A condition matcher against the resource query.
  pub resource_query: Option<ModuleRuleCondition>,
  /// The `ModuleType` to use for the matched resource.
  pub module_type: Option<ModuleType>,
  pub uses: Vec<BoxedLoader>,
  /// Internal matching method, not intended to be used by the user. (Loader experimental)
  pub func__: Option<ModuleRuleFunc>,
}

impl Debug for ModuleRule {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("ModuleRule")
      .field("test", &self.test)
      .field("resource", &self.resource)
      .field("resource_query", &self.resource_query)
      .field("module_type", &self.module_type)
      .field("func__", &self.func__.as_ref().map(|_| ".."))
      .field("uses", &self.uses)
      .finish()
  }
}

#[derive(Debug, Default)]
pub struct ModuleOptions {
  pub rules: Vec<ModuleRule>,
  pub parser: Option<ParserOptions>,
}
