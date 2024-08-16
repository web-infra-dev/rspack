use std::collections::HashMap;

use indexmap::map::IndexMap;
use indexmap::set::IndexSet;
use serde::{Deserialize, Serialize};

use crate::export_visitor::ExportSpecifier;

pub type ClientImports = HashMap<String, IndexSet<String>>;
type SSRModuleMapping = HashMap<String, HashMap<String, ServerRef>>;
pub type ServerImports = HashMap<String, ServerActionRef>;
// action_id -> chunk_group -> platform
pub type ServerActions = IndexMap<String, HashMap<String, HashMap<String, String>>>;

#[derive(Debug, Serialize, Clone)]
pub struct ServerActionRef {
  pub names: Vec<String>,
}

#[derive(Debug, Default, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ServerReferenceManifest {
  pub ssr_module_mapping: SSRModuleMapping,
  pub server_imports: ServerImports,
  pub server_actions: ServerActions,
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

#[derive(Debug, Clone)]

pub struct RSCAdditionalData {
  pub directives: Vec<String>,
  pub exports: Vec<ExportSpecifier>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ReactRoute {
  pub name: String,
  pub import: String,
}
