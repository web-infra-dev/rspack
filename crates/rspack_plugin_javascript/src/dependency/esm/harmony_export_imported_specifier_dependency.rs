use rspack_core::{
  export_from_import, get_exports_type, process_export_info, ConnectionState, Dependency,
  DependencyCategory, DependencyId, DependencyTemplate, DependencyType, ErrorSpan, ExportInfo,
  ExportsReferencedType, ExportsType, HarmonyExportInitFragment, ModuleDependency, ModuleGraph,
  ModuleIdentifier, RuntimeSpec, TemplateContext, TemplateReplaceSource,
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
  name: Option<JsWord>,
  resource_identifier: String,
}

impl HarmonyExportImportedSpecifierDependency {
  pub fn new(request: JsWord, ids: Vec<(JsWord, Option<JsWord>)>) -> Self {
    let resource_identifier = create_resource_identifier_for_esm_dependency(&request);
    Self {
      id: DependencyId::new(),
      name: None,
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

  fn get_referenced_exports(
    &self,
    module_graph: &ModuleGraph,
    runtime: Option<&RuntimeSpec>,
  ) -> ExportsReferencedType {
    let mode = get_mode(
      self.name.clone(),
      &self.ids.iter().map(|id| id.0.clone()).collect::<Vec<_>>(),
      module_graph,
      &self.id,
    );
    match mode.kind {
      ExportModeType::Missing
      | ExportModeType::Unused
      | ExportModeType::EmptyStar
      | ExportModeType::ReexportUndefined => ExportsReferencedType::No,
      ExportModeType::ReexportDynamicDefault | ExportModeType::DynamicReexport => {
        ExportsReferencedType::Object
      }
      ExportModeType::ReexportNamedDefault
      | ExportModeType::ReexportNamespaceObject
      | ExportModeType::ReexportFakeNamespaceObject => {
        if let Some(partial_namespace_export_info) = &mode.partial_namespace_export_info {
          let mut referenced_exports = vec![];
          process_export_info(
            module_graph,
            runtime,
            &mut referenced_exports,
            vec![],
            Some(partial_namespace_export_info),
            mode.kind == ExportModeType::ReexportFakeNamespaceObject,
            &mut Default::default(),
          );
          referenced_exports.into()
        } else {
          ExportsReferencedType::Object
        }
      }
      ExportModeType::NormalReexport => {
        let mut referenced_exports = vec![];
        if let Some(items) = mode.items {
          for item in items {
            if item.hidden {
              continue;
            }
            process_export_info(
              module_graph,
              runtime,
              &mut referenced_exports,
              item.ids,
              Some(item.export_info),
              false,
              &mut Default::default(),
            );
          }
        }
        referenced_exports.into()
      }
      ExportModeType::Unset => {
        unreachable!("should not export mode unset");
      }
    }
  }
}

#[allow(unused)]
#[derive(Debug, Default, PartialEq, Eq)]
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

#[derive(Debug)]
pub struct NormalReexportItem<'a> {
  pub name: JsWord,
  pub ids: Vec<JsWord>,
  pub hidden: bool,
  pub checked: bool,
  pub export_info: &'a ExportInfo,
}

#[derive(Debug, Default)]
pub struct ExportMode<'a> {
  pub kind: ExportModeType,
  pub items: Option<Vec<NormalReexportItem<'a>>>,
  pub name: Option<JsWord>,
  pub fake_type: Option<u8>,
  pub partial_namespace_export_info: Option<ExportInfo>,
}

// TODO cache get_mode result
#[allow(unused)]
pub fn get_mode<'a>(
  name: Option<JsWord>,
  ids: &Vec<JsWord>,
  module_graph: &'a ModuleGraph,
  id: &DependencyId,
) -> ExportMode<'a> {
  let parent_module = module_graph
    .parent_module_by_dependency_id(id)
    .expect("should have parent module");
  let exports_type = get_exports_type(module_graph, id, &parent_module);
  let exports_info = module_graph.get_exports_info(&parent_module);
  if let Some(name) = name.as_ref() && !ids.is_empty() && let Some(id) = ids.get(0) && id == "default" {
    match exports_type {
      ExportsType::Dynamic => {
        return ExportMode { kind: ExportModeType::ReexportDynamicDefault, name: Some(name.clone()), ..Default::default() }
      },
      ExportsType::DefaultOnly | ExportsType::DefaultWithNamed => {
        return ExportMode { kind: ExportModeType::ReexportNamedDefault, name: Some(name.clone()), ..Default::default() }
      },
      _ => {}
    }
  }

  if let Some(name) = name {
    let export_info = exports_info
      .id
      .get_read_only_export_info(&name, module_graph);
    if !ids.is_empty() {
      match exports_type {
        ExportsType::DefaultOnly => {
          return ExportMode {
            kind: ExportModeType::ReexportUndefined,
            name: Some(name),
            ..Default::default()
          }
        }
        _ => {
          return ExportMode {
            kind: ExportModeType::NormalReexport,
            items: Some(vec![NormalReexportItem {
              name,
              ids: ids.to_vec(),
              hidden: false,
              checked: false,
              export_info,
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
            name: Some(name),
            fake_type: Some(0),
            ..Default::default()
          }
        }
        ExportsType::DefaultWithNamed => {
          return ExportMode {
            kind: ExportModeType::ReexportFakeNamespaceObject,
            name: Some(name),
            fake_type: Some(2),
            ..Default::default()
          }
        }
        _ => {
          return ExportMode {
            kind: ExportModeType::ReexportNamespaceObject,
            name: Some(name),
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
