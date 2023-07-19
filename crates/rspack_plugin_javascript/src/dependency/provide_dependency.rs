use rspack_core::{
  module_raw, property_access, Dependency, DependencyCategory, DependencyId, DependencyTemplate,
  DependencyType, ErrorSpan, InitFragment, InitFragmentStage, ModuleDependency, TemplateContext,
  TemplateReplaceSource,
};
use swc_core::ecma::atoms::JsWord;

#[derive(Debug, Clone)]
pub struct ProvideDependency {
  id: DependencyId,
  start: u32,
  end: u32,
  ids: Vec<String>,
  request: JsWord,
  identifier: JsWord,
}

impl ProvideDependency {
  pub fn new(start: u32, end: u32, ids: Vec<String>, request: JsWord, identifier: JsWord) -> Self {
    Self {
      id: DependencyId::new(),
      start,
      end,
      ids,
      request,
      identifier,
    }
  }
}

impl Dependency for ProvideDependency {
  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Esm
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::Provided
  }
}

impl ModuleDependency for ProvideDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.request
  }

  fn span(&self) -> Option<&ErrorSpan> {
    None
  }

  fn as_code_generatable_dependency(&self) -> Option<&dyn DependencyTemplate> {
    Some(self)
  }

  fn set_request(&mut self, request: String) {
    self.request = request.into();
  }
}

impl DependencyTemplate for ProvideDependency {
  fn apply(&self, source: &mut TemplateReplaceSource, template_context: &mut TemplateContext) {
    let TemplateContext {
      compilation,
      runtime_requirements,
      init_fragments,
      ..
    } = template_context;

    init_fragments.push(InitFragment::new(
      format!(
        "/* provided dependency */ var {} = {}{};\n",
        self.identifier,
        module_raw(
          compilation,
          runtime_requirements,
          &self.id,
          &self.request,
          false
        ),
        property_access(&self.ids, 0)
      ),
      InitFragmentStage::STAGE_PROVIDES,
      None,
    ));

    source.replace(self.start, self.end, &self.identifier, None);
  }
}
