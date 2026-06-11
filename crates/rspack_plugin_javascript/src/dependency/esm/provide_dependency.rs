use rspack_cacheable::{
  cacheable, cacheable_dyn,
  with::{AsPreset, AsVec},
};
use rspack_core::{
  AsContextDependency, Compilation, Dependency, DependencyCategory, DependencyCodeGeneration,
  DependencyId, DependencyLocation, DependencyRange, DependencyTemplate, DependencyTemplateType,
  DependencyType, ExportsInfoArtifact, ExtendedReferencedExport, FactorizeInfo, InitFragmentKey,
  InitFragmentStage, ModuleDependency, ModuleGraph, ModuleGraphCacheArtifact, NormalInitFragment,
  RuntimeSpec, TemplateContext, TemplateReplaceSource, UsedName, create_exports_object_referenced,
  property_access, to_normal_comment,
};
use rspack_util::ext::DynHash;
use swc_atoms::Atom;

#[cacheable]
#[derive(Debug, Clone)]
pub struct ProvideDependency {
  id: DependencyId,
  #[cacheable(with=AsPreset)]
  request: Atom,
  identifier: String,
  #[cacheable(with=AsVec<AsPreset>)]
  ids: Vec<Atom>,
  range: DependencyRange,
  loc: Option<DependencyLocation>,
  factorize_info: FactorizeInfo,
}

impl ProvideDependency {
  pub fn new(
    range: DependencyRange,
    request: Atom,
    identifier: String,
    ids: Vec<Atom>,
    loc: Option<DependencyLocation>,
  ) -> Self {
    Self {
      range,
      request,
      loc,
      identifier,
      ids,
      id: DependencyId::new(),
      factorize_info: Default::default(),
    }
  }
}

#[cacheable_dyn]
impl Dependency for ProvideDependency {
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
    &DependencyType::Provided
  }

  fn get_referenced_exports(
    &self,
    _module_graph: &ModuleGraph,
    _module_graph_cache: &ModuleGraphCacheArtifact,
    _exports_info_artifact: &ExportsInfoArtifact,
    _runtime: Option<&RuntimeSpec>,
  ) -> Vec<ExtendedReferencedExport> {
    if self.ids.is_empty() {
      create_exports_object_referenced()
    } else {
      vec![ExtendedReferencedExport::Array(self.ids.clone())]
    }
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::True
  }
}

#[cacheable_dyn]
impl ModuleDependency for ProvideDependency {
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
impl DependencyCodeGeneration for ProvideDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(ProvideDependencyTemplate::template_type())
  }

  fn update_hash(
    &self,
    hasher: &mut dyn std::hash::Hasher,
    compilation: &Compilation,
    runtime: Option<&RuntimeSpec>,
  ) {
    self.identifier.dyn_hash(hasher);
    self.ids.dyn_hash(hasher);
    // Case: a ProvidePlugin variable is replaced by an inlined const export,
    // e.g. `provided = (__webpack_require__("./constants"), 2)`. The generated
    // code embeds the target export's inline literal, so the dependency hash must
    // include that payload and not only the provided identifier/import ids.
    let used_name = compilation
      .get_module_graph()
      .connection_by_dependency_id(&self.id)
      .and_then(|connection| {
        let exports_info = compilation
          .exports_info_artifact
          .get_exports_info_data(connection.module_identifier());
        exports_info.get_used_name(&compilation.exports_info_artifact, runtime, &self.ids)
      });
    if let Some(UsedName::Inlined(inlined)) = used_name {
      inlined.dyn_hash(hasher);
    }
  }
}

impl AsContextDependency for ProvideDependency {}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct ProvideDependencyTemplate;

impl ProvideDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Custom("ProvideDependency")
  }
}

impl DependencyTemplate for ProvideDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<ProvideDependency>()
      .expect("ProvideDependencyTemplate should only be used for ProvideDependency");

    let TemplateContext {
      compilation,
      runtime,
      runtime_template,
      init_fragments,
      ..
    } = code_generatable_context;
    let module_graph = compilation.get_module_graph();
    let Some(con) = module_graph.connection_by_dependency_id(&dep.id) else {
      // not find connection, maybe because it's not resolved in make phase, and `bail` is false
      return;
    };

    let exports_info = compilation
      .exports_info_artifact
      .get_exports_info_data(con.module_identifier());
    let used_name =
      exports_info.get_used_name(&compilation.exports_info_artifact, *runtime, &dep.ids);
    let module_raw = runtime_template.module_raw(compilation, dep.id(), dep.request(), dep.weak());
    let provided_expr = match used_name {
      Some(UsedName::Normal(used_name)) => format!("{module_raw}{}", property_access(used_name, 0)),
      Some(UsedName::Inlined(inlined)) => format!(
        "({}, {})",
        module_raw,
        inlined.render(&to_normal_comment(&format!(
          "inlined export {}",
          property_access(&dep.ids, 0)
        )))
      ),
      None => module_raw,
    };

    init_fragments.push(Box::new(
      NormalInitFragment::new(
        format!(
          "/* provided dependency */ var {} = {};\n",
          dep.identifier, provided_expr
        ),
        InitFragmentStage::StageProvides,
        1,
        InitFragmentKey::ModuleExternal(format!("provided {}", dep.identifier)),
        None,
      )
      .with_top_level_decl_symbols(vec![dep.identifier.clone().into()]),
    ));
    source.replace(dep.range.start, dep.range.end, dep.identifier.clone(), None);
  }
}
