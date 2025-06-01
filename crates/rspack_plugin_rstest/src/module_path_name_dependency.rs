use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AsContextDependency, AsModuleDependency, DependencyCodeGeneration, DependencyTemplate,
  DependencyTemplateType, DependencyType, InitFragmentExt, InitFragmentKey, InitFragmentStage,
  Module, NormalInitFragment, TemplateContext, TemplateReplaceSource,
};

#[cacheable]
#[derive(Debug, Clone, PartialEq)]
pub enum NameType {
  DirName,
  FileName,
}

#[cacheable]
#[derive(Debug, Clone)]
pub struct ModulePathNameDependency {
  r#type: NameType,
}

impl ModulePathNameDependency {
  pub fn new(r#type: NameType) -> Self {
    Self { r#type }
  }
}

#[cacheable_dyn]
impl DependencyCodeGeneration for ModulePathNameDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(ModulePathNameDependencyTemplate::template_type())
  }
}

impl AsModuleDependency for ModulePathNameDependency {}
impl AsContextDependency for ModulePathNameDependency {}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct ModulePathNameDependencyTemplate;

impl ModulePathNameDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Dependency(DependencyType::Rstest)
  }
}

impl DependencyTemplate for ModulePathNameDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    _source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let TemplateContext {
      module,
      init_fragments,
      ..
    } = code_generatable_context;

    let m = module.as_normal_module();
    if let Some(m) = m {
      let resource_path = &m.resource_resolved_data().resource_path;
      let context = &m.get_context();

      let dep = dep
        .as_any()
        .downcast_ref::<ModulePathNameDependency>()
        .expect("ModulePathNameDependencyTemplate can only be applied to ModulePathNameDependency");

      if dep.r#type == NameType::FileName {
        if let Some(resource_path) = resource_path {
          let init = NormalInitFragment::new(
            format!("const __filename = '{resource_path}';\n"),
            InitFragmentStage::StageConstants,
            0,
            InitFragmentKey::Const(format!("retest __filename {}", m.id())),
            None,
          );

          init_fragments.push(init.boxed());
        }
      } else if dep.r#type == NameType::DirName {
        if let Some(context) = context {
          let init = NormalInitFragment::new(
            format!("const __dirname = '{context}';\n"),
            InitFragmentStage::StageConstants,
            0,
            InitFragmentKey::Const(format!("retest __dirname {}", m.id())),
            None,
          );

          init_fragments.push(init.boxed());
        }
      }
    }
  }
}
