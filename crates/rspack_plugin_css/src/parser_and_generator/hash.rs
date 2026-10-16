use rspack_collections::IdentifierSet;
use rspack_core::{Compilation, Dependency, DependencyId, Module, RuntimeSpec};
use rspack_error::Result;
use rspack_hash::{RspackHasher, rspack_hash_object};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
  dependency::{
    CssIcssExportDependency, CssIcssImportDependency, CssIcssSymbolDependency, CssImportDependency,
  },
  parser_and_generator::{
    CssParserAndGenerator,
    walker::{
      CssExportFuture, CssExportRequirements, CssModuleGraphSink, CssModuleGraphWalker,
      CssModuleImportSink,
    },
  },
};

/// Hashing and rendering run in separate compilation phases, but use the same
/// export and source-import walks. Hashing never computes generated local names.
pub(super) async fn hash_css_dependencies(
  module: &dyn Module,
  hasher: &mut RspackHasher,
  compilation: &Compilation,
  runtime: Option<&RuntimeSpec>,
) -> Result<()> {
  let mut walker = CssModuleGraphWalker::new(compilation);
  let mut sink = CssHashSink {
    hasher,
    runtime,
    visited: IdentifierSet::default(),
    visited_sources: IdentifierSet::default(),
    visited_exports: FxHashSet::default(),
    bindings: FxHashMap::default(),
  };
  sink.visited_sources.insert(module.identifier());
  sink.visit_module(&mut walker, module, false).await?;
  walker.walk_css_imports(&mut sink, module, false).await
}

struct CssHashSink<'h, 'g> {
  hasher: &'h mut RspackHasher,
  runtime: Option<&'g RuntimeSpec>,
  visited: IdentifierSet,
  visited_sources: IdentifierSet,
  visited_exports: FxHashSet<DependencyId>,
  bindings: FxHashMap<DependencyId, usize>,
}

impl<'g> CssHashSink<'_, 'g> {
  async fn visit_module(
    &mut self,
    walker: &mut CssModuleGraphWalker<'g>,
    module: &dyn Module,
    hash_fields: bool,
  ) -> Result<bool> {
    if !self.visited.insert(module.identifier()) {
      return Ok(false);
    }
    if hash_fields {
      // Runtime hashes are computed in parallel, so target runtime hashes are
      // unavailable here. Use current build inputs and the module graph cache.
      rspack_hash_object!(self.hasher, {
        "buildHash" => &module.build_info().hash,
        "graphHash" => walker.compilation().build_chunk_graph_artifact.chunk_graph
          .get_module_graph_hash(module, walker.compilation(), self.runtime),
      });
      if let Some(normal_module) = module.as_normal_module()
        && let Some(generator) = normal_module
          .parser_and_generator()
          .downcast_ref::<CssParserAndGenerator>()
      {
        generator.update_hash(normal_module, self.hasher);
      }
    }
    // The core module hash handles the root's own dependency fields.
    self.hash_dependencies(walker.compilation(), module, hash_fields);
    for dependency in module.get_dependencies() {
      if dependency
        .downcast_ref::<CssIcssExportDependency>()
        .is_some()
        || dependency
          .downcast_ref::<CssIcssImportDependency>()
          .is_some()
      {
        walker
          .resolve_css_export_dependency(
            self,
            *dependency.id(),
            CssExportRequirements::default(),
            true,
          )
          .await?;
      }
    }
    Ok(true)
  }

  fn hash_dependencies(
    &mut self,
    compilation: &Compilation,
    module: &dyn Module,
    hash_fields: bool,
  ) {
    let dependencies = module.get_dependencies();
    // Bindings always refer to definitions/imports in the same module. Their
    // source-order indices are stable, unlike allocation-dependent DependencyIds.
    // Recording edges instead of recursively hashing definitions handles local
    // self-compositions and cycles without repeating a shared definition's fields.
    self.bindings.extend(
      dependencies
        .iter()
        .enumerate()
        .filter(|(_, dependency)| {
          dependency
            .downcast_ref::<CssIcssExportDependency>()
            .is_some()
            || dependency
              .downcast_ref::<CssIcssImportDependency>()
              .is_some()
        })
        .map(|(index, dependency)| (*dependency.id(), index)),
    );

    if hash_fields && let Some(dependencies) = module.get_presentational_dependencies() {
      for dependency in dependencies {
        dependency.update_hash(self.hasher, compilation, self.runtime);
      }
    }
    for (index, dependency) in dependencies.iter().enumerate() {
      if hash_fields && let Some(dependency) = dependency.as_dependency_code_generation() {
        dependency.update_hash(self.hasher, compilation, self.runtime);
      }
      if let Some(symbol) = dependency.downcast_ref::<CssIcssSymbolDependency>() {
        rspack_hash_object!(self.hasher, {
          "dependency" => index,
          "symbol" => self.bindings[&symbol.target],
        });
      }
    }
  }
}

impl<'g> CssModuleGraphSink<'g> for CssHashSink<'_, 'g> {
  type ExportValue = ();

  fn should_visit_export(&mut self, id: DependencyId) -> bool {
    self.visited_exports.insert(id)
  }

  fn reference(&mut self, from: DependencyId, to: DependencyId) {
    rspack_hash_object!(self.hasher, {
      "dependency" => self.bindings[&from],
      "reference" => self.bindings[&to],
    });
  }

  fn compose(&mut self, from: DependencyId, to: DependencyId) {
    rspack_hash_object!(self.hasher, {
      "dependency" => self.bindings[&from],
      "composes" => self.bindings[&to],
    });
  }

  fn resolve_icss_import<'a>(
    &'a mut self,
    walker: &'a mut CssModuleGraphWalker<'g>,
    import: &'g CssIcssImportDependency,
    _requirements: CssExportRequirements,
    _follow_self_imports: bool,
  ) -> CssExportFuture<'a, Self::ExportValue>
  where
    'g: 'a,
  {
    Box::pin(async move {
      let Some(module) = walker
        .compilation()
        .get_module_graph()
        .get_module_by_dependency_id(import.id())
        .filter(|module| module.build_info().css.is_some())
      else {
        return Ok(());
      };
      if self.visit_module(walker, module.as_ref(), true).await? {
        walker
          .walk_css_imports(self, module.as_ref(), false)
          .await?;
      }
      Ok(())
    })
  }
}

impl<'g> CssModuleImportSink<'g> for CssHashSink<'_, 'g> {
  type State = ();

  async fn enter_import(
    &mut self,
    walker: &mut CssModuleGraphWalker<'g>,
    dependency: &'g CssImportDependency,
    module: &'g dyn Module,
    include_import_sources: bool,
  ) -> Result<Option<Self::State>> {
    if module.build_info().css.is_none()
      || (!include_import_sources && !walker.is_css_import_inlined(dependency, module))
      || !self.visited_sources.insert(module.identifier())
    {
      return Ok(None);
    }
    self.visit_module(walker, module, true).await?;
    Ok(Some(()))
  }

  async fn leave_import(
    &mut self,
    _dependency: &'g CssImportDependency,
    _module: &'g dyn Module,
    _state: (),
  ) -> Result<()> {
    Ok(())
  }
}
