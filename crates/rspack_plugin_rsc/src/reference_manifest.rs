use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct ManifestExport {
  /// Rspack module id
  pub id: String,
  /// Export name
  pub name: String,
  /// Chunks for the module. JS and CSS.
  pub chunks: Vec<String>,
  /// If chunk contains async module
  #[serde(skip_serializing_if = "Option::is_none")]
  pub r#async: Option<bool>,
}

pub type ManifestNode = FxHashMap<String, ManifestExport>;

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ClientReferenceResolution {
  Local,
  Shared {
    share_key: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    share_scope: Vec<String>,
  },
  Remote {
    request: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    share_scope: Vec<String>,
  },
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ClientReferenceManifestEntry {
  pub export_name: String,
  pub module_id: String,
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub chunks: Vec<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub r#async: Option<bool>,
  pub resolution: ClientReferenceResolution,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActionReferenceManifestEntry {
  pub local_action_id: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub export_name: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub module_resource: Option<String>,
}

pub type ClientReferenceManifest = FxHashMap<String, ClientReferenceManifestEntry>;
pub type ActionReferenceManifest = FxHashMap<String, ActionReferenceManifestEntry>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CrossOriginMode {
  #[serde(rename = "use-credentials")]
  UseCredentials,
  #[serde(rename = "")]
  Anonymous,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleLoading {
  pub prefix: String,
  #[serde(rename = "crossOrigin")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub cross_origin: Option<CrossOriginMode>,
}

pub type ServerReferenceManifest = FxHashMap<String, ManifestExport>;
