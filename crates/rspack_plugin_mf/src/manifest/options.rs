use rspack_util::fx_hash::FxHashMap as HashMap;

use crate::manifest::data::StatsBuildInfo;

#[derive(Debug, Clone)]
#[cfg_attr(allocative, derive(allocative::Allocative))]
pub struct RemoteAliasTarget {
  pub name: String,
  pub entry: Option<String>,
}

#[derive(Debug, Clone)]
#[cfg_attr(allocative, derive(allocative::Allocative))]
pub struct ManifestExposeOption {
  pub path: String,
  pub name: String,
}

#[derive(Debug, Clone)]
#[cfg_attr(allocative, derive(allocative::Allocative))]
pub struct ManifestSharedOption {
  pub name: String,
  pub version: Option<String>,
  pub required_version: Option<String>,
  pub singleton: Option<bool>,
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(allocative, derive(allocative::Allocative))]
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

#[cfg(allocative)]
use rspack_util::allocative;
