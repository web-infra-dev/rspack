use rspack_cacheable::{cacheable, cacheable_dyn, with::Skip};
use rspack_collections::IdentifierMap;
use rspack_core::{
  ChunkGraph, CodeGenerationData, CodeGenerationDataItem, Compilation, Dependency, DependencyId,
  Module, ModuleIdentifier, RuntimeSpec, get_runtime_key, rspack_sources::Source,
};
use rspack_error::Result;
use rspack_hash::RspackHashDigest;
use rustc_hash::FxHashMap;
use smol_str::SmolStr;

use crate::{
  dependency::{CssIcssExportDependency, CssLocalIdentKind},
  utils::{LocalIdentOptions, css_generator_options},
};

/// Local identifiers generated for every CSS module visited by one codegen.
/// A module entry, including an empty one, means its identifiers are ready.
#[cacheable]
#[derive(Debug, Default)]
pub(super) struct CssLocalIdents {
  #[cacheable(with=Skip)]
  modules: IdentifierMap<FxHashMap<DependencyId, SmolStr>>,
}

#[cacheable_dyn]
impl CodeGenerationDataItem for CssLocalIdents {}

impl CssLocalIdents {
  pub(super) async fn ensure_module(
    data: &mut CodeGenerationData,
    source: &dyn Source,
    module: &dyn Module,
    compilation: &Compilation,
    runtime: Option<&RuntimeSpec>,
  ) -> Result<()> {
    let module_identifier = module.identifier();
    if data
      .get::<Self>()
      .is_some_and(|idents| idents.modules.contains_key(&module_identifier))
    {
      return Ok(());
    }

    let mut idents = FxHashMap::default();
    if let Some(first_local) = module.get_dependencies().iter().position(|dependency| {
      dependency
        .downcast_ref::<CssIcssExportDependency>()
        .is_some_and(|definition| definition.local_ident.is_some())
    }) {
      let normal_module = module
        .as_normal_module()
        .expect("CSS module should be a normal module");
      let options = LocalIdentOptions::new(
        normal_module.resource_resolved_data(),
        source.source().into_string_lossy(),
        &compilation.options,
        css_generator_options(normal_module.get_generator_options()),
        Self::cached_module_hash(module_identifier, compilation, runtime),
      );
      for dependency in &module.get_dependencies()[first_local..] {
        let Some(definition) = dependency.downcast_ref::<CssIcssExportDependency>() else {
          continue;
        };
        let Some(kind) = definition.local_ident else {
          continue;
        };
        let ident = options.get_local_ident(&definition.value).await?;
        let ident = match kind {
          CssLocalIdentKind::Ident => ident,
          CssLocalIdentKind::DashedIdent => {
            format!("--{}", ident.strip_prefix("_--").unwrap_or(&ident))
          }
        };
        idents.insert(*definition.id(), SmolStr::new(ident));
      }
    }

    if !data.contains::<Self>() {
      data.insert(Self::default());
    }
    data
      .get_mut::<Self>()
      .expect("CSS local ident data should have been initialized")
      .modules
      .insert(module_identifier, idents);
    Ok(())
  }

  pub(super) fn get<'a>(
    data: &'a CodeGenerationData,
    module_identifier: ModuleIdentifier,
    dependency: DependencyId,
  ) -> &'a SmolStr {
    data
      .get::<Self>()
      .and_then(|idents| idents.modules.get(&module_identifier))
      .and_then(|idents| idents.get(&dependency))
      .expect("local identifier should be generated before rendering")
  }

  fn cached_module_hash<'a>(
    module_identifier: ModuleIdentifier,
    compilation: &'a Compilation,
    runtime: Option<&RuntimeSpec>,
  ) -> &'a RspackHashDigest {
    let hashes = compilation
      .cgm_hash_artifact
      .get_runtime_map(&module_identifier)
      .expect("CSS module hashes should be stored before code generation");
    runtime
      .and_then(|runtime| ChunkGraph::get_module_hash(compilation, module_identifier, runtime))
      // A child without a chunk has its runtime-independent hash stored here.
      .or_else(|| hashes.get(&RuntimeSpec::default()))
      // A child can belong to a shared chunk whose runtime includes the parent.
      .or_else(|| {
        let runtime = runtime?;
        let child_runtime = compilation
          .build_chunk_graph_artifact
          .chunk_graph
          .get_module_runtimes_iter(
            module_identifier,
            &compilation.build_chunk_graph_artifact.chunk_by_ukey,
          )
          .filter(|child_runtime| runtime.is_subset(child_runtime))
          .min_by(|a, b| {
            a.len()
              .cmp(&b.len())
              .then_with(|| get_runtime_key(a).cmp(get_runtime_key(b)))
          })?;
        ChunkGraph::get_module_hash(compilation, module_identifier, child_runtime)
      })
      // Without a matching runtime, a common digest is still unambiguous.
      .or_else(|| {
        let mut values = hashes.values();
        let hash = values.next()?;
        values.all(|value| value == hash).then_some(hash)
      })
      .expect("CSS module should have a hash for its code generation runtime")
  }
}
