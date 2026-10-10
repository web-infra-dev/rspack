use rspack_cacheable::{cacheable, cacheable_dyn, with::AsPreset};
use rspack_core::{
  AsContextDependency, Compilation, CssExportType, Dependency, DependencyCategory,
  DependencyCodeGeneration, DependencyId, DependencyRange, DependencyType, ExportsInfoArtifact,
  ModuleDependency, ReferencedExport, RuntimeSpec,
};
use rspack_hash::{RspackHasher, rspack_hash_object};
use rspack_intern::Atom;
use smol_str::SmolStr;

/// A named import, shared by ICSS, `@value`, and cross-file `composes`.
/// For `@value color as accent from "./colors.css"`, this owns the request,
/// imported name `color`, and local name `accent`. Export definitions and symbol
/// uses refer to its ID; it establishes the module edge without rewriting CSS.
#[cacheable]
#[derive(Debug)]
pub struct CssIcssImportDependency {
  id: DependencyId,
  request: String,
  #[cacheable(with=AsPreset)]
  import_name: Atom,
  #[cacheable(with=AsPreset)]
  local_name: SmolStr,
  range: DependencyRange,
  source_order: Option<i32>,
  export_type: Option<CssExportType>,
}

impl CssIcssImportDependency {
  pub fn new(
    request: String,
    import_name: Atom,
    local_name: SmolStr,
    range: DependencyRange,
    export_type: Option<CssExportType>,
  ) -> Self {
    Self {
      id: DependencyId::new(),
      request,
      import_name,
      local_name,
      range,
      source_order: None,
      export_type,
    }
  }

  pub(crate) fn import_name(&self) -> &str {
    self.import_name.as_str()
  }

  pub fn set_source_order(&mut self, source_order: i32) {
    self.source_order = Some(source_order);
  }

  pub fn export_type(&self) -> Option<CssExportType> {
    self.export_type
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
    runtime: Option<&RuntimeSpec>,
  ) {
    rspack_hash_object!(hasher, {
      "request" => &self.request,
      "importName" => self.import_name.as_str(),
      "localName" => &self.local_name,
      "range" => (self.range.start, self.range.end),
      "exportType" => self.export_type,
    });
    super::hash::hash_css_import_target(self.id, hasher, compilation, runtime);
  }
}
impl AsContextDependency for CssIcssImportDependency {}
