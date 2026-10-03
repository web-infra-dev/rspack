use rspack_util::fx_hash::FxHashMap as HashMap;

use crate::{ShareScope, manifest::data::StatsBuildInfo};

#[derive(Debug, Clone)]
pub struct RemoteAliasTarget {
  pub name: String,
  pub entry: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ManifestExposeOption {
  pub path: String,
  pub name: String,
}

#[derive(Debug, Clone)]
pub struct ManifestSharedOption {
  pub name: String,
  pub version: Option<String>,
  pub required_version: Option<String>,
  pub singleton: Option<bool>,
}

#[derive(Debug, Clone, Default)]
pub struct ModuleFederationManifestPluginOptions {
  pub name: Option<String>,
  pub global_name: Option<String>,
  pub stats_file_name: String,
  pub manifest_file_name: String,
  pub disable_assets_analyze: bool,
  pub remote_alias_map: HashMap<String, RemoteAliasTarget>,
  pub exposes: Vec<ManifestExposeOption>,
  pub shared: Vec<ManifestSharedOption>,
  pub build_info: Option<StatsBuildInfo>,
}

impl From<ManifestExposeOption> for EnhancedManifestExposeOption {
  fn from(value: ManifestExposeOption) -> Self {
    Self {
      path: value.path,
      name: value.name,
      layer: None,
    }
  }
}

impl From<ManifestSharedOption> for EnhancedManifestSharedOption {
  fn from(value: ManifestSharedOption) -> Self {
    Self {
      name: value.name,
      version: value.version,
      required_version: value.required_version,
      singleton: value.singleton,
      share_scope: ShareScope::Single("default".to_string()),
      layer: None,
    }
  }
}

impl From<ModuleFederationManifestPluginOptions> for EnhancedModuleFederationManifestPluginOptions {
  fn from(value: ModuleFederationManifestPluginOptions) -> Self {
    Self {
      name: value.name,
      global_name: value.global_name,
      stats_file_name: value.stats_file_name,
      manifest_file_name: value.manifest_file_name,
      disable_assets_analyze: value.disable_assets_analyze,
      remote_alias_map: value.remote_alias_map,
      exposes: value.exposes.into_iter().map(Into::into).collect(),
      shared: value.shared.into_iter().map(Into::into).collect(),
      build_info: value.build_info,
    }
  }
}

/// Manifest expose metadata with an optional compilation layer.
#[derive(Debug, Clone)]
pub struct EnhancedManifestExposeOption {
  pub path: String,
  pub name: String,
  pub layer: Option<String>,
}

#[derive(Debug, Clone)]
pub struct EnhancedManifestSharedOption {
  pub name: String,
  pub version: Option<String>,
  pub required_version: Option<String>,
  pub share_scope: ShareScope,
  pub layer: Option<String>,
  pub singleton: Option<bool>,
}

#[derive(Debug, Clone, Default)]
pub struct EnhancedModuleFederationManifestPluginOptions {
  pub name: Option<String>,
  pub global_name: Option<String>,
  pub stats_file_name: String,
  pub manifest_file_name: String,
  pub disable_assets_analyze: bool,
  pub remote_alias_map: HashMap<String, RemoteAliasTarget>,
  pub exposes: Vec<EnhancedManifestExposeOption>,
  pub shared: Vec<EnhancedManifestSharedOption>,
  pub build_info: Option<StatsBuildInfo>,
}
