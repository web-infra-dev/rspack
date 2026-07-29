mod container;
mod manifest;
mod sharing;

use rspack_hash::{RspackHash, RspackHasher};

pub(crate) fn push_identifier_component(key: &mut String, value: &str) {
  key.push_str(&value.len().to_string());
  key.push(':');
  key.push_str(value);
}

#[rspack_cacheable::cacheable(hashable)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum ShareScope {
  Single(String),
  Multiple(Vec<String>),
}

impl ShareScope {
  pub fn key(&self) -> String {
    match self {
      ShareScope::Single(scope) => scope.clone(),
      ShareScope::Multiple(scopes) => scopes.join("|"),
    }
  }

  /// Stable, collision-free representation for internal module identifiers.
  /// Runtime and display values must continue to use [`Self::scopes`] or [`Self::key`].
  pub(crate) fn identifier_key(&self) -> String {
    let mut key = String::new();
    match self {
      ShareScope::Single(scope) => {
        key.push('s');
        push_identifier_component(&mut key, scope);
      }
      ShareScope::Multiple(scopes) => {
        key.push('m');
        key.push_str(&scopes.len().to_string());
        key.push(':');
        for scope in scopes {
          push_identifier_component(&mut key, scope);
        }
      }
    }
    key
  }

  pub(crate) fn identifier_fragment(&self) -> String {
    match self {
      ShareScope::Single(scope) => format!("({scope})"),
      ShareScope::Multiple(_) => format!("[{}]", self.identifier_key()),
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct SharedIdentity {
  pub(crate) share_scope: ShareScope,
  pub(crate) share_key: String,
  pub(crate) layer: Option<String>,
}

impl SharedIdentity {
  pub(crate) fn new(share_scope: &ShareScope, share_key: &str, layer: Option<&str>) -> Self {
    Self {
      share_scope: share_scope.clone(),
      share_key: share_key.to_string(),
      layer: layer.map(str::to_string),
    }
  }

  pub(crate) fn identifier_key(&self) -> String {
    let mut key = String::new();
    push_identifier_component(&mut key, &self.share_scope.identifier_key());
    match &self.layer {
      Some(layer) => {
        key.push('l');
        push_identifier_component(&mut key, layer);
      }
      None => key.push('n'),
    }
    push_identifier_component(&mut key, &self.share_key);
    key
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

#[cfg(test)]
mod tests {
  use super::{ShareScope, SharedIdentity};

  #[test]
  fn share_scope_identifier_key_is_collision_free() {
    let single = ShareScope::Single("a|b".to_string());
    let multiple = ShareScope::Multiple(vec!["a".to_string(), "b".to_string()]);

    assert_eq!(single.key(), multiple.key());
    assert_ne!(single.identifier_key(), multiple.identifier_key());
    assert_ne!(single.identifier_fragment(), multiple.identifier_fragment());
    assert_ne!(
      ShareScope::Multiple(vec!["a:".to_string(), "b".to_string()]).identifier_key(),
      ShareScope::Multiple(vec!["a".to_string(), ":b".to_string()]).identifier_key()
    );
  }

  #[test]
  fn shared_identity_key_is_collision_free() {
    let scope = ShareScope::Single("default".to_string());
    let first = SharedIdentity::new(&scope, "c", Some("a) b"));
    let second = SharedIdentity::new(&scope, "b) c", Some("a"));
    let unlayered = SharedIdentity::new(&scope, "(a) b) c", None);

    assert_ne!(first.identifier_key(), second.identifier_key());
    assert_ne!(first.identifier_key(), unlayered.identifier_key());
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
  use rspack_core::{
    Compilation, ModuleCodeTemplate, RuntimeCodeTemplate, RuntimeGlobals, RuntimeVariable,
    runtime_mode::RuntimeMode,
  };
  use serde::Serialize;

  pub fn json_stringify<T: ?Sized + Serialize>(v: &T) -> String {
    simd_json::to_string(v).unwrap()
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
