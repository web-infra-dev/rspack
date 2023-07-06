use std::collections::HashSet;

use rspack_core::{
  export_from_import, get_exports_type, tree_shaking::visitor::SymbolRef, DependencyId,
  DependencyTemplate, ExportsType, InitFragment, InitFragmentStage, ModuleIdentifier,
  RuntimeGlobals, TemplateContext, TemplateReplaceSource,
};
use rspack_symbol::{IndirectType, StarSymbolKind, SymbolType, DEFAULT_JS_WORD};
use swc_core::ecma::atoms::JsWord;

use super::format_exports;

// Create _webpack_require__.d(__webpack_exports__, {}).
// import { a } from 'a'; export { a }
#[derive(Debug)]
pub struct HarmonyExportImportedSpecifierDependency {
  pub request: JsWord,
  pub ids: Vec<(JsWord, Option<JsWord>)>,
  module_identifier: ModuleIdentifier,
}

impl HarmonyExportImportedSpecifierDependency {
  pub fn new(
    request: JsWord,
    ids: Vec<(JsWord, Option<JsWord>)>,
    module_identifier: ModuleIdentifier,
  ) -> Self {
    Self {
      request,
      ids,
      module_identifier,
    }
  }
}

impl DependencyTemplate for HarmonyExportImportedSpecifierDependency {
  fn apply(
    &self,
    _source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let compilation = &code_generatable_context.compilation;
    let module = &code_generatable_context.module;
    let dependency_id = compilation
      .module_graph
      .dependencies_by_module_identifier(&self.module_identifier)
      .expect("should have dependencies")
      .iter()
      .map(|id| {
        compilation
          .module_graph
          .dependency_by_id(id)
          .expect("should have dependency")
      })
      .find(|d| d.request() == &self.request)
      .expect("should have dependency")
      .id();

    let import_var = compilation
      .module_graph
      .get_import_var(&module.identifier(), &self.request);

    let used_exports = if compilation.options.builtins.tree_shaking.is_true() {
      let set = compilation
        .used_symbol_ref
        .iter()
        .filter_map(|item| match item {
          SymbolRef::Declaration(decl) if decl.src() == module.identifier() => {
            if *decl.ty() == SymbolType::Temp {
              if let Some(key) = &self.ids.iter().find(|e| {
                if e.1.is_some() {
                  e.0 == *d.exported()
                } else {
                  false
                }
              }) {
                return Some(&key.0);
              }
            }
            Some(&decl.id().atom)
          }
          SymbolRef::Indirect(i) if i.importer == module.identifier() && i.is_reexport() => {
            Some(i.id())
          }
          SymbolRef::Indirect(i) if i.src == module.identifier() => match i.ty {
            IndirectType::Import(_, _) => Some(i.indirect_id()),
            IndirectType::ImportDefault(_) => Some(&DEFAULT_JS_WORD),
            _ => None,
          },
          SymbolRef::Star(s)
            if s.module_ident == module.identifier() && s.ty() == StarSymbolKind::ReExportAllAs =>
          {
            Some(s.binding())
          }
          _ => None,
        })
        .collect::<HashSet<_>>();
      Some(set)
    } else {
      None
    };

    let mut exports = vec![];

    for id in &self.ids {
      if used_exports.is_none() || matches!(used_exports.as_ref(), Some(x) if x.contains(&id.0)) {
        exports.push((
          id.0.clone(),
          JsWord::from(export_from_import(
            code_generatable_context,
            true,
            import_var.clone(),
            id.1.clone().map(|i| vec![i]).unwrap_or_default(),
            dependency_id,
            false,
          )),
        ));
      }
    }

    if !exports.is_empty() {
      let TemplateContext {
        runtime_requirements,
        init_fragments,
        compilation,
        module,
        ..
      } = code_generatable_context;

      let exports_argument = compilation
        .module_graph
        .module_graph_module_by_identifier(&module.identifier())
        .expect("should have mgm")
        .get_exports_argument();
      runtime_requirements.insert(RuntimeGlobals::EXPORTS);
      runtime_requirements.insert(RuntimeGlobals::DEFINE_PROPERTY_GETTERS);
      init_fragments.push(InitFragment::new(
        format!(
          "{}({exports_argument}, {});\n",
          RuntimeGlobals::DEFINE_PROPERTY_GETTERS,
          format_exports(&exports)
        ),
        InitFragmentStage::STAGE_HARMONY_EXPORTS,
        None,
      ));
    }
  }
}

#[allow(unused)]
#[derive(Debug, Default)]
pub enum ExportModeType {
  #[default]
  Unset,
  Missing,
  Unused,
  EmptyStar,
  ReexportDynamicDefault,
  ReexportNamedDefault,
  ReexportNamespaceObject,
  ReexportFakeNamespaceObject,
  ReexportUndefined,
  NormalReexport,
  DynamicReexport,
}

#[derive(Debug, Default)]
pub struct NormalReexportItem {
  pub name: String,
  pub ids: Vec<String>,
}

#[derive(Debug, Default)]
pub struct ExportMode {
  pub kind: ExportModeType,
  pub items: Option<Vec<NormalReexportItem>>,
  pub name: Option<String>,
  pub fake_type: Option<u8>,
}

#[allow(unused)]
pub fn get_mode(
  name: Option<String>,
  ids: Vec<String>,
  code_generatable_context: &mut TemplateContext,
  id: &DependencyId,
) -> ExportMode {
  let TemplateContext {
    compilation,
    module,
    ..
  } = code_generatable_context;

  let exports_type = get_exports_type(&compilation.module_graph, id, &module.identifier());

  if let Some(name) = &name && !ids.is_empty() && let Some(id) = ids.get(0) && *id == "default" {
    match exports_type {
      ExportsType::Dynamic => {
        return ExportMode { kind: ExportModeType::ReexportDynamicDefault, name: Some(name.to_string()), ..Default::default() }
      },
      ExportsType::DefaultOnly | ExportsType::DefaultWithNamed => {
        return ExportMode { kind: ExportModeType::ReexportNamedDefault, name: Some(name.to_string()), ..Default::default() }
      },
      _ => {}
    }
  }

  if let Some(name) = &name {
    if !ids.is_empty() {
      match exports_type {
        ExportsType::DefaultOnly => {
          return ExportMode {
            kind: ExportModeType::ReexportUndefined,
            name: Some(name.to_string()),
            ..Default::default()
          }
        }
        _ => {
          return ExportMode {
            kind: ExportModeType::NormalReexport,
            items: Some(vec![NormalReexportItem {
              name: name.to_string(),
              ids,
            }]),
            ..Default::default()
          }
        }
      }
    } else {
      match exports_type {
        ExportsType::DefaultOnly => {
          return ExportMode {
            kind: ExportModeType::ReexportFakeNamespaceObject,
            name: Some(name.to_string()),
            fake_type: Some(0),
            ..Default::default()
          }
        }
        ExportsType::DefaultWithNamed => {
          return ExportMode {
            kind: ExportModeType::ReexportFakeNamespaceObject,
            name: Some(name.to_string()),
            fake_type: Some(2),
            ..Default::default()
          }
        }
        _ => {
          return ExportMode {
            kind: ExportModeType::ReexportNamespaceObject,
            name: Some(name.to_string()),
            ..Default::default()
          }
        }
      }
    }
  }
  // todo star reexporting

  ExportMode {
    kind: ExportModeType::NormalReexport,
    items: Some(vec![]),
    ..Default::default()
  }
}
