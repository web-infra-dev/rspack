use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AsContextDependency, ChunkGraph, Compilation, Dependency, DependencyCodeGeneration, DependencyId,
  DependencyTemplate, DependencyTemplateType, DependencyType, ExportsInfoArtifact, FactorizeInfo,
  InitFragmentKey, InitFragmentStage, ModuleDependency, ModuleGraphCacheArtifact,
  NormalInitFragment, RuntimeGlobals, RuntimeSpec, TemplateContext, TemplateReplaceSource,
  create_exports_object_referenced, create_no_exports_referenced,
};
use rspack_util::ext::DynHash;

#[cacheable]
#[derive(Debug, Clone)]
pub struct ModuleDecoratorDependency {
  decorator: RuntimeGlobals,
  allow_exports_access: bool,
  id: DependencyId,
  factorize_info: FactorizeInfo,
}

impl ModuleDecoratorDependency {
  pub fn new(decorator: RuntimeGlobals, allow_exports_access: bool) -> Self {
    Self {
      decorator,
      allow_exports_access,
      id: DependencyId::new(),
      factorize_info: Default::default(),
    }
  }
}

#[cacheable_dyn]
impl ModuleDependency for ModuleDecoratorDependency {
  fn request(&self) -> &str {
    "self"
  }

  fn factorize_info(&self) -> &FactorizeInfo {
    &self.factorize_info
  }

  fn factorize_info_mut(&mut self) -> &mut FactorizeInfo {
    &mut self.factorize_info
  }
}

#[cacheable_dyn]
impl DependencyCodeGeneration for ModuleDecoratorDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(ModuleDecoratorDependencyTemplate::template_type())
  }

  fn update_hash(
    &self,
    hasher: &mut dyn std::hash::Hasher,
    _compilation: &Compilation,
    _runtime: Option<&RuntimeSpec>,
  ) {
    self.decorator.dyn_hash(hasher);
    self.allow_exports_access.dyn_hash(hasher);
  }
}

impl AsContextDependency for ModuleDecoratorDependency {}

#[cacheable_dyn]
impl Dependency for ModuleDecoratorDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn resource_identifier(&self) -> Option<&str> {
    Some("self")
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::ModuleDecorator
  }

  fn get_referenced_exports(
    &self,
    _module_graph: &rspack_core::ModuleGraph,
    _module_graph_cache: &ModuleGraphCacheArtifact,
    _exports_info_artifact: &ExportsInfoArtifact,
    _runtime: Option<&rspack_core::RuntimeSpec>,
  ) -> Vec<rspack_core::ExtendedReferencedExport> {
    if self.allow_exports_access {
      create_exports_object_referenced()
    } else {
      create_no_exports_referenced()
    }
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::False
  }
}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct ModuleDecoratorDependencyTemplate;

impl ModuleDecoratorDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Dependency(DependencyType::ModuleDecorator)
  }
}

impl DependencyTemplate for ModuleDecoratorDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    _source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<ModuleDecoratorDependency>()
      .expect(
        "ModuleDecoratorDependencyTemplate should only be used for ModuleDecoratorDependency",
      );

    let TemplateContext {
      init_fragments,
      compilation,
      module,
      runtime_template,
      ..
    } = code_generatable_context;

    let module_graph = compilation.get_module_graph();
    let module = module_graph
      .module_by_identifier(&module.identifier())
      .expect("should have mgm");

    // ref: tests/webpack-test/cases/scope-hoisting/issue-5096 will return a `null` as module id
    let module_id =
      ChunkGraph::get_module_id(&compilation.module_ids_artifact, module.identifier())
        .map(|s| s.to_string())
        .unwrap_or_default();

    init_fragments.push(Box::new(NormalInitFragment::new(
      format!(
        "/* module decorator */ {module} = {decorator}({module});\n",
        module = runtime_template.render_module_argument(module.get_module_argument()),
        decorator = runtime_template.render_runtime_globals(&dep.decorator),
      ),
      InitFragmentStage::StageProvides,
      0,
      InitFragmentKey::ModuleDecorator(module_id),
      None,
    )));
  }
}
