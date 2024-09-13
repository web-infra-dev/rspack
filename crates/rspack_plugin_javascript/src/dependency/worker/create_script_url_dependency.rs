use rspack_core::{
  AsContextDependency, AsModuleDependency, Compilation, Dependency, DependencyCategory,
  DependencyId, DependencyRange, DependencyTemplate, DependencyType, RealDependencyLocation,
  RuntimeGlobals, RuntimeSpec, TemplateContext, TemplateReplaceSource,
};

#[derive(Debug, Clone)]
pub struct CreateScriptUrlDependency {
  id: DependencyId,
  range: RealDependencyLocation,
  range_path: DependencyRange,
}

impl CreateScriptUrlDependency {
  pub fn new(range: RealDependencyLocation, range_path: DependencyRange) -> Self {
    Self {
      id: DependencyId::new(),
      range,
      range_path,
    }
  }
}

impl Dependency for CreateScriptUrlDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Worker
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::CreateScriptUrl
  }

  fn range(&self) -> Option<&RealDependencyLocation> {
    Some(&self.range)
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::False
  }
}

impl DependencyTemplate for CreateScriptUrlDependency {
  fn apply(
    &self,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    code_generatable_context
      .runtime_requirements
      .insert(RuntimeGlobals::CREATE_SCRIPT_URL);

    source.insert(
      self.range_path.start,
      format!("{}(", RuntimeGlobals::CREATE_SCRIPT_URL).as_str(),
      None,
    );
    source.insert(self.range_path.end, ")", None);
  }

  fn dependency_id(&self) -> Option<DependencyId> {
    Some(self.id)
  }

  fn update_hash(
    &self,
    _hasher: &mut dyn std::hash::Hasher,
    _compilation: &Compilation,
    _runtime: Option<&RuntimeSpec>,
  ) {
  }
}

impl AsModuleDependency for CreateScriptUrlDependency {}
impl AsContextDependency for CreateScriptUrlDependency {}
