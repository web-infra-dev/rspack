mod container;
mod manifest;
mod sharing;

use rspack_hash::{RspackHash, RspackHasher};

#[rspack_cacheable::cacheable]
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize)]
#[serde(untagged)]
pub enum ShareScope {
  Single(String),
  Multiple(Vec<String>),
}

impl ShareScope {
  pub fn key(&self) -> String {
    match self {
      ShareScope::Single(s) => s.clone(),
      ShareScope::Multiple(v) => v.join("|"),
    }
  }

  pub fn scopes(&self) -> &[String] {
    match self {
      ShareScope::Single(s) => std::slice::from_ref(s),
      ShareScope::Multiple(v) => v.as_slice(),
    }
  }

  pub fn is_empty(&self) -> bool {
    match self {
      ShareScope::Single(_) => false,
      ShareScope::Multiple(v) => v.is_empty(),
    }
  }
}

impl RspackHash for ShareScope {
  fn hash(&self, state: &mut RspackHasher) {
    match self {
      ShareScope::Single(scope) => scope.hash(state),
      ShareScope::Multiple(scopes) => scopes.hash(state),
    }
  }
}

pub use container::{
  container_plugin::{ContainerPlugin, ContainerPluginOptions, ExposeOptions},
  container_reference_plugin::{
    ContainerReferencePlugin, ContainerReferencePluginOptions, RemoteOptions,
  },
  embed_federation_runtime_module::EmbedFederationRuntimeModule,
  module_federation_runtime_plugin::{
    ModuleFederationRuntimeExperimentsOptions, ModuleFederationRuntimePlugin,
    ModuleFederationRuntimePluginOptions,
  },
};
pub use manifest::{
  ManifestExposeOption, ManifestSharedOption, ModuleFederationManifestPlugin,
  ModuleFederationManifestPluginOptions, RemoteAliasTarget, StatsBuildInfo,
};
pub use sharing::{
  collect_shared_entry_plugin::{CollectSharedEntryPlugin, CollectSharedEntryPluginOptions},
  consume_shared_module::ConsumeSharedModule,
  consume_shared_plugin::{
    ConsumeOptions, ConsumeSharedPlugin, ConsumeSharedPluginOptions, ConsumeVersion,
  },
  provide_shared_module::ProvideSharedModule,
  provide_shared_plugin::{ProvideOptions, ProvideSharedPlugin, ProvideVersion},
  share_runtime_module::{
    CodeGenerationDataShareInit, DataInitStage, ShareInitData, ShareRuntimeModule,
  },
  share_runtime_plugin::ShareRuntimePlugin,
  shared_container_plugin::{SharedContainerPlugin, SharedContainerPluginOptions},
  shared_used_exports_optimizer_plugin::{
    OptimizeSharedConfig, SharedUsedExportsOptimizerPlugin, SharedUsedExportsOptimizerPluginOptions,
  },
};

mod utils {
  use std::fmt;

  use rspack_core::{
    Compilation, ModuleCodeTemplate, RuntimeCodeTemplate, RuntimeGlobals, RuntimeVariable,
    runtime_mode::RuntimeMode,
  };
  use serde::Serialize;

  pub fn json_stringify<T: ?Sized + Serialize + fmt::Debug>(v: &T) -> String {
    simd_json::to_string(v).unwrap_or_else(|e| panic!("{e}: {v:?} should able to json stringify"))
  }

  pub fn module_identifier_namespace(runtime_mode: RuntimeMode) -> &'static str {
    match runtime_mode {
      RuntimeMode::Webpack => "webpack",
      RuntimeMode::Rspack => "rspack",
    }
  }

  pub fn runtime_require_scope_name(runtime_template: &RuntimeCodeTemplate) -> String {
    if runtime_template.render_mode().is_legacy() {
      runtime_template.render_runtime_globals(&RuntimeGlobals::REQUIRE)
    } else {
      runtime_template.render_runtime_variable(&RuntimeVariable::Context)
    }
  }

  pub fn runtime_require_scope_requirement(compilation: &Compilation) -> RuntimeGlobals {
    if compilation.options.experiments.runtime_mode == RuntimeMode::Rspack {
      RuntimeGlobals::REQUIRE_SCOPE
    } else {
      RuntimeGlobals::default()
    }
  }

  pub fn module_require_scope_name(
    compilation: &Compilation,
    runtime_template: &mut ModuleCodeTemplate,
  ) -> String {
    if compilation.options.experiments.runtime_mode == RuntimeMode::Rspack {
      runtime_template
        .runtime_requirements_mut()
        .insert(RuntimeGlobals::REQUIRE_SCOPE);
      runtime_template.render_runtime_variable(&RuntimeVariable::Context)
    } else {
      runtime_template.render_runtime_globals(&RuntimeGlobals::REQUIRE)
    }
  }
}
