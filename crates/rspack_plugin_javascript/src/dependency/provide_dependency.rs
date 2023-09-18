use rspack_core::{
  create_exports_object_referenced, module_exports, module_raw, property_access, Dependency,
  DependencyCategory, DependencyId, DependencyTemplate, DependencyType, ErrorSpan,
  ExtendedReferencedExport, InitFragment, InitFragmentStage, ModuleDependency, NormalInitFragment,
  TemplateContext, TemplateReplaceSource,
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
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Esm
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::Provided
  }
}

impl ModuleDependency for ProvideDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.request
  }

  fn span(&self) -> Option<&ErrorSpan> {
    None
  }

  fn get_referenced_exports(
    &self,
    _module_graph: &rspack_core::ModuleGraph,
    _runtime: Option<&rspack_core::RuntimeSpec>,
  ) -> Vec<ExtendedReferencedExport> {
    if self.ids.is_empty() {
      return create_exports_object_referenced();
    }
    vec![ExtendedReferencedExport::Array(
      self
        .ids
        .iter()
        .map(|id| JsWord::from(id.as_str()))
        .collect::<Vec<JsWord>>(),
    )]
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

    init_fragments.push(Box::new(NormalInitFragment::new(
      format!(
        "/* provided dependency */ var {} = {}{};\n",
        self.identifier,
        module_exports(
          compilation,
          &self.id,
          &self.request,
          false,
          runtime_requirements,
        ),
        property_access(&self.ids, 0)
      ),
      InitFragmentStage::StageProvides,
      None,
    )));

    source.replace(self.start, self.end, &self.identifier, None);
  }
}
