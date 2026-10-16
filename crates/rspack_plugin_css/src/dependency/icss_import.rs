use rspack_cacheable::{cacheable, cacheable_dyn, with::AsPreset};
use rspack_core::{
  AsContextDependency, ChunkGraph, Compilation, Dependency, DependencyCategory,
  DependencyCodeGeneration, DependencyId, DependencyRange, DependencyType, ExportsInfoArtifact,
  ModuleDependency, ReferencedExport, RuntimeSpec,
};
use rspack_hash::{RspackHash, RspackHasher};
use rspack_intern::Atom;
use smol_str::SmolStr;

/// A named import, shared by ICSS, `@value`, and cross-file `composes`.
/// For `@value color as accent from "./colors.css"`, this owns the request,
/// imported name `color`, and local name `accent`. Export definitions and symbol
/// uses refer to its ID; it establishes the module edge without rewriting CSS.
#[cacheable]
#[derive(Debug, RspackHash)]
pub struct CssIcssImportDependency {
  #[rspack_hash(skip)]
  id: DependencyId,
  request: String,
  #[cacheable(with=AsPreset)]
  import_name: Atom,
  #[cacheable(with=AsPreset)]
  local_name: SmolStr,
  range: DependencyRange,
  // Dependencies are already hashed in source order.
  #[rspack_hash(skip)]
  source_order: Option<i32>,
}

impl CssIcssImportDependency {
  pub fn new(
    request: String,
    import_name: Atom,
    local_name: SmolStr,
    range: DependencyRange,
  ) -> Self {
    Self {
      id: DependencyId::new(),
      request,
      import_name,
      local_name,
      range,
      source_order: None,
    }
  }

  pub(crate) fn import_name(&self) -> &str {
    self.import_name.as_str()
  }

  pub fn set_source_order(&mut self, source_order: i32) {
    self.source_order = Some(source_order);
  }
}

#[cacheable_dyn]
impl Dependency for CssIcssImportDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::CssCompose
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::CssIcssImport
  }

  fn range(&self) -> Option<DependencyRange> {
    Some(self.range)
  }

  fn source_order(&self) -> Option<i32> {
    self.source_order
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    // ICSS values can be inlined through multiple reexports into a consumer's CSS.
    rspack_core::AffectType::Transitive
  }

  fn get_referenced_exports(
    &self,
    module_graph: &rspack_core::ModuleGraph,
    _module_graph_cache: &rspack_core::ModuleGraphCacheArtifact,
    _exports_info_artifact: &ExportsInfoArtifact,
    _runtime: Option<&RuntimeSpec>,
  ) -> Vec<ReferencedExport> {
    let name = module_graph
      .get_module_by_dependency_id(&self.id)
      .map(|module| crate::css_exports::css_export_name(module.as_ref(), &self.import_name));
    vec![ReferencedExport::from(Atom::from(
      name.as_deref().unwrap_or(&self.import_name),
    ))]
  }
}

#[cacheable_dyn]
impl ModuleDependency for CssIcssImportDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.request
  }
}

#[cacheable_dyn]
impl DependencyCodeGeneration for CssIcssImportDependency {
  fn update_hash(
    &self,
    hasher: &mut RspackHasher,
    compilation: &Compilation,
    _runtime: Option<&RuntimeSpec>,
  ) {
    let graph = compilation.get_module_graph();
    let module_id = graph
      .get_module_by_dependency_id(&self.id)
      .and_then(|module| {
        ChunkGraph::get_module_id(&compilation.module_ids_artifact, module.identifier())
      });
    // The generated JS expression can refer to the resolved module ID.
    RspackHash::hash(&(self, module_id), hasher);
  }
}
impl AsContextDependency for CssIcssImportDependency {}
