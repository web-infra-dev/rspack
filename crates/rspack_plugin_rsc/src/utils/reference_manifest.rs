use std::collections::HashMap;

use serde::Serialize;

#[derive(Debug, Default, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ServerReferenceManifest {
  pub ssr_module_mapping: HashMap<String, HashMap<String, ServerRef>>,
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
  pub ssr_module_mapping: HashMap<String, HashMap<String, ServerRef>>,
}

#[derive(Debug, Serialize, Clone)]
pub struct ClientRef {
  pub id: String,
  pub name: String,
  pub chunks: Vec<String>,
}
