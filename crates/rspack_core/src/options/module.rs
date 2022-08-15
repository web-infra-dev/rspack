use crate::ModuleType;

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

#[derive(Debug, Default, Clone)]
pub struct ModuleRule {
  pub test: Option<regex::Regex>,
  pub resource: Option<regex::Regex>,
  pub resource_query: Option<regex::Regex>,
  pub module_type: Option<ModuleType>,
}

#[derive(Debug, Default, Clone)]
pub struct ModuleOptions {
  pub rules: Vec<ModuleRule>,
  pub parser: Option<ParserOptions>,
}
