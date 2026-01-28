use itertools::Itertools;
use rspack_cacheable::{
  cacheable, cacheable_dyn,
  with::{AsPreset, AsVec},
};
use rspack_core::{
  AsContextDependency, Compilation, Dependency, DependencyCategory, DependencyCodeGeneration,
  DependencyId, DependencyLocation, DependencyRange, DependencyTemplate, DependencyTemplateType,
  DependencyType, ExportsInfoGetter, ExtendedReferencedExport, FactorizeInfo, GetUsedNameParam,
  InitFragmentKey, InitFragmentStage, ModuleDependency, ModuleGraph, ModuleGraphCacheArtifact,
  NormalInitFragment, PrefetchExportsInfoMode, RuntimeSpec, SharedSourceMap, TemplateContext,
  TemplateReplaceSource, UsedName, create_exports_object_referenced,
};
use rspack_util::ext::DynHash;
use swc_core::atoms::Atom;

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
    source_map: Option<SharedSourceMap>,
  ) -> Self {
    let loc = range.to_loc(source_map.as_deref());
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
    _compilation: &Compilation,
    _runtime: Option<&RuntimeSpec>,
  ) {
    self.identifier.dyn_hash(hasher);
    self.ids.dyn_hash(hasher);
  }
}

fn path_to_string(path: Option<&UsedName>) -> String {
  match path {
    Some(p) => match p {
      UsedName::Normal(vec) if !vec.is_empty() => vec
        .iter()
        .map(|part| format!("[\"{}\"]", part.as_str()))
        .join(""),
      _ => String::new(),
    },
    None => String::new(),
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
      runtime_requirements,
      init_fragments,
      ..
    } = code_generatable_context;
    let module_graph = compilation.get_module_graph();
    let Some(con) = module_graph.connection_by_dependency_id(&dep.id) else {
      // not find connection, maybe because it's not resolved in make phase, and `bail` is false
      return;
    };

    let used_name = if dep.ids.is_empty() {
      let exports_info_used =
        module_graph.get_prefetched_exports_info_used(con.module_identifier(), *runtime);
      ExportsInfoGetter::get_used_name(
        GetUsedNameParam::WithoutNames(&exports_info_used),
        *runtime,
        &dep.ids,
      )
    } else {
      let exports_info = module_graph.get_prefetched_exports_info(
        con.module_identifier(),
        PrefetchExportsInfoMode::Nested(&dep.ids),
      );
      ExportsInfoGetter::get_used_name(
        GetUsedNameParam::WithNames(&exports_info),
        *runtime,
        &dep.ids,
      )
    };

    init_fragments.push(Box::new(NormalInitFragment::new(
      format!(
        "/* provided dependency */ var {} = {}{};\n",
        dep.identifier,
        compilation.runtime_template.module_raw(
          compilation,
          runtime_requirements,
          dep.id(),
          dep.request(),
          dep.weak()
        ),
        path_to_string(used_name.as_ref())
      ),
      InitFragmentStage::StageProvides,
      1,
      InitFragmentKey::ModuleExternal(format!("provided {}", dep.identifier)),
      None,
    )));
    source.replace(dep.range.start, dep.range.end, &dep.identifier, None);
  }
}
