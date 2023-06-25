use rspack_core::{
  get_import_var, import_statement, tree_shaking::visitor::SymbolRef, CodeGeneratableContext,
  CodeGeneratableDependency, CodeGeneratableSource, Dependency, DependencyCategory, DependencyId,
  DependencyType, ErrorSpan, InitFragment, InitFragmentStage, ModuleDependency, RuntimeGlobals,
};
use rspack_symbol::IndirectTopLevelSymbol;
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
  pub specifiers: Vec<(JsWord, Option<JsWord>)>,
  pub dependency_type: DependencyType,
  pub export_all: bool,
}

impl HarmonyImportDependency {
  pub fn new(
    request: JsWord,
    span: Option<ErrorSpan>,
    refs: Vec<HarmonyImportSpecifierDependency>,
    specifiers: Vec<(JsWord, Option<JsWord>)>,
    dependency_type: DependencyType,
    export_all: bool,
  ) -> Self {
    Self {
      request,
      span,
      id: None,
      refs,
      specifiers,
      dependency_type,
      export_all,
    }
  }
}

impl CodeGeneratableDependency for HarmonyImportDependency {
  fn apply(
    &self,
    source: &mut CodeGeneratableSource,
    code_generatable_context: &mut CodeGeneratableContext,
  ) {
    let compilation = &code_generatable_context.compilation;
    let module = &code_generatable_context.module;
    let id: DependencyId = self.id().expect("should have dependency id");

    let ref_mgm = compilation
      .module_graph
      .module_graph_module_by_dependency_id(&id)
      .expect("should have ref module");
    if !compilation
      .include_module_ids
      .contains(&ref_mgm.module_identifier)
    {
      return;
    }

    if !self.export_all {
      let specifiers = self
        .specifiers
        .iter()
        .filter(|(local, imported)| {
          if !ref_mgm.module_type.is_js_like() {
            return true;
          }

          if let Some(imported) = imported {
            if imported == "namespace" {
              return true;
            }
            if imported == "default" {
              return compilation.used_symbol_ref.contains(&SymbolRef::Indirect(
                IndirectTopLevelSymbol {
                  src: ref_mgm.module_identifier,
                  ty: rspack_symbol::IndirectType::ImportDefault(local.clone()),
                  importer: module.identifier(),
                },
              ));
            }
          }

          let symbol = if matches!(self.dependency_type, DependencyType::EsmImport) {
            SymbolRef::Indirect(IndirectTopLevelSymbol {
              src: ref_mgm.module_identifier,
              ty: rspack_symbol::IndirectType::Import(local.clone(), imported.clone()),
              importer: module.identifier(),
            })
          } else {
            SymbolRef::Indirect(IndirectTopLevelSymbol {
              src: module.identifier(),
              ty: rspack_symbol::IndirectType::ReExport(local.clone(), imported.clone()),
              importer: module.identifier(),
            })
          };

          compilation.used_symbol_ref.contains(&symbol)
        })
        .collect::<Vec<_>>();

      if specifiers.is_empty()
        && compilation
          .side_effects_free_modules
          .contains(&ref_mgm.module_identifier)
      {
        return;
      }
    }

    self
      .refs
      .iter()
      .for_each(|dep| dep.apply(source, code_generatable_context, &id, self.request.as_ref()));

    let content: (String, String) =
      import_statement(code_generatable_context, &id, &self.request, false);

    let CodeGeneratableContext {
      init_fragments,
      compilation,
      module,
      runtime_requirements,
      ..
    } = code_generatable_context;

    let ref_module = compilation
      .module_graph
      .module_identifier_by_dependency_id(&id)
      .expect("should have dependency referenced module");
    let import_var = get_import_var(&self.request);
    if compilation.module_graph.is_async(ref_module) {
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
    if self.export_all {
      runtime_requirements.add(RuntimeGlobals::EXPORT_STAR);
      let exports_argument = compilation
        .module_graph
        .module_graph_module_by_identifier(&module.identifier())
        .expect("should have mgm")
        .get_exports_argument();
      init_fragments.push(InitFragment::new(
        format!(
          "{}.{}({import_var}, {exports_argument});\n",
          RuntimeGlobals::REQUIRE,
          RuntimeGlobals::EXPORT_STAR,
        ),
        if compilation.module_graph.is_async(ref_module) {
          InitFragmentStage::STAGE_ASYNC_HARMONY_IMPORTS
        } else {
          InitFragmentStage::STAGE_HARMONY_IMPORTS
        },
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
    &self.dependency_type
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

  fn as_code_generatable_dependency(&self) -> Option<Box<&dyn CodeGeneratableDependency>> {
    Some(Box::new(self))
  }

  fn set_request(&mut self, request: String) {
    self.request = request.into();
  }
}
