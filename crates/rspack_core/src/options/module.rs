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

#[derive(Default)]
pub struct ModuleRule {
  pub test: Option<regex::Regex>,
  pub resource: Option<regex::Regex>,
  pub resource_query: Option<regex::Regex>,
  pub module_type: Option<ModuleType>,
  // For loader experimental
  pub func__: Option<ModuleRuleFunc>,
  pub uses: Vec<BoxedLoader>,
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
