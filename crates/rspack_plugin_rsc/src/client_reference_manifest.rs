use rustc_hash::{FxHashMap, FxHashSet};
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

#[derive(Debug, Clone, Serialize)]
pub struct ClientReferenceManifest {
  #[serde(rename = "clientModules")]
  pub client_modules: ManifestNode,
  #[serde(rename = "moduleLoading")]
  pub module_loading: ModuleLoading,
  #[serde(rename = "ssrModuleMapping")]
  pub ssr_module_mapping: FxHashMap<String, ManifestNode>,
  #[serde(rename = "entryCSSFiles")]
  pub entry_css_files: FxHashMap<String, FxHashSet<String>>,
  #[serde(rename = "entryJSFiles")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub entry_js_files: Option<FxHashMap<String, Vec<String>>>,
}
