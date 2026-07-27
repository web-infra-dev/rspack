use serde::Serialize;

use crate::ShareScope;

#[derive(Debug, Serialize, Clone, Default)]
pub struct StatsAssetsGroup {
  #[serde(default)]
  pub js: AssetsSplit,
  #[serde(default)]
  pub css: AssetsSplit,
}

#[derive(Debug, Serialize, Clone, Default)]
pub struct AssetsSplit {
  #[serde(default)]
  pub sync: Vec<String>,
  #[serde(default)]
  pub r#async: Vec<String>,
}

#[derive(Debug, Serialize, Clone, Default)]
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

#[derive(Debug, Serialize, Clone)]
pub struct StatsExpose {
  pub path: String,
  #[serde(default)]
  pub file: String,
  pub id: String,
  pub name: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub layer: Option<String>,
  #[serde(default)]
  pub requires: Vec<String>,
  #[serde(
    default,
    rename = "requiredShared",
    skip_serializing_if = "Vec::is_empty"
  )]
  pub required_shared: Vec<StatsSharedRequirement>,
  #[serde(default)]
  pub assets: StatsAssetsGroup,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct StatsSharedRequirement {
  pub name: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub layer: Option<String>,
  #[serde(rename = "shareScope", skip_serializing_if = "Option::is_none")]
  pub share_scope: Option<ShareScope>,
}

#[derive(Debug, Serialize, Clone)]
pub struct StatsShared {
  pub id: String,
  #[serde(rename = "identityId", skip_serializing_if = "Option::is_none")]
  pub identity_id: Option<String>,
  pub name: String,
  pub version: String,
  #[serde(default)]
  pub requiredVersion: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub layer: Option<String>,
  #[serde(rename = "shareScope", skip_serializing_if = "Option::is_none")]
  pub share_scope: Option<ShareScope>,
  #[serde(default)]
  pub singleton: Option<bool>,
  #[serde(default)]
  pub assets: StatsAssetsGroup,
  #[serde(default)]
  pub usedIn: Vec<String>,
  #[serde(default)]
  pub usedExports: Vec<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct StatsRemote {
  pub alias: String,
  pub consumingFederationContainerName: String,
  pub federationContainerName: String,
  pub moduleName: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub entry: Option<String>,
  #[serde(default)]
  pub usedIn: Vec<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct BasicStatsMetaData {
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

#[derive(Debug, Serialize, Clone, Default)]
pub struct RemoteEntryMeta {
  #[serde(default)]
  pub name: String,
  #[serde(default)]
  pub path: String,
  #[serde(default)]
  pub r#type: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct StatsRoot {
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
pub struct ManifestExpose {
  pub id: String,
  pub name: String,
  pub path: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub layer: Option<String>,
  #[serde(
    default,
    rename = "requiredShared",
    skip_serializing_if = "Vec::is_empty"
  )]
  pub required_shared: Vec<StatsSharedRequirement>,
  pub assets: StatsAssetsGroup,
}

#[derive(Debug, Serialize, Clone)]
pub struct ManifestShared {
  pub id: String,
  #[serde(rename = "identityId", skip_serializing_if = "Option::is_none")]
  pub identity_id: Option<String>,
  pub name: String,
  pub version: String,
  #[serde(default)]
  pub requiredVersion: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub layer: Option<String>,
  #[serde(rename = "shareScope", skip_serializing_if = "Option::is_none")]
  pub share_scope: Option<ShareScope>,
  #[serde(default)]
  pub singleton: Option<bool>,
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub usedExports: Vec<String>,
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub referenceExports: Vec<String>,
  #[serde(default)]
  pub assets: StatsAssetsGroup,
}

#[derive(Debug, Serialize, Clone)]
pub struct ManifestRemote {
  pub federationContainerName: String,
  pub moduleName: String,
  pub alias: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub entry: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct ManifestRoot {
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
