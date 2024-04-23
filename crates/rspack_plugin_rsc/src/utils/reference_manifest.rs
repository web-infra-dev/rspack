use std::collections::HashMap;

use indexmap::set::IndexSet;
use serde::Serialize;

pub type ClientImports = HashMap<String, IndexSet<String>>;
type SSRModuleMapping = HashMap<String, HashMap<String, ServerRef>>;

#[derive(Debug, Default, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ServerReferenceManifest {
  pub ssr_module_mapping: SSRModuleMapping,
}

#[derive(Debug, Serialize, Clone)]
pub struct ServerRef {
  pub id: String,
  pub name: String,
  pub chunks: Vec<String>,
}

#[derive(Debug, Default, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ClientReferenceManifest {
  pub client_modules: HashMap<String, ClientRef>,
  pub ssr_module_mapping: SSRModuleMapping,
}

#[derive(Debug, Serialize, Clone)]
pub struct ClientRef {
  pub id: String,
  pub name: String,
  pub chunks: Vec<String>,
}
