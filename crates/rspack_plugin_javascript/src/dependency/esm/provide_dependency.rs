use itertools::Itertools;
use rspack_cacheable::{
  cacheable, cacheable_dyn,
  with::{AsPreset, AsVec, Skip},
};
use rspack_core::{
  create_exports_object_referenced, module_raw, AsContextDependency, Compilation, Dependency,
  DependencyCategory, DependencyCodeGeneration, DependencyId, DependencyLocation, DependencyRange,
  DependencyTemplate, DependencyTemplateType, DependencyType, ExtendedReferencedExport,
  FactorizeInfo, InitFragmentKey, InitFragmentStage, ModuleDependency, ModuleGraph,
  NormalInitFragment, RuntimeSpec, SharedSourceMap, TemplateContext, TemplateReplaceSource,
  UsedName,
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
  #[cacheable(with=Skip)]
  source_map: Option<SharedSourceMap>,
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
    Self {
      range,
      request,
      source_map,
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
    self.range.to_loc(self.source_map.as_ref())
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

  fn set_request(&mut self, request: String) {
    self.request = request.into();
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
      UsedName::Str(str) => format!("[\"{}\"]", str.as_str()),
      UsedName::Vec(vec) if !vec.is_empty() => vec
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
    let exports_info = module_graph.get_exports_info(con.module_identifier());
    let used_name =
      exports_info.get_used_name(&module_graph, *runtime, UsedName::Vec(dep.ids.clone()));
    init_fragments.push(Box::new(NormalInitFragment::new(
      format!(
        "/* provided dependency */ var {} = {}{};\n",
        dep.identifier,
        module_raw(
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
