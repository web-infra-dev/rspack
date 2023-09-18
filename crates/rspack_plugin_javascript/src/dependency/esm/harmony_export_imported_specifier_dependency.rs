use rspack_core::{
  create_exports_object_referenced, create_no_exports_referenced, export_from_import,
  get_exports_type, process_export_info, ConnectionState, Dependency, DependencyCategory,
  DependencyCondition, DependencyId, DependencyTemplate, DependencyType, ErrorSpan, ExportInfoId,
  ExportInfoProvided, ExportsType, ExtendedReferencedExport, HarmonyExportInitFragment,
  ModuleDependency, ModuleGraph, ModuleIdentifier, RuntimeSpec, TemplateContext,
  TemplateReplaceSource, UsageState,
};
use rustc_hash::FxHashSet as HashSet;
use swc_core::ecma::atoms::JsWord;

use super::create_resource_identifier_for_esm_dependency;

// Create _webpack_require__.d(__webpack_exports__, {}).
// case1: `import { a } from 'a'; export { a }`
// case2: `export { a } from 'a';`
// TODO case3: `export * from 'a'`
#[derive(Debug, Clone)]
pub struct HarmonyExportImportedSpecifierDependency {
  pub id: DependencyId,
  pub request: JsWord,
  pub ids: Vec<(JsWord, Option<JsWord>)>,
  name: Option<JsWord>,
  resource_identifier: String,
  // Because it is shared by multiply HarmonyExportImportedSpecifierDependency, so put it to `BuildInfo`
  // pub active_exports: HashSet<JsWord>,
  // pub all_star_exports: Option<Vec<DependencyId>>,
  pub other_star_exports: Option<Vec<DependencyId>>, // look like it is unused
}

impl HarmonyExportImportedSpecifierDependency {
  pub fn new(request: JsWord, ids: Vec<(JsWord, Option<JsWord>)>, name: Option<JsWord>) -> Self {
    let resource_identifier = create_resource_identifier_for_esm_dependency(&request);
    Self {
      id: DependencyId::new(),
      name,
      request,
      ids,
      resource_identifier,
      other_star_exports: None,
    }
  }

  pub fn active_exports<'a>(&self, module_graph: &'a ModuleGraph) -> &'a HashSet<JsWord> {
    dbg!(&module_graph.module_graph_module_by_dependency_id(&self.id));
    let build_info = module_graph
      .parent_module_by_dependency_id(&self.id)
      .and_then(|item| module_graph.module_graph_module_by_identifier(&item))
      .expect("should have mgm")
      .build_info
      .as_ref()
      .expect("should have build info");
    &build_info.harmony_named_exports
  }

  pub fn all_star_exports<'a>(&self, module_graph: &'a ModuleGraph) -> &'a Vec<DependencyId> {
    let build_info = module_graph
      .module_graph_module_by_dependency_id(&self.id)
      .expect("should have mgm")
      .build_info
      .as_ref()
      .expect("should have build info");
    &build_info.all_star_exports
  }

  // TODO cache get_mode result
  #[allow(unused)]
  pub fn get_mode(
    &self,
    name: Option<JsWord>,
    ids: &Vec<JsWord>,
    module_graph: &ModuleGraph,
    id: &DependencyId,
    runtime: Option<&RuntimeSpec>,
  ) -> ExportMode {
    let imported_module_identifier = if let Some(imported_module_identifier) =
      module_graph.module_identifier_by_dependency_id(id)
    {
      imported_module_identifier
    } else {
      return ExportMode::new(ExportModeType::Missing);
    };

    let parent_module = module_graph
      .parent_module_by_dependency_id(id)
      .expect("should have parent module");
    let exports_type = get_exports_type(module_graph, id, &parent_module);
    let exports_info = module_graph.get_exports_info(&parent_module);

    // Special handling for reexporting the default export
    // from non-namespace modules
    if let Some(name) = name.as_ref() && !ids.is_empty() && let Some(id) = ids.get(0) && id == "default" {
      match exports_type {
        ExportsType::Dynamic => {
          let mut export_mode = ExportMode::new(ExportModeType::ReexportDynamicDefault, );
          export_mode.name = Some(name.clone());
          return export_mode;
        },
        ExportsType::DefaultOnly | ExportsType::DefaultWithNamed => {
          let export_info = exports_info.id.get_read_only_export_info(name, module_graph).id;
          let mut export_mode = ExportMode::new( ExportModeType::ReexportNamedDefault);
          export_mode.name = Some(name.clone());
          export_mode.partial_namespace_export_info = Some(export_info);
          return export_mode;
        },
        _ => {}
      }
    }

    // reexporting with a fixed name
    if let Some(name) = name {
      let export_info = exports_info
        .id
        .get_read_only_export_info(&name, module_graph)
        .id;
      if !ids.is_empty() {
        // export { name as name }
        match exports_type {
          ExportsType::DefaultOnly => {
            let mut export_mode = ExportMode::new(ExportModeType::ReexportUndefined);
            export_mode.name = Some(name);
            return export_mode;
          }
          _ => {
            let mut export_mode = ExportMode::new(ExportModeType::NormalReexport);
            export_mode.items = Some(vec![NormalReexportItem {
              name,
              ids: ids.to_vec(),
              hidden: false,
              checked: false,
              export_info,
            }]);
            return export_mode;
          }
        }
      } else {
        // export * as name
        match exports_type {
          ExportsType::DefaultOnly => {
            let mut export_mode = ExportMode::new(ExportModeType::ReexportFakeNamespaceObject);
            export_mode.name = Some(name);
            export_mode.partial_namespace_export_info = Some(export_info);
            export_mode.fake_type = 0;
            return export_mode;
          }
          ExportsType::DefaultWithNamed => {
            let mut export_mode = ExportMode::new(ExportModeType::ReexportFakeNamespaceObject);
            export_mode.name = Some(name);
            export_mode.partial_namespace_export_info = Some(export_info);
            export_mode.fake_type = 2;
            return export_mode;
          }
          _ => {
            let mut export_mode = ExportMode::new(ExportModeType::ReexportNamespaceObject);
            export_mode.name = Some(name);
            export_mode.partial_namespace_export_info = Some(export_info);
            return export_mode;
          }
        }
      }
    }

    let StarReexportsInfo {
      exports,
      checked,
      ignored_exports,
      hidden,
    } = self.get_star_reexports(module_graph, runtime, imported_module_identifier);
    dbg!(&exports);
    if let Some(exports) = exports {
      if exports.is_empty() {
        let mut export_mode = ExportMode::new(ExportModeType::EmptyStar);
        export_mode.hidden = hidden;
        return export_mode;
      }

      let mut items = exports
        .into_iter()
        .map(|export_name| NormalReexportItem {
          name: export_name.clone(),
          ids: vec![export_name.clone()],
          hidden: false,
          checked: checked.as_ref().map(|c| c.contains(&export_name)).is_some(),
          export_info: exports_info
            .id
            .get_read_only_export_info(&export_name, module_graph)
            .id,
        })
        .collect::<Vec<_>>();

      if let Some(hidden) = &hidden {
        for export_name in hidden.iter() {
          items.push(NormalReexportItem {
            name: export_name.clone(),
            ids: vec![export_name.clone()],
            hidden: true,
            checked: false,
            export_info: exports_info
              .id
              .get_read_only_export_info(export_name, module_graph)
              .id,
          });
        }
      }
      let mut export_mode = ExportMode::new(ExportModeType::NormalReexport);
      export_mode.items = Some(items);
      export_mode
    } else {
      let mut export_mode = ExportMode::new(ExportModeType::DynamicReexport);
      export_mode.ignored = Some(ignored_exports);
      export_mode.hidden = hidden;
      export_mode
    }
  }

  pub fn get_star_reexports(
    &self,
    module_graph: &ModuleGraph,
    runtime: Option<&RuntimeSpec>,
    imported_module_identifier: &ModuleIdentifier,
  ) -> StarReexportsInfo {
    let imported_exports_info = module_graph.get_exports_info(imported_module_identifier);
    let other_export_info = module_graph
      .export_info_map
      .get(&imported_exports_info.other_exports_info)
      .expect("should have export info");
    let no_extra_exports = matches!(other_export_info.provided, Some(ExportInfoProvided::False));
    let no_extra_imports = matches!(other_export_info.get_used(runtime), UsageState::Unused);
    let ignored_exports: HashSet<JsWord> = {
      let mut e = self.active_exports(module_graph).clone();
      e.insert("default".into());
      e
    };
    let mut hidden_exports = self.discover_active_exports_from_other_star_exports(module_graph);
    if !no_extra_exports && !no_extra_imports {
      if let Some(hidden_exports) = hidden_exports.as_mut() {
        for e in ignored_exports.iter() {
          hidden_exports.remove(e);
        }
      }
      return StarReexportsInfo {
        ignored_exports,
        hidden: hidden_exports,
        ..Default::default()
      };
    }

    let mut exports = HashSet::default();
    let mut checked = HashSet::default();
    let mut hidden = if hidden_exports.is_some() {
      Some(HashSet::default())
    } else {
      None
    };

    let parent_module = module_graph
      .parent_module_by_dependency_id(&self.id)
      .expect("should have parent module");
    let exports_info = module_graph.get_exports_info(&parent_module);

    if no_extra_imports {
      for export_info_id in exports_info.get_ordered_exports() {
        let export_info = module_graph
          .export_info_map
          .get(export_info_id)
          .expect("should have export info");
        let export_name = export_info.name.clone().unwrap_or_default();
        // dbg!(
        //   &export_info.get_used(runtime),
        //   &ignored_exports,
        //   &export_name
        // );
        if ignored_exports.contains(&export_name)
          || matches!(export_info.get_used(runtime), UsageState::Unused)
        {
          println!("ignored by ignore exprots");
          continue;
        }

        dbg!(&export_info);
        let imported_export_info = imported_exports_info
          .id
          .get_read_only_export_info(&export_name, module_graph);
        if matches!(
          imported_export_info.provided,
          Some(ExportInfoProvided::False)
        ) {
          continue;
        }
        if let Some(hidden) = hidden.as_mut() && hidden_exports.as_ref()
          .map(|hidden_exports| hidden_exports.contains(&export_name))
          .is_some()
        {
          hidden.insert(export_name.clone());
          continue;
        }
        exports.insert(export_name.clone());
        if matches!(
          imported_export_info.provided,
          Some(ExportInfoProvided::True)
        ) {
          continue;
        }
        checked.insert(export_name);
      }
    } else if no_extra_exports {
      for import_export_info_id in imported_exports_info.get_ordered_exports() {
        let import_export_info = module_graph
          .export_info_map
          .get(import_export_info_id)
          .expect("should have export info");
        let import_export_info_name = import_export_info.name.clone().unwrap_or_default();
        if ignored_exports.contains(&import_export_info_name)
          || matches!(import_export_info.provided, Some(ExportInfoProvided::False))
        {
          continue;
        }
        let export_info = exports_info
          .id
          .get_read_only_export_info(&import_export_info_name, module_graph);
        if matches!(export_info.get_used(runtime), UsageState::Unused) {
          continue;
        }
        if let Some(hidden) = hidden.as_mut() && hidden_exports.as_ref()
          .map(|hidden_exports| hidden_exports.contains(&import_export_info_name))
          .is_some()
        {
          hidden.insert(import_export_info_name.clone());
          continue;
        }
        exports.insert(import_export_info_name.clone());
        if matches!(import_export_info.provided, Some(ExportInfoProvided::True)) {
          continue;
        }
        checked.insert(import_export_info_name);
      }
    }

    StarReexportsInfo {
      ignored_exports,
      exports: Some(exports),
      checked: Some(checked),
      hidden,
    }
  }

  pub fn discover_active_exports_from_other_star_exports(
    &self,
    module_graph: &ModuleGraph,
  ) -> Option<HashSet<JsWord>> {
    if let Some(other_star_exports) = &self.other_star_exports {
      if other_star_exports.is_empty() {
        return None;
      }
    }

    let all_star_exports = self.all_star_exports(module_graph);
    if !all_star_exports.is_empty() {
      let names = determine_export_assignments(module_graph, all_star_exports.clone(), None);
      return Some(names);
    }

    if let Some(other_star_exports) = &self.other_star_exports {
      let names =
        determine_export_assignments(module_graph, other_star_exports.clone(), Some(self.id));
      return Some(names);
    }
    None
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
          .old_get_used_exports(),
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

  fn get_condition(&self) -> Option<DependencyCondition> {
    let id = self.id;
    Some(DependencyCondition::Fn(Box::new(
      move |_mc, runtime, module_graph: &ModuleGraph| {
        let dep = module_graph
          .dependency_by_id(&id)
          .expect("should have dependency");
        let down_casted_dep = dep
          .downcast_ref::<HarmonyExportImportedSpecifierDependency>()
          .expect("should be HarmonyExportImportedSpecifierDependency");
        let mode = down_casted_dep.get_mode(
          down_casted_dep.name.clone(),
          &down_casted_dep
            .ids
            .iter()
            .map(|id| id.0.clone())
            .collect::<Vec<_>>(),
          module_graph,
          &down_casted_dep.id,
          runtime,
        );
        dbg!(&mode);
        ConnectionState::Bool(!matches!(
          mode.ty,
          ExportModeType::Unused | ExportModeType::EmptyStar
        ))
      },
    )))
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
  ) -> Vec<ExtendedReferencedExport> {
    let mode = self.get_mode(
      self.name.clone(),
      &self.ids.iter().map(|id| id.0.clone()).collect::<Vec<_>>(),
      module_graph,
      &self.id,
      runtime,
    );
    match mode.ty {
      ExportModeType::Missing
      | ExportModeType::Unused
      | ExportModeType::EmptyStar
      | ExportModeType::ReexportUndefined => create_no_exports_referenced(),
      ExportModeType::ReexportDynamicDefault | ExportModeType::DynamicReexport => {
        create_exports_object_referenced()
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
            Some(*partial_namespace_export_info),
            mode.ty == ExportModeType::ReexportFakeNamespaceObject,
            &mut Default::default(),
          );
          referenced_exports
            .into_iter()
            .map(ExtendedReferencedExport::Array)
            .collect::<Vec<_>>()
        } else {
          create_exports_object_referenced()
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
        referenced_exports
          .into_iter()
          .map(ExtendedReferencedExport::Array)
          .collect::<Vec<_>>()
      }
    }
  }
}

#[allow(unused)]
#[derive(Debug, PartialEq, Eq)]
pub enum ExportModeType {
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
pub struct NormalReexportItem {
  pub name: JsWord,
  pub ids: Vec<JsWord>,
  pub hidden: bool,
  pub checked: bool,
  pub export_info: ExportInfoId,
}

#[derive(Debug)]
pub struct ExportMode {
  /// corresponding to `type` field in webpack's `EpxortMode`
  pub ty: ExportModeType,
  pub items: Option<Vec<NormalReexportItem>>,
  pub name: Option<JsWord>,
  pub fake_type: u8,
  pub partial_namespace_export_info: Option<ExportInfoId>,
  pub ignored: Option<HashSet<JsWord>>,
  pub hidden: Option<HashSet<JsWord>>,
}

impl ExportMode {
  pub fn new(ty: ExportModeType) -> Self {
    Self {
      ty,
      items: None,
      name: None,
      fake_type: 0,
      partial_namespace_export_info: None,
      ignored: None,
      hidden: None,
    }
  }
}

#[derive(Debug, Default)]
pub struct StarReexportsInfo {
  exports: Option<HashSet<JsWord>>,
  checked: Option<HashSet<JsWord>>,
  ignored_exports: HashSet<JsWord>,
  hidden: Option<HashSet<JsWord>>,
}

fn determine_export_assignments(
  module_graph: &ModuleGraph,
  mut dependencies: Vec<DependencyId>,
  additional_dependency: Option<DependencyId>,
) -> HashSet<JsWord> {
  if let Some(additional_dependency) = additional_dependency {
    dependencies.push(additional_dependency);
  }

  let mut names = HashSet::default();

  for dependency in dependencies.iter() {
    if let Some(module_identifier) = module_graph.module_identifier_by_dependency_id(dependency) {
      let exports_info = module_graph.get_exports_info(module_identifier);
      for export_info_id in exports_info.exports.values() {
        let export_info = module_graph
          .export_info_map
          .get(export_info_id)
          .expect("should have export info");
        // This is safe because a real export can't export empty string
        let export_name = export_info.name.clone().unwrap_or_default();
        if matches!(export_info.provided, Some(ExportInfoProvided::True))
          && &export_name != "default"
          && !names.contains(&export_name)
        {
          names.insert(export_name.clone());
        }
      }
    }
  }

  names
}
