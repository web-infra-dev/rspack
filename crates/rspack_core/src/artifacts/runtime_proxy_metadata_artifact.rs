use std::{
  fmt::Write,
  ops::{Deref, DerefMut},
};

use rustc_hash::FxHashMap;

use crate::{
  ArtifactExt, ChunkUkey, RuntimeGlobals, incremental::IncrementalPasses, property_access,
};

#[derive(Debug, Default, Clone)]
pub struct RuntimeProxyMetadata {
  pub tree_runtime_requirements: RuntimeGlobals,
  pub module_proxy_requirements: RuntimeGlobals,
  pub bootstrap_proxy_requirements: RuntimeGlobals,
  pub runtime_module_requirements: RuntimeGlobals,
  pub context_setter_fields: RuntimeGlobals,
  pub force_context_fields: RuntimeGlobals,
  pub hook_exposed_requirements: RuntimeGlobals,
}

impl RuntimeProxyMetadata {
  fn renderable_fields(runtime_globals: RuntimeGlobals) -> RuntimeGlobals {
    runtime_globals
      .renderable_require_scope()
      .difference(RuntimeGlobals::REQUIRE | RuntimeGlobals::REQUIRE_SCOPE)
  }

  pub fn lexical_fields(&self) -> RuntimeGlobals {
    Self::renderable_fields(self.tree_runtime_requirements)
  }

  pub fn context_fields(&self) -> RuntimeGlobals {
    let mut fields = self.module_proxy_requirements;
    fields.insert(self.bootstrap_proxy_requirements);
    fields.insert(self.force_context_fields);
    fields.insert(self.hook_exposed_requirements);
    Self::renderable_fields(fields)
  }

  pub fn context_setter_fields(&self) -> RuntimeGlobals {
    Self::renderable_fields(self.context_setter_fields)
  }

  pub fn render_lexical_declarations(
    &self,
    render_runtime_global: Option<&dyn Fn(RuntimeGlobals) -> Option<String>>,
  ) -> String {
    let names = self
      .lexical_fields()
      .iter_names()
      .filter_map(|(_, runtime_global)| {
        let lexical_name = runtime_global.to_lexical_name()?;
        if let Some(render_runtime_global) = render_runtime_global
          && let Some(value) = render_runtime_global(runtime_global)
        {
          Some(format!("{lexical_name}={value}"))
        } else if runtime_global.should_initialize_as_object() {
          Some(format!("{lexical_name}={{}}"))
        } else if runtime_global.should_initialize_as_array() {
          Some(format!("{lexical_name}=[]"))
        } else {
          Some(lexical_name.to_string())
        }
      })
      .collect::<Vec<_>>();
    if names.is_empty() {
      String::new()
    } else {
      format!("var {};\n", names.join(", "))
    }
  }

  pub fn render_context_setter_assignments(&self, runtime_context: &str) -> String {
    let mut source = String::new();
    for (_, runtime_global) in self.context_fields().iter_names() {
      let (Some(property_name), Some(lexical_name)) = (
        runtime_global.rspack_context_property_name(),
        runtime_global.to_lexical_name(),
      ) else {
        continue;
      };
      writeln!(
        source,
        "{runtime_context}{}={lexical_name};",
        property_access([property_name], 0)
      )
      .expect("write to string should succeed");
    }
    source
  }
}

#[derive(Debug, Default, Clone)]
pub struct RuntimeProxyMetadataArtifact(FxHashMap<ChunkUkey, RuntimeProxyMetadata>);

impl ArtifactExt for RuntimeProxyMetadataArtifact {
  const PASS: IncrementalPasses = IncrementalPasses::CHUNKS_RUNTIME_REQUIREMENTS;
}

impl Deref for RuntimeProxyMetadataArtifact {
  type Target = FxHashMap<ChunkUkey, RuntimeProxyMetadata>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl DerefMut for RuntimeProxyMetadataArtifact {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

impl From<FxHashMap<ChunkUkey, RuntimeProxyMetadata>> for RuntimeProxyMetadataArtifact {
  fn from(value: FxHashMap<ChunkUkey, RuntimeProxyMetadata>) -> Self {
    Self(value)
  }
}

impl From<RuntimeProxyMetadataArtifact> for FxHashMap<ChunkUkey, RuntimeProxyMetadata> {
  fn from(value: RuntimeProxyMetadataArtifact) -> Self {
    value.0
  }
}

impl FromIterator<<FxHashMap<ChunkUkey, RuntimeProxyMetadata> as IntoIterator>::Item>
  for RuntimeProxyMetadataArtifact
{
  fn from_iter<
    T: IntoIterator<Item = <FxHashMap<ChunkUkey, RuntimeProxyMetadata> as IntoIterator>::Item>,
  >(
    iter: T,
  ) -> Self {
    Self(FxHashMap::from_iter(iter))
  }
}

impl IntoIterator for RuntimeProxyMetadataArtifact {
  type Item = <FxHashMap<ChunkUkey, RuntimeProxyMetadata> as IntoIterator>::Item;
  type IntoIter = <FxHashMap<ChunkUkey, RuntimeProxyMetadata> as IntoIterator>::IntoIter;

  fn into_iter(self) -> Self::IntoIter {
    self.0.into_iter()
  }
}
