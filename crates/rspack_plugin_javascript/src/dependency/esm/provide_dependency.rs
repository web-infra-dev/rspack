use itertools::Itertools;
use rspack_cacheable::{
  cacheable, cacheable_dyn,
  with::{AsPreset, AsVec, Skip},
};
use rspack_core::{
  create_exports_object_referenced, module_raw, Compilation, DependencyLocation, DependencyRange,
  DependencyType, ExtendedReferencedExport, FactorizeInfo, ModuleGraph, NormalInitFragment,
  RuntimeSpec, SharedSourceMap, UsedName,
};
use rspack_core::{AsContextDependency, Dependency, InitFragmentKey, InitFragmentStage};
use rspack_core::{DependencyCategory, DependencyId, DependencyTemplate};
use rspack_core::{ModuleDependency, TemplateContext, TemplateReplaceSource};
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
impl DependencyTemplate for ProvideDependency {
  fn apply(
    &self,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let TemplateContext {
      compilation,
      runtime,
      runtime_requirements,
      init_fragments,
      ..
    } = code_generatable_context;
    let module_graph = compilation.get_module_graph();
    let Some(con) = module_graph.connection_by_dependency_id(&self.id) else {
      // not find connection, maybe because it's not resolved in make phase, and `bail` is false
      return;
    };
    let exports_info = module_graph.get_exports_info(con.module_identifier());
    let used_name =
      exports_info.get_used_name(&module_graph, *runtime, UsedName::Vec(self.ids.clone()));
    init_fragments.push(Box::new(NormalInitFragment::new(
      format!(
        "/* provided dependency */ var {} = {}{};\n",
        self.identifier,
        module_raw(
          compilation,
          runtime_requirements,
          self.id(),
          self.request(),
          self.weak()
        ),
        path_to_string(used_name.as_ref())
      ),
      InitFragmentStage::StageProvides,
      1,
      InitFragmentKey::ModuleExternal(format!("provided {}", self.identifier)),
      None,
    )));
    source.replace(self.range.start, self.range.end, &self.identifier, None);
  }

  fn dependency_id(&self) -> Option<DependencyId> {
    Some(self.id)
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
