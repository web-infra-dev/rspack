use rspack_cacheable::{cacheable, cacheable_dyn, with::AsPreset};
use rspack_core::{
  AsContextDependency, Dependency, DependencyCategory, DependencyCodeGeneration, DependencyId,
  DependencyLocation, DependencyRange, DependencyTemplate, DependencyTemplateType, DependencyType,
  ExportsInfoArtifact, ExtendedReferencedExport, FactorizeInfo, ModuleDependency, ModuleGraph,
  ModuleGraphCacheArtifact, NormalInitFragment, RuntimeGlobals, RuntimeSpec, TemplateContext,
  TemplateReplaceSource, create_exports_object_referenced,
};
use rspack_util::{ext::DynHash, json_stringify_str};
use swc_core::atoms::Atom;

const IMPORT_META_RSC_BINDING: &str = "__rspack_import_meta_rsc__";

#[cacheable]
#[derive(Debug, Clone)]
pub struct ImportMetaRscDependency {
  id: DependencyId,
  #[cacheable(with=AsPreset)]
  request: Atom,
  importer: String,
  range: DependencyRange,
  loc: Option<DependencyLocation>,
  factorize_info: FactorizeInfo,
}

impl ImportMetaRscDependency {
  pub fn new(importer: String, range: DependencyRange, loc: Option<DependencyLocation>) -> Self {
    Self {
      id: DependencyId::new(),
      request: Atom::from("react"),
      importer,
      range,
      loc,
      factorize_info: Default::default(),
    }
  }
}

#[cacheable_dyn]
impl Dependency for ImportMetaRscDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn loc(&self) -> Option<DependencyLocation> {
    self.loc.clone()
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Esm
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::ImportMetaRsc
  }

  fn range(&self) -> Option<DependencyRange> {
    Some(self.range)
  }

  fn get_referenced_exports(
    &self,
    _module_graph: &ModuleGraph,
    _module_graph_cache: &ModuleGraphCacheArtifact,
    _exports_info_artifact: &ExportsInfoArtifact,
    _runtime: Option<&RuntimeSpec>,
  ) -> Vec<ExtendedReferencedExport> {
    create_exports_object_referenced()
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::True
  }
}

#[cacheable_dyn]
impl ModuleDependency for ImportMetaRscDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.request
  }

  fn factorize_info(&self) -> &FactorizeInfo {
    &self.factorize_info
  }

  fn factorize_info_mut(&mut self) -> &mut FactorizeInfo {
    &mut self.factorize_info
  }
}

#[cacheable_dyn]
impl DependencyCodeGeneration for ImportMetaRscDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(ImportMetaRscDependencyTemplate::template_type())
  }

  fn update_hash(
    &self,
    hasher: &mut dyn std::hash::Hasher,
    _compilation: &rspack_core::Compilation,
    _runtime: Option<&RuntimeSpec>,
  ) {
    self.importer.dyn_hash(hasher);
  }
}

impl AsContextDependency for ImportMetaRscDependency {}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct ImportMetaRscDependencyTemplate;

impl ImportMetaRscDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Custom("ImportMetaRscDependency")
  }
}

// `import.meta.rspackRsc` is generated once per module as:
//
// var __rspack_import_meta_rsc__ = {
//   loadCss: function() {
//     return ((__webpack_require__.rscM.entryCssFiles["/path/to/component.js"] || []).map(...));
//   }
// };
//
// Each use of `import.meta.rspackRsc` is then replaced with
// `__rspack_import_meta_rsc__`, so multiple `loadCss()` calls in the same module
// share the same helper object.
impl DependencyTemplate for ImportMetaRscDependencyTemplate {
  fn render(
    &self,
    dependency: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let dependency = dependency
      .as_any()
      .downcast_ref::<ImportMetaRscDependency>()
      .expect("ImportMetaRscDependencyTemplate should only be used for ImportMetaRscDependency");

    let TemplateContext {
      compilation,
      init_fragments,
      runtime_template,
      ..
    } = code_generatable_context;
    let rsc_manifest = runtime_template.render_runtime_globals(&RuntimeGlobals::RSC_MANIFEST);
    let react =
      runtime_template.module_raw(compilation, dependency.id(), dependency.request(), false);
    let importer = json_stringify_str(&dependency.importer);

    init_fragments.push(Box::new(
      NormalInitFragment::new(
        format!(
          r#"var {IMPORT_META_RSC_BINDING} = {{
  loadCss: function() {{
    return (({rsc_manifest}.entryCssFiles[{importer}] || []).map(function(href) {{
      return {react}.createElement("link", {{
        key: href,
        rel: "stylesheet",
        href: href,
        precedence: "default"
      }});
    }}));
  }}
}};
"#
        ),
        rspack_core::InitFragmentStage::StageProvides,
        0,
        rspack_core::InitFragmentKey::ModuleExternal(format!(
          "import.meta.rspackRsc {}",
          dependency.importer
        )),
        None,
      )
      .with_top_level_decl_symbols(vec![Atom::from(IMPORT_META_RSC_BINDING)]),
    ));

    source.replace(
      dependency.range.start,
      dependency.range.end,
      IMPORT_META_RSC_BINDING.to_string(),
      None,
    );
  }
}
