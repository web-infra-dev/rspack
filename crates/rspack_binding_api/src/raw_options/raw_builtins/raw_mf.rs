use std::{collections::HashMap, sync::Arc};

use napi::Either;
use napi_derive::napi;
use rspack_plugin_mf::{
  ConsumeOptions, ConsumeSharedPluginOptions, ConsumeVersion, ContainerPluginOptions,
  ContainerReferencePluginOptions, ExposeOptions, ManifestExposeOption, ManifestSharedOption,
  ModuleFederationManifestPluginOptions, ModuleFederationRuntimePluginOptions, ProvideOptions,
  ProvideVersion, RemoteAliasTarget, RemoteOptions, StatsBuildInfo,
};

use crate::options::{
  entry::{JsEntryRuntime, JsEntryRuntimeWrapper},
  library::JsLibraryOptions,
};

#[derive(Debug)]
#[napi(object)]
pub struct RawContainerPluginOptions {
  pub name: String,
  pub share_scope: String,
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
      share_scope: value.share_scope,
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
  pub import: Vec<String>,
}

impl From<RawExposeOptions> for (String, ExposeOptions) {
  fn from(value: RawExposeOptions) -> Self {
    (
      value.key,
      ExposeOptions {
        name: value.name,
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
  pub share_scope: Option<String>,
  pub enhanced: bool,
}

impl From<RawContainerReferencePluginOptions> for ContainerReferencePluginOptions {
  fn from(value: RawContainerReferencePluginOptions) -> Self {
    Self {
      remote_type: value.remote_type,
      remotes: value.remotes.into_iter().map(|e| e.into()).collect(),
      share_scope: value.share_scope,
      enhanced: value.enhanced,
    }
  }
}

#[derive(Debug)]
#[napi(object)]
pub struct RawRemoteOptions {
  pub key: String,
  pub external: Vec<String>,
  pub share_scope: String,
}

impl From<RawRemoteOptions> for (String, RemoteOptions) {
  fn from(value: RawRemoteOptions) -> Self {
    (
      value.key,
      RemoteOptions {
        external: value.external,
        share_scope: value.share_scope,
      },
    )
  }
}

#[derive(Debug)]
#[napi(object)]
pub struct RawProvideOptions {
  pub key: String,
  pub share_key: String,
  pub share_scope: String,
  #[napi(ts_type = "string | false | undefined")]
  pub version: Option<RawVersion>,
  pub eager: bool,
  pub singleton: Option<bool>,
  #[napi(ts_type = "string | false | undefined")]
  pub required_version: Option<RawVersion>,
  pub strict_version: Option<bool>,
}

impl From<RawProvideOptions> for (String, ProvideOptions) {
  fn from(value: RawProvideOptions) -> Self {
    (
      value.key,
      ProvideOptions {
        share_key: value.share_key,
        share_scope: value.share_scope,
        version: value.version.map(|v| RawVersionWrapper(v).into()),
        eager: value.eager,
        singleton: value.singleton,
        required_version: value.required_version.map(|v| RawVersionWrapper(v).into()),
        strict_version: value.strict_version,
      },
    )
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
pub struct RawConsumeOptions {
  pub key: String,
  pub import: Option<String>,
  pub import_resolved: Option<String>,
  pub share_key: String,
  pub share_scope: String,
  #[napi(ts_type = "string | false | undefined")]
  pub required_version: Option<RawVersion>,
  pub package_name: Option<String>,
  pub strict_version: bool,
  pub singleton: bool,
  pub eager: bool,
}

impl From<RawConsumeOptions> for (String, ConsumeOptions) {
  fn from(value: RawConsumeOptions) -> Self {
    (
      value.key,
      ConsumeOptions {
        import: value.import,
        import_resolved: value.import_resolved,
        share_key: value.share_key,
        share_scope: value.share_scope,
        required_version: value.required_version.map(|v| RawVersionWrapper(v).into()),
        package_name: value.package_name,
        strict_version: value.strict_version,
        singleton: value.singleton,
        eager: value.eager,
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
}

impl From<RawModuleFederationRuntimePluginOptions> for ModuleFederationRuntimePluginOptions {
  fn from(value: RawModuleFederationRuntimePluginOptions) -> Self {
    Self {
      entry_runtime: value.entry_runtime,
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
      }),
    }
  }
}
