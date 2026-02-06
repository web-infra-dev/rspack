use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub(crate) struct StatsAssetsGroup {
  #[serde(default)]
  pub js: AssetsSplit,
  #[serde(default)]
  pub css: AssetsSplit,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub(crate) struct AssetsSplit {
  #[serde(default)]
  pub sync: Vec<String>,
  #[serde(default)]
  pub r#async: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct StatsBuildInfo {
  #[serde(rename = "buildVersion")]
  pub build_version: String,
  #[serde(rename = "buildName", skip_serializing_if = "Option::is_none")]
  pub build_name: Option<String>,
  #[serde(rename = "target", skip_serializing_if = "Option::is_none")]
  pub target: Option<Vec<String>>,
  #[serde(rename = "plugins", skip_serializing_if = "Option::is_none")]
  pub plugins: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct StatsExpose {
  pub path: String,
  #[serde(default)]
  pub file: String,
  pub id: String,
  pub name: String,
  #[serde(default)]
  pub requires: Vec<String>,
  #[serde(default)]
  pub assets: StatsAssetsGroup,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct StatsShared {
  pub id: String,
  pub name: String,
  pub version: String,
  #[serde(default)]
  pub requiredVersion: Option<String>,
  #[serde(default)]
  pub singleton: Option<bool>,
  #[serde(default)]
  pub assets: StatsAssetsGroup,
  #[serde(default)]
  pub usedIn: Vec<String>,
  #[serde(default)]
  pub usedExports: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct StatsRemote {
  pub alias: String,
  pub consumingFederationContainerName: String,
  pub federationContainerName: String,
  pub moduleName: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub entry: Option<String>,
  #[serde(default)]
  pub usedIn: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct BasicStatsMetaData {
  pub name: String,
  pub globalName: String,
  #[serde(rename = "buildInfo", skip_serializing_if = "Option::is_none")]
  pub build_info: Option<StatsBuildInfo>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub publicPath: Option<String>,
  #[serde(default)]
  pub remoteEntry: RemoteEntryMeta,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub r#type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub(crate) struct RemoteEntryMeta {
  #[serde(default)]
  pub name: String,
  #[serde(default)]
  pub path: String,
  #[serde(default)]
  pub r#type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct StatsRoot {
  pub id: String,
  pub name: String,
  pub metaData: BasicStatsMetaData,
  #[serde(default)]
  pub shared: Vec<StatsShared>,
  #[serde(default)]
  pub remotes: Vec<StatsRemote>,
  #[serde(default)]
  pub exposes: Vec<StatsExpose>,
}

#[derive(Debug, Serialize, Clone)]
pub(super) struct ManifestExpose {
  pub id: String,
  pub name: String,
  pub path: String,
  pub assets: StatsAssetsGroup,
}

#[derive(Debug, Serialize, Clone)]
pub(super) struct ManifestShared {
  pub id: String,
  pub name: String,
  pub version: String,
  #[serde(default)]
  pub requiredVersion: Option<String>,
  #[serde(default)]
  pub singleton: Option<bool>,
  #[serde(default)]
  pub assets: StatsAssetsGroup,
}

#[derive(Debug, Serialize, Clone)]
pub(super) struct ManifestRemote {
  pub federationContainerName: String,
  pub moduleName: String,
  pub alias: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub entry: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub(super) struct ManifestRoot {
  pub id: String,
  pub name: String,
  pub metaData: BasicStatsMetaData,
  #[serde(default)]
  pub shared: Vec<ManifestShared>,
  #[serde(default)]
  pub remotes: Vec<ManifestRemote>,
  #[serde(default)]
  pub exposes: Vec<ManifestExpose>,
}
