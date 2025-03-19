use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AsContextDependency, CodeGenerationDataFilename, CodeGenerationDataUrl, Compilation, Dependency,
  DependencyCategory, DependencyId, DependencyRange, DependencyTemplate, DependencyType,
  FactorizeInfo, ModuleDependency, ModuleIdentifier, PublicPath, RuntimeSpec, TemplateContext,
  TemplateReplaceSource,
};

use crate::utils::{css_escape_string, AUTO_PUBLIC_PATH_PLACEHOLDER};

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
    // Here we need the asset module's runtime to get the code generation result, which is not equal to
    // the css module's runtime, but actually multiple runtime optimization doesn't affect asset module,
    // in different runtime asset module will always have the same code generation result, so we use
    // `runtime: None` to get the only one code generation result
    let code_gen_result = compilation.code_generation_results.get(identifier, None);
    if let Some(url) = code_gen_result.data.get::<CodeGenerationDataUrl>() {
      Some(url.inner().to_string())
    } else if let Some(data) = code_gen_result.data.get::<CodeGenerationDataFilename>() {
      let filename = data.filename();
      let public_path = match data.public_path() {
        PublicPath::Filename(p) => PublicPath::render_filename(compilation, p),
        PublicPath::Auto => AUTO_PUBLIC_PATH_PLACEHOLDER.to_string(),
      };
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

  fn range(&self) -> Option<&DependencyRange> {
    Some(&self.range)
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

  fn set_request(&mut self, request: String) {
    self.request = request;
  }

  fn factorize_info(&self) -> &FactorizeInfo {
    &self.factorize_info
  }

  fn factorize_info_mut(&mut self) -> &mut FactorizeInfo {
    &mut self.factorize_info
  }
}

#[cacheable_dyn]
impl DependencyTemplate for CssUrlDependency {
  fn apply(
    &self,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let TemplateContext { compilation, .. } = code_generatable_context;
    if let Some(mgm) = compilation
      .get_module_graph()
      .module_graph_module_by_dependency_id(self.id())
      && let Some(target_url) = self.get_target_url(&mgm.module_identifier, compilation)
    {
      let target_url = css_escape_string(&target_url);
      let content = if self.replace_function {
        format!("url({target_url})")
      } else {
        target_url
      };
      source.replace(self.range.start, self.range.end, &content, None);
    }
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

impl AsContextDependency for CssUrlDependency {}
