use std::sync::Arc;

use napi::Either;
use napi_derive::napi;
use rspack_plugin_mf::{
  ConsumeOptions, ConsumeSharedPluginOptions, ConsumeVersion, ContainerPluginOptions,
  ContainerReferencePluginOptions, ExposeOptions, ProvideOptions, ProvideVersion, RemoteOptions,
};

use crate::{RawEntryRuntime, RawEntryRuntimeWrapper, RawLibraryOptions};

#[derive(Debug)]
#[napi(object)]
pub struct RawContainerPluginOptions {
  pub name: String,
  pub share_scope: String,
  pub library: RawLibraryOptions,
  #[napi(ts_type = "false | string")]
  pub runtime: Option<RawEntryRuntime>,
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
      runtime: value.runtime.map(|r| RawEntryRuntimeWrapper(r).into()),
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
