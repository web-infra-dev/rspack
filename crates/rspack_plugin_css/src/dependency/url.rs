use cow_utils::CowUtils;
use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AsContextDependency, CodeGenerationDataFilename, CodeGenerationDataUrl, Compilation, Dependency,
  DependencyCategory, DependencyCodeGeneration, DependencyId, DependencyRange, DependencyTemplate,
  DependencyTemplateType, DependencyType, FactorizeInfo, ModuleDependency, ModuleIdentifier,
  TemplateContext, TemplateReplaceSource,
};

use crate::utils::{AUTO_PUBLIC_PATH_PLACEHOLDER, css_escape_string};

#[cacheable]
#[derive(Debug, Clone)]
pub struct CssUrlDependency {
  id: DependencyId,
  request: String,
  range: DependencyRange,
  replace_function: bool,
  factorize_info: FactorizeInfo,
}

impl CssUrlDependency {
  pub fn new(request: String, range: DependencyRange, replace_function: bool) -> Self {
    Self {
      request,
      range,
      id: DependencyId::new(),
      replace_function,
      factorize_info: Default::default(),
    }
  }

  fn get_target_url(
    &self,
    identifier: &ModuleIdentifier,
    compilation: &Compilation,
  ) -> Option<String> {
    // url points to asset modules, and asset modules should have same codegen results for all runtimes
    let code_gen_result = compilation.code_generation_results.get_one(identifier);

    if let Some(url) = code_gen_result.data.get::<CodeGenerationDataUrl>() {
      Some(url.inner().to_string())
    } else if let Some(data) = code_gen_result.data.get::<CodeGenerationDataFilename>() {
      let filename = data.filename();
      let public_path = data.public_path().cow_replace(
        "__RSPACK_PLUGIN_ASSET_AUTO_PUBLIC_PATH__",
        AUTO_PUBLIC_PATH_PLACEHOLDER,
      );
      Some(format!("{public_path}{filename}"))
    } else {
      None
    }
  }
}

#[cacheable_dyn]
impl Dependency for CssUrlDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Url
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::CssUrl
  }

  fn range(&self) -> Option<DependencyRange> {
    Some(self.range)
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::True
  }
}

#[cacheable_dyn]
impl ModuleDependency for CssUrlDependency {
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
impl DependencyCodeGeneration for CssUrlDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(CssUrlDependencyTemplate::template_type())
  }
}

impl AsContextDependency for CssUrlDependency {}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct CssUrlDependencyTemplate;

impl CssUrlDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Dependency(DependencyType::CssUrl)
  }
}

impl DependencyTemplate for CssUrlDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<CssUrlDependency>()
      .expect("CssUrlDependencyTemplate should be used for CssUrlDependency");

    let TemplateContext { compilation, .. } = code_generatable_context;
    if let Some(mgm) = compilation
      .get_module_graph()
      .module_graph_module_by_dependency_id(dep.id())
      && let Some(target_url) = dep.get_target_url(&mgm.module_identifier, compilation)
    {
      let target_url = css_escape_string(&target_url);
      let content = if dep.replace_function {
        format!("url({target_url})")
      } else {
        target_url
      };
      source.replace(dep.range.start, dep.range.end, &content, None);
    }
  }
}
