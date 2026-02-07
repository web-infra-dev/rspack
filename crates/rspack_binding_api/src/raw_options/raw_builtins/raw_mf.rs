use std::{collections::HashMap, sync::Arc};

use napi::Either;
use napi_derive::napi;
use rspack_plugin_mf::{
  CollectSharedEntryPluginOptions, ConsumeOptions, ConsumeSharedPluginOptions, ConsumeVersion,
  ContainerPluginOptions, ContainerReferencePluginOptions, ExposeOptions, ManifestExposeOption,
  ManifestSharedOption, ModuleFederationManifestPluginOptions,
  ModuleFederationRuntimeExperimentsOptions, ModuleFederationRuntimePluginOptions,
  OptimizeSharedConfig, ProvideOptions, ProvideVersion, RemoteAliasTarget, RemoteOptions,
  SharedContainerPluginOptions, SharedUsedExportsOptimizerPluginOptions, StatsBuildInfo,
};

use crate::options::{
  entry::{JsEntryRuntime, JsEntryRuntimeWrapper},
  library::JsLibraryOptions,
};

pub type RawShareScope = Either<String, Vec<String>>;

fn into_share_scope(value: RawShareScope) -> Vec<String> {
  let scopes = match value {
    Either::A(scope) => vec![scope],
    Either::B(scopes) => scopes,
  };
  if scopes.is_empty() {
    return vec!["default".to_string()];
  }
  scopes
}

#[derive(Debug)]
#[napi(object)]
pub struct RawContainerPluginOptions {
  pub name: String,
  #[napi(ts_type = "string | string[]")]
  pub share_scope: RawShareScope,
  pub library: JsLibraryOptions,
  #[napi(ts_type = "false | string")]
  pub runtime: Option<JsEntryRuntime>,
  pub filename: Option<String>,
  pub exposes: Vec<RawExposeOptions>,
  pub enhanced: bool,
}

impl From<RawContainerPluginOptions> for ContainerPluginOptions {
  fn from(value: RawContainerPluginOptions) -> Self {
    Self {
      name: value.name,
      share_scope: into_share_scope(value.share_scope),
      library: value.library.into(),
      runtime: value.runtime.map(|r| JsEntryRuntimeWrapper(r).into()),
      filename: value.filename.map(|f| f.into()),
      exposes: value.exposes.into_iter().map(|e| e.into()).collect(),
      enhanced: value.enhanced,
    }
  }
}

#[derive(Debug, Clone)]
#[napi(object)]
pub struct RawExposeOptions {
  pub key: String,
  pub name: Option<String>,
  pub layer: Option<String>,
  pub import: Vec<String>,
}

impl From<RawExposeOptions> for (String, ExposeOptions) {
  fn from(value: RawExposeOptions) -> Self {
    (
      value.key,
      ExposeOptions {
        name: value.name,
        layer: value.layer,
        import: value.import,
      },
    )
  }
}

#[derive(Debug)]
#[napi(object)]
pub struct RawContainerReferencePluginOptions {
  pub remote_type: String,
  pub remotes: Vec<RawRemoteOptions>,
  #[napi(ts_type = "string | string[] | undefined")]
  pub share_scope: Option<RawShareScope>,
  pub enhanced: bool,
}

impl From<RawContainerReferencePluginOptions> for ContainerReferencePluginOptions {
  fn from(value: RawContainerReferencePluginOptions) -> Self {
    Self {
      remote_type: value.remote_type,
      remotes: value.remotes.into_iter().map(|e| e.into()).collect(),
      share_scope: value.share_scope.map(into_share_scope),
      enhanced: value.enhanced,
    }
  }
}

#[derive(Debug)]
#[napi(object)]
pub struct RawRemoteOptions {
  pub key: String,
  pub external: Vec<String>,
  #[napi(ts_type = "string | string[]")]
  pub share_scope: RawShareScope,
}

impl From<RawRemoteOptions> for (String, RemoteOptions) {
  fn from(value: RawRemoteOptions) -> Self {
    (
      value.key,
      RemoteOptions {
        external: value.external,
        share_scope: into_share_scope(value.share_scope),
      },
    )
  }
}

#[derive(Debug)]
#[napi(object)]
pub struct RawProvideOptions {
  pub key: String,
  pub request: Option<String>,
  pub layer: Option<String>,
  pub share_key: String,
  #[napi(ts_type = "string | string[]")]
  pub share_scope: RawShareScope,
  #[napi(ts_type = "string | false | undefined")]
  pub version: Option<RawVersion>,
  pub eager: bool,
  pub singleton: Option<bool>,
  #[napi(ts_type = "string | false | undefined")]
  pub required_version: Option<RawVersion>,
  pub strict_version: Option<bool>,
  pub tree_shaking_mode: Option<String>,
}

impl From<RawProvideOptions> for (String, ProvideOptions) {
  fn from(value: RawProvideOptions) -> Self {
    (
      value.key,
      ProvideOptions {
        request: value.request,
        layer: value.layer,
        share_key: value.share_key,
        share_scope: into_share_scope(value.share_scope),
        version: value.version.map(|v| RawVersionWrapper(v).into()),
        eager: value.eager,
        singleton: value.singleton,
        required_version: value.required_version.map(|v| RawVersionWrapper(v).into()),
        strict_version: value.strict_version,
        tree_shaking_mode: value.tree_shaking_mode,
      },
    )
  }
}

#[derive(Debug)]
#[napi(object)]
pub struct RawCollectShareEntryPluginOptions {
  pub consumes: Vec<RawConsumeOptions>,
  pub filename: Option<String>,
}

impl From<RawCollectShareEntryPluginOptions> for CollectSharedEntryPluginOptions {
  fn from(value: RawCollectShareEntryPluginOptions) -> Self {
    Self {
      consumes: value
        .consumes
        .into_iter()
        .map(|provide| {
          let (key, consume_options): (String, ConsumeOptions) = provide.into();
          (key, std::sync::Arc::new(consume_options))
        })
        .collect(),
      filename: value.filename,
    }
  }
}

#[derive(Debug)]
#[napi(object)]
pub struct RawSharedContainerPluginOptions {
  pub name: String,
  pub request: String,
  pub version: String,
  pub file_name: Option<String>,
  pub library: JsLibraryOptions,
}

impl From<RawSharedContainerPluginOptions> for SharedContainerPluginOptions {
  fn from(value: RawSharedContainerPluginOptions) -> Self {
    SharedContainerPluginOptions {
      name: value.name,
      request: value.request,
      version: value.version,
      library: value.library.into(),
      file_name: value.file_name.clone().map(Into::into),
    }
  }
}

#[derive(Debug)]
#[napi(object)]
pub struct RawConsumeSharedPluginOptions {
  pub consumes: Vec<RawConsumeOptions>,
  pub enhanced: bool,
}

impl From<RawConsumeSharedPluginOptions> for ConsumeSharedPluginOptions {
  fn from(value: RawConsumeSharedPluginOptions) -> Self {
    Self {
      consumes: value
        .consumes
        .into_iter()
        .map(|c| c.into())
        .map(|(k, v)| (k, Arc::new(v)))
        .collect(),
      enhanced: value.enhanced,
    }
  }
}

#[derive(Debug)]
#[napi(object)]
pub struct RawOptimizeSharedConfig {
  pub share_key: String,
  pub tree_shaking: bool,
  pub used_exports: Option<Vec<String>>,
}

impl From<RawOptimizeSharedConfig> for OptimizeSharedConfig {
  fn from(value: RawOptimizeSharedConfig) -> Self {
    Self {
      share_key: value.share_key,
      tree_shaking: value.tree_shaking,
      used_exports: value.used_exports.unwrap_or_default(),
    }
  }
}

#[derive(Debug)]
#[napi(object)]
pub struct RawSharedUsedExportsOptimizerPluginOptions {
  pub shared: Vec<RawOptimizeSharedConfig>,
  pub inject_tree_shaking_used_exports: Option<bool>,
  pub manifest_file_name: Option<String>,
  pub stats_file_name: Option<String>,
}

impl From<RawSharedUsedExportsOptimizerPluginOptions> for SharedUsedExportsOptimizerPluginOptions {
  fn from(value: RawSharedUsedExportsOptimizerPluginOptions) -> Self {
    Self {
      shared: value
        .shared
        .into_iter()
        .map(|config| config.into())
        .collect(),
      inject_tree_shaking_used_exports: value.inject_tree_shaking_used_exports.unwrap_or(true),
      manifest_file_name: value
        .manifest_file_name
        .and_then(|s| if s.trim().is_empty() { None } else { Some(s) }),
      stats_file_name: value
        .stats_file_name
        .and_then(|s| if s.trim().is_empty() { None } else { Some(s) }),
    }
  }
}

#[derive(Debug)]
#[napi(object)]
pub struct RawConsumeOptions {
  pub key: String,
  pub request: Option<String>,
  pub issuer_layer: Option<String>,
  pub layer: Option<String>,
  pub import: Option<String>,
  pub import_resolved: Option<String>,
  pub share_key: String,
  #[napi(ts_type = "string | string[]")]
  pub share_scope: RawShareScope,
  #[napi(ts_type = "string | false | undefined")]
  pub required_version: Option<RawVersion>,
  pub package_name: Option<String>,
  pub strict_version: bool,
  pub singleton: bool,
  pub eager: bool,
  pub tree_shaking_mode: Option<String>,
}

impl From<RawConsumeOptions> for (String, ConsumeOptions) {
  fn from(value: RawConsumeOptions) -> Self {
    (
      value.key,
      ConsumeOptions {
        request: value.request,
        issuer_layer: value.issuer_layer,
        layer: value.layer,
        import: value.import,
        import_resolved: value.import_resolved,
        share_key: value.share_key,
        share_scope: into_share_scope(value.share_scope),
        required_version: value.required_version.map(|v| RawVersionWrapper(v).into()),
        package_name: value.package_name,
        strict_version: value.strict_version,
        singleton: value.singleton,
        eager: value.eager,
        tree_shaking_mode: value.tree_shaking_mode,
      },
    )
  }
}

pub type RawVersion = Either<String, bool>;

struct RawVersionWrapper(RawVersion);

impl From<RawVersionWrapper> for ProvideVersion {
  fn from(value: RawVersionWrapper) -> Self {
    match value.0 {
      Either::A(s) => ProvideVersion::Version(s),
      Either::B(_) => ProvideVersion::False,
    }
  }
}

impl From<RawVersionWrapper> for ConsumeVersion {
  fn from(value: RawVersionWrapper) -> Self {
    match value.0 {
      Either::A(s) => ConsumeVersion::Version(s),
      Either::B(_) => ConsumeVersion::False,
    }
  }
}

#[derive(Debug)]
#[napi(object)]
pub struct RawModuleFederationRuntimePluginOptions {
  #[napi(ts_type = "string | undefined")]
  pub entry_runtime: Option<String>,
  pub experiments: Option<RawModuleFederationRuntimeExperimentsOptions>,
}

impl From<RawModuleFederationRuntimePluginOptions> for ModuleFederationRuntimePluginOptions {
  fn from(value: RawModuleFederationRuntimePluginOptions) -> Self {
    Self {
      entry_runtime: value.entry_runtime,
      experiments: value.experiments.map(Into::into).unwrap_or_default(),
    }
  }
}

#[derive(Debug, Default)]
#[napi(object)]
pub struct RawModuleFederationRuntimeExperimentsOptions {
  #[napi(js_name = "asyncStartup")]
  pub async_startup: Option<bool>,
  pub rsc: Option<bool>,
}

impl From<RawModuleFederationRuntimeExperimentsOptions>
  for ModuleFederationRuntimeExperimentsOptions
{
  fn from(value: RawModuleFederationRuntimeExperimentsOptions) -> Self {
    Self {
      async_startup: value.async_startup.unwrap_or(false),
      rsc: value.rsc.unwrap_or(false),
    }
  }
}

#[derive(Debug)]
#[napi(object)]
pub struct RawRemoteAliasTarget {
  pub name: String,
  pub entry: Option<String>,
}

#[derive(Debug)]
#[napi(object)]
pub struct RawManifestExposeOption {
  pub path: String,
  pub name: String,
}

#[derive(Debug)]
#[napi(object)]
pub struct RawManifestSharedOption {
  pub name: String,
  pub version: Option<String>,
  pub required_version: Option<String>,
  pub singleton: Option<bool>,
}

#[derive(Debug)]
#[napi(object)]
pub struct RawStatsBuildInfo {
  pub build_version: String,
  pub build_name: Option<String>,
  // only appear when enable tree_shaking
  pub target: Option<Vec<String>>,
  pub plugins: Option<Vec<String>>,
}

#[derive(Debug)]
#[napi(object)]
pub struct RawModuleFederationManifestPluginOptions {
  pub name: Option<String>,
  pub global_name: Option<String>,
  pub file_name: Option<String>,
  pub file_path: Option<String>,
  pub stats_file_name: Option<String>,
  pub manifest_file_name: Option<String>,
  pub disable_assets_analyze: Option<bool>,
  pub remote_alias_map: Option<HashMap<String, RawRemoteAliasTarget>>,
  pub exposes: Option<Vec<RawManifestExposeOption>>,
  pub shared: Option<Vec<RawManifestSharedOption>>,
  pub build_info: Option<RawStatsBuildInfo>,
}

impl From<RawModuleFederationManifestPluginOptions> for ModuleFederationManifestPluginOptions {
  fn from(value: RawModuleFederationManifestPluginOptions) -> Self {
    ModuleFederationManifestPluginOptions {
      name: value.name,
      global_name: value.global_name,
      stats_file_name: value.stats_file_name.unwrap_or_default(),
      manifest_file_name: value.manifest_file_name.unwrap_or_default(),
      disable_assets_analyze: value.disable_assets_analyze.unwrap_or(false),
      remote_alias_map: value
        .remote_alias_map
        .unwrap_or_default()
        .into_iter()
        .map(|(k, v)| {
          (
            k,
            RemoteAliasTarget {
              name: v.name,
              entry: v.entry,
            },
          )
        })
        .collect::<HashMap<String, RemoteAliasTarget>>(),
      exposes: value
        .exposes
        .unwrap_or_default()
        .into_iter()
        .map(|expose| ManifestExposeOption {
          path: expose.path,
          name: expose.name,
        })
        .collect(),
      shared: value
        .shared
        .unwrap_or_default()
        .into_iter()
        .map(|shared| ManifestSharedOption {
          name: shared.name,
          version: shared.version,
          required_version: shared.required_version,
          singleton: shared.singleton,
        })
        .collect(),
      build_info: value.build_info.map(|info| StatsBuildInfo {
        build_version: info.build_version,
        build_name: info.build_name,
        target: info.target,
        plugins: info.plugins,
      }),
    }
  }
}
