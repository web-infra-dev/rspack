use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  Compilation, DependencyCodeGeneration, DependencyLocation, DependencyRange, DependencyTemplate,
  DependencyTemplateType, InitFragmentExt, InitFragmentKey, InitFragmentStage, ModuleArgument,
  NormalInitFragment, RuntimeGlobals, RuntimeSpec, TemplateContext, TemplateReplaceSource,
};
use rspack_util::ext::DynHash;

#[cacheable]
#[derive(Debug, Clone)]
pub struct ModuleArgumentDependency {
  id: Option<String>,
  range: DependencyRange,
  loc: Option<DependencyLocation>,
}

impl ModuleArgumentDependency {
  pub fn new(id: Option<String>, range: DependencyRange, source: Option<&str>) -> Self {
    let loc = range.to_loc(source);
    Self { id, range, loc }
  }

  pub fn loc(&self) -> Option<DependencyLocation> {
    self.loc.clone()
  }
}

#[cacheable_dyn]
impl DependencyCodeGeneration for ModuleArgumentDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(ModuleArgumentDependencyTemplate::template_type())
  }

  fn update_hash(
    &self,
    hasher: &mut dyn std::hash::Hasher,
    _compilation: &Compilation,
    _runtime: Option<&RuntimeSpec>,
  ) {
    self.id.dyn_hash(hasher);
    self.range.dyn_hash(hasher);
  }
}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct ModuleArgumentDependencyTemplate;

impl ModuleArgumentDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Custom("ModuleArgumentDependency")
  }
}

impl DependencyTemplate for ModuleArgumentDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<ModuleArgumentDependency>()
      .expect("ModuleArgumentDependencyTemplate should be used for ModuleArgumentDependency");

    let TemplateContext {
      compilation,
      module,
      runtime_template,
      init_fragments,
      ..
    } = code_generatable_context;

    let module_argument_value = compilation
      .get_module_graph()
      .module_by_identifier(&module.identifier())
      .expect("should have mgm")
      .get_module_argument();

    let module_argument = runtime_template.render_module_argument(module_argument_value);

    let content = if let Some(id) = &dep.id {
      match id.as_str() {
        "id" => runtime_template
          .runtime_requirements_mut()
          .insert(RuntimeGlobals::MODULE_ID),
        "loaded" => runtime_template
          .runtime_requirements_mut()
          .insert(RuntimeGlobals::MODULE_LOADED),
        _ => {}
      };

      // Check if there's a collision (user declared "module" at top level)
      // In that case, module_argument will be RspackModule instead of Module
      if module_argument_value == ModuleArgument::RspackModule {
        // There's a collision - user declared "module", so we need to use an internal variable
        let internal_var = format!("__webpack_internal_module_{id}__");
        
        // Add init fragment to declare the internal variable
        init_fragments.push(
          NormalInitFragment::new(
            format!("var {internal_var} = {module_argument}.{id};\n"),
            InitFragmentStage::StageConstants,
            0,
            InitFragmentKey::Const(internal_var.clone()),
            None,
          )
          .boxed(),
        );
        
        internal_var
      } else {
        // No collision - use normal approach
        format!("{module_argument}.{id}")
      }
    } else {
      // For __webpack_module__ without property access
      if module_argument_value == ModuleArgument::RspackModule {
        // There's a collision
        let internal_var = "__webpack_internal_module__".to_string();
        
        // Add init fragment to declare the internal variable
        init_fragments.push(
          NormalInitFragment::new(
            format!("var {internal_var} = {module_argument};\n"),
            InitFragmentStage::StageConstants,
            0,
            InitFragmentKey::Const(internal_var.clone()),
            None,
          )
          .boxed(),
        );
        
        internal_var
      } else {
        // No collision
        module_argument
      }
    };

    source.replace(dep.range.start, dep.range.end, content.as_str(), None);
  }
}
