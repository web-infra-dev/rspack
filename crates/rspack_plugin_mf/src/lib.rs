#![feature(let_chains)]
#![feature(hash_raw_entry)]

mod container;
mod sharing;

pub use container::container_plugin::{ContainerPlugin, ContainerPluginOptions, ExposeOptions};
pub use container::container_reference_plugin::{
  ContainerReferencePlugin, ContainerReferencePluginOptions, RemoteOptions,
};
pub use container::federation_runtime::FederationRuntimePlugin;
pub use sharing::consume_shared_plugin::{
  ConsumeOptions, ConsumeSharedPlugin, ConsumeSharedPluginOptions, ConsumeVersion,
};
pub use sharing::provide_shared_plugin::{ProvideOptions, ProvideSharedPlugin, ProvideVersion};
pub use sharing::share_runtime_module::{
  CodeGenerationDataShareInit, DataInitStage, ShareInitData, ShareRuntimeModule,
};
pub use sharing::share_runtime_plugin::ShareRuntimePlugin;

mod utils {
  use std::fmt;

  use serde::Serialize;

  pub fn json_stringify<T: ?Sized + Serialize + fmt::Debug>(v: &T) -> String {
    serde_json::to_string(v).unwrap_or_else(|e| panic!("{e}: {v:?} should able to json stringify"))
  }
}
