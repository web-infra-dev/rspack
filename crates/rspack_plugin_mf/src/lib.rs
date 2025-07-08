#![feature(let_chains)]

mod container;
mod sharing;

pub use container::{
  container_plugin::{ContainerPlugin, ContainerPluginOptions, ExposeOptions},
  container_reference_plugin::{
    ContainerReferencePlugin, ContainerReferencePluginOptions, RemoteOptions,
  },
  module_federation_runtime_plugin::ModuleFederationRuntimePlugin,
};
pub use sharing::{
  consume_shared_plugin::{
    ConsumeOptions, ConsumeSharedPlugin, ConsumeSharedPluginOptions, ConsumeVersion,
  },
  provide_shared_plugin::{ProvideOptions, ProvideSharedPlugin, ProvideVersion},
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
