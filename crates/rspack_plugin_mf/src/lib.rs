mod container;
mod manifest;
mod sharing;

pub use container::{
  container_plugin::{ContainerPlugin, ContainerPluginOptions, ExposeOptions},
  container_reference_plugin::{
    ContainerReferencePlugin, ContainerReferencePluginOptions, RemoteOptions,
  },
  module_federation_runtime_plugin::{
    ModuleFederationRuntimePlugin, ModuleFederationRuntimePluginOptions,
  },
};
pub use manifest::{
  ManifestExposeOption, ManifestSharedOption, ModuleFederationManifestPlugin,
  ModuleFederationManifestPluginOptions, RemoteAliasTarget, StatsBuildInfo,
};
pub use sharing::{
  collect_share_entry_plugin::{CollectShareEntryPlugin, CollectShareEntryPluginOptions},
  consume_shared_module::ConsumeSharedModule,
  consume_shared_plugin::{
    ConsumeOptions, ConsumeSharedPlugin, ConsumeSharedPluginOptions, ConsumeVersion,
  },
  optimize_dependency_referenced_exports_plugin::{
    OptimizeDependencyReferencedExportsPlugin, OptimizeDependencyReferencedExportsPluginOptions,
    OptimizeSharedConfig,
  },
  provide_shared_module::ProvideSharedModule,
  provide_shared_plugin::{ProvideOptions, ProvideSharedPlugin, ProvideVersion},
  share_container_entry_dependency::ShareContainerEntryOptions,
  share_container_plugin::{ShareContainerPlugin, ShareContainerPluginOptions},
  share_runtime_module::{
    CodeGenerationDataShareInit, DataInitStage, ShareInitData, ShareRuntimeModule,
  },
  share_runtime_plugin::ShareRuntimePlugin,
  share_usage_plugin::{ShareUsagePlugin, ShareUsagePluginOptions},
};

mod utils {
  use std::fmt;

  use serde::Serialize;

  pub fn json_stringify<T: ?Sized + Serialize + fmt::Debug>(v: &T) -> String {
    serde_json::to_string(v).unwrap_or_else(|e| panic!("{e}: {v:?} should able to json stringify"))
  }
}
