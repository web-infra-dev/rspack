use crate::ModuleType;

#[derive(Debug, Default)]
pub struct ModuleRule {
  pub test: Option<regex::Regex>,
  pub resource: Option<regex::Regex>,
  pub resource_query: Option<regex::Regex>,
  pub module_type: Option<ModuleType>,
}

#[derive(Debug, Default)]
pub struct ModuleOptions {
  pub rules: Vec<ModuleRule>,
}
