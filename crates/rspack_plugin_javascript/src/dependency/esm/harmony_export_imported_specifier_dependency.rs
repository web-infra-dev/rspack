use rspack_core::{
  export_from_import, get_exports_type, ConnectionState, Dependency, DependencyCategory,
  DependencyId, DependencyTemplate, DependencyType, ErrorSpan, ExportsType,
  HarmonyExportInitFragment, ModuleDependency, ModuleGraph, ModuleIdentifier, TemplateContext,
  TemplateReplaceSource,
};
use rustc_hash::FxHashSet as HashSet;
use swc_core::ecma::atoms::JsWord;

use super::create_resource_identifier_for_esm_dependency;

// Create _webpack_require__.d(__webpack_exports__, {}).
// import { a } from 'a'; export { a }
#[derive(Debug, Clone)]
pub struct HarmonyExportImportedSpecifierDependency {
  pub id: DependencyId,
  pub request: JsWord,
  pub ids: Vec<(JsWord, Option<JsWord>)>,
  resource_identifier: String,
}

impl HarmonyExportImportedSpecifierDependency {
  pub fn new(request: JsWord, ids: Vec<(JsWord, Option<JsWord>)>) -> Self {
    let resource_identifier = create_resource_identifier_for_esm_dependency(&request);
    Self {
      id: DependencyId::new(),
      request,
      ids,
      resource_identifier,
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

    let import_var = compilation
      .module_graph
      .get_import_var(&module.identifier(), &self.request);

    let used_exports = if compilation.options.builtins.tree_shaking.is_true() {
      Some(
        compilation
          .module_graph
          .get_exports_info(&module.identifier())
          .get_used_exports(),
      )
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
            import_var,
            id.1.clone().map(|i| vec![i]).unwrap_or_default(),
            &self.id,
            false,
            false,
          )),
        ));
      }
    }

    for export in exports {
      code_generatable_context
        .init_fragments
        .push(Box::new(HarmonyExportInitFragment::new(export)));
    }
  }
}

impl Dependency for HarmonyExportImportedSpecifierDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Esm
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::EsmExportImportedSpecifier
  }
}

impl ModuleDependency for HarmonyExportImportedSpecifierDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.request
  }

  fn span(&self) -> Option<&ErrorSpan> {
    None
  }

  fn set_request(&mut self, request: String) {
    self.request = request.into();
  }

  fn resource_identifier(&self) -> Option<&str> {
    Some(&self.resource_identifier)
  }

  fn get_module_evaluation_side_effects_state(
    &self,
    _module_graph: &ModuleGraph,
    _module_chain: &mut HashSet<ModuleIdentifier>,
  ) -> ConnectionState {
    ConnectionState::Bool(false)
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
