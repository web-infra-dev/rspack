use lazy_regex::Lazy;
use regex::Regex;
use rspack_core::Module;
use serde::Deserialize;

static RSPACK_RSC_MODULE_INFORMATION: Lazy<Regex> = Lazy::new(|| {
  Regex::new(r"/\* __rspack_internal_rsc_module_information_do_not_use__ (\{[^}]+\}) \*/").unwrap()
});

const CLIENT_DIRECTIVE: &str = "use client";
const SERVER_ACTION_DIRECTIVE: &str = "use server";

pub type RSCModuleType = &'static str;
pub const RSC_MODULE_TYPES: RSCModuleTypes = RSCModuleTypes {
  client: "client",
  server: "server",
};

pub struct RSCModuleTypes {
  pub client: RSCModuleType,
  pub server: RSCModuleType,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RSCMeta {
  pub r#type: String, // RSCModuleType
  pub actions: Option<Vec<String>>,
  pub action_ids: Option<std::collections::HashMap<String, String>>,
  pub client_refs: Option<Vec<String>>,
  pub client_entry_type: Option<String>,
  pub is_client_ref: bool,
}

fn get_rsc_module_information(source: &str) -> Option<RSCMeta> {
  RSPACK_RSC_MODULE_INFORMATION
    .captures(source)
    .and_then(|caps| caps.get(1).map(|m| m.as_str()))
    .and_then(|info| serde_json::from_str(info).unwrap())
}

pub fn get_module_rsc_information(module: &dyn Module) -> Option<RSCMeta> {
  module
    .original_source()
    .and_then(|s| get_rsc_module_information(s.source().as_ref()))
}
