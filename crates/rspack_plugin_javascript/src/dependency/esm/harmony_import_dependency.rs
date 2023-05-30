use rspack_core::{
  get_import_var, import_statement, CodeGeneratable, CodeGeneratableContext, CodeGeneratableResult,
  CodeReplaceSourceDependency, CodeReplaceSourceDependencyContext,
  CodeReplaceSourceDependencyReplaceSource, Dependency, DependencyCategory, DependencyId,
  DependencyType, ErrorSpan, InitFragment, InitFragmentStage, ModuleDependency,
};
use swc_core::ecma::atoms::JsWord;

use super::HarmonyImportSpecifierDependency;

#[derive(Debug, Clone)]
pub struct HarmonyImportDependency {
  // pub start: u32,
  // pub end: u32,
  pub request: JsWord,
  pub id: Option<DependencyId>,
  pub span: Option<ErrorSpan>,
  pub refs: Vec<HarmonyImportSpecifierDependency>,
}

impl HarmonyImportDependency {
  pub fn new(
    request: JsWord,
    span: Option<ErrorSpan>,
    refs: Vec<HarmonyImportSpecifierDependency>,
  ) -> Self {
    Self {
      request,
      span,
      id: None,
      refs,
    }
  }
}

impl CodeReplaceSourceDependency for HarmonyImportDependency {
  fn apply(
    &self,
    source: &mut CodeReplaceSourceDependencyReplaceSource,
    code_generatable_context: &mut CodeReplaceSourceDependencyContext,
  ) {
    let id: DependencyId = self.id().expect("should have dependency id");

    self
      .refs
      .iter()
      .for_each(|dep| dep.apply(source, code_generatable_context, &id, self.request.as_ref()));

    let content: (String, String) =
      import_statement(code_generatable_context, &id, &self.request, false);

    let CodeReplaceSourceDependencyContext {
      init_fragments,
      compilation,
      ..
    } = code_generatable_context;

    let ref_module = compilation
      .module_graph
      .module_identifier_by_dependency_id(&id)
      .expect("should have dependency referenced module");

    if compilation.module_graph.is_async(ref_module) {
      let import_var = get_import_var(&self.request);
      init_fragments.push(InitFragment::new(
        content.0,
        InitFragmentStage::STAGE_HARMONY_IMPORTS,
        None,
      ));
      init_fragments.push(InitFragment::new(
        format!(
          "var __webpack_async_dependencies__ = __webpack_handle_async_dependencies__([{import_var}]);\n([{import_var}] = __webpack_async_dependencies__.then ? (await __webpack_async_dependencies__)() : __webpack_async_dependencies__);"
        ),
        InitFragmentStage::STAGE_HARMONY_IMPORTS,
        None,
      ));
      init_fragments.push(InitFragment::new(
        content.1,
        InitFragmentStage::STAGE_ASYNC_HARMONY_IMPORTS,
        None,
      ));
    } else {
      init_fragments.push(InitFragment::new(
        format!("{}{}", content.0, content.1),
        InitFragmentStage::STAGE_HARMONY_IMPORTS,
        None,
      ));
    }
  }
}

impl Dependency for HarmonyImportDependency {
  fn id(&self) -> Option<DependencyId> {
    self.id
  }
  fn set_id(&mut self, id: Option<DependencyId>) {
    self.id = id;
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Esm
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::EsmImport
  }
}

impl ModuleDependency for HarmonyImportDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.request
  }

  fn span(&self) -> Option<&ErrorSpan> {
    self.span.as_ref()
  }

  fn as_code_replace_source_dependency(&self) -> Option<Box<dyn CodeReplaceSourceDependency>> {
    Some(Box::new(self.clone()))
  }
}

impl CodeGeneratable for HarmonyImportDependency {
  fn generate(
    &self,
    _code_generatable_context: &mut CodeGeneratableContext,
  ) -> rspack_error::Result<CodeGeneratableResult> {
    todo!()
  }
}
