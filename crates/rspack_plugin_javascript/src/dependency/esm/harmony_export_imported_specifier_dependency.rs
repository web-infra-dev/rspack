use std::hash::BuildHasherDefault;
use std::{collections::HashMap, sync::Arc};

use indexmap::IndexSet;
use rspack_core::{
  create_exports_object_referenced, create_no_exports_referenced, export_from_import,
  get_exports_type, get_import_var, process_export_info, property_access, property_name,
  string_of_used_name, AsContextDependency, ConnectionState, Dependency, DependencyCategory,
  DependencyCondition, DependencyId, DependencyTemplate, DependencyType, ExportInfoId,
  ExportInfoProvided, ExportNameOrSpec, ExportSpec, ExportsInfoId, ExportsOfExportsSpec,
  ExportsSpec, ExportsType, ExtendedReferencedExport, HarmonyExportInitFragment, InitFragmentExt,
  InitFragmentKey, InitFragmentStage, ModuleDependency, ModuleGraph, ModuleIdentifier,
  NormalInitFragment, RuntimeGlobals, RuntimeSpec, Template, TemplateContext,
  TemplateReplaceSource, UsageState, UsedName,
};
use rustc_hash::{FxHashSet as HashSet, FxHasher};
use swc_core::ecma::atoms::Atom;

use super::{create_resource_identifier_for_esm_dependency, harmony_import_dependency_apply};

// Create _webpack_require__.d(__webpack_exports__, {}).
// case1: `import { a } from 'a'; export { a }`
// case2: `export { a } from 'a';`
// case3: `export * from 'a'`
#[derive(Debug, Clone)]
pub struct HarmonyExportImportedSpecifierDependency {
  pub id: DependencyId,
  pub source_order: i32,
  pub request: Atom,
  pub ids: Vec<(Atom, Option<Atom>)>,
  /// used for get_mode, legacy issue
  pub mode_ids: Vec<(Atom, Option<Atom>)>,
  pub name: Option<Atom>,
  resource_identifier: String,
  // Because it is shared by multiply HarmonyExportImportedSpecifierDependency, so put it to `BuildInfo`
  // pub active_exports: HashSet<Atom>,
  // pub all_star_exports: Option<Vec<DependencyId>>,
  pub other_star_exports: Option<Vec<DependencyId>>,
  pub export_all: bool,
}

impl HarmonyExportImportedSpecifierDependency {
  pub fn new(
    request: Atom,
    source_order: i32,
    ids: Vec<(Atom, Option<Atom>)>,
    mode_ids: Vec<(Atom, Option<Atom>)>,
    name: Option<Atom>,
    export_all: bool,
    other_star_exports: Option<Vec<DependencyId>>,
  ) -> Self {
    let resource_identifier = create_resource_identifier_for_esm_dependency(&request);
    Self {
      id: DependencyId::new(),
      source_order,
      mode_ids,
      name,
      request,
      ids,
      resource_identifier,
      export_all,
      other_star_exports,
    }
  }

  pub fn active_exports<'a>(&self, module_graph: &'a ModuleGraph) -> &'a HashSet<Atom> {
    let build_info = module_graph
      .parent_module_by_dependency_id(&self.id)
      .and_then(|ident| module_graph.module_by_identifier(&ident))
      .expect("should have mgm")
      .build_info()
      .expect("should have build info");
    &build_info.harmony_named_exports
  }

  pub fn all_star_exports<'a>(&self, module_graph: &'a ModuleGraph) -> &'a Vec<DependencyId> {
    let build_info = module_graph
      .parent_module_by_dependency_id(&self.id)
      .and_then(|ident| module_graph.module_by_identifier(&ident))
      .expect("should have mgm")
      .build_info()
      .expect("should have build info");
    &build_info.all_star_exports
  }

  // TODO cache get_mode result
  #[allow(unused)]
  pub fn get_mode(
    &self,
    name: Option<Atom>,
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
    let exports_info = module_graph.get_exports_info(&parent_module);

    let is_name_unused = if let Some(ref name) = name {
      exports_info.get_used(UsedName::Str(name.clone()), runtime, module_graph)
        == UsageState::Unused
    } else {
      !exports_info.is_used(runtime, module_graph)
    };
    if is_name_unused {
      let mut mode = ExportMode::new(ExportModeType::Unused);
      mode.name = Some("*".into());
      return mode;
    }
    let imported_exports_type = get_exports_type(module_graph, id, &parent_module);
    let ids = self.get_ids(module_graph);

    // Special handling for reexporting the default export
    // from non-namespace modules
    if let Some(name) = name.as_ref()
      && ids.first().map(|item| item.as_ref()) == Some("default")
    {
      match imported_exports_type {
        ExportsType::Dynamic => {
          let mut export_mode = ExportMode::new(ExportModeType::ReexportDynamicDefault);
          export_mode.name = Some(name.clone());
          return export_mode;
        }
        ExportsType::DefaultOnly | ExportsType::DefaultWithNamed => {
          let export_info_id = exports_info
            .id
            .get_read_only_export_info(name, module_graph)
            .id;
          let mut export_mode = ExportMode::new(ExportModeType::ReexportNamedDefault);
          export_mode.name = Some(name.clone());
          export_mode.partial_namespace_export_info = Some(export_info_id);
          return export_mode;
        }
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
        match imported_exports_type {
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
        match imported_exports_type {
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
    } = self.get_star_reexports(
      module_graph,
      runtime,
      Some(exports_info.id),
      imported_module_identifier,
    );
    // dbg!(
    //   self.request(),
    //   &exports,
    //   &imported_module_identifier,
    //   &checked,
    //   &hidden
    // );
    if let Some(exports) = exports {
      if exports.is_empty() {
        let mut export_mode = ExportMode::new(ExportModeType::EmptyStar);
        export_mode.hidden = hidden;
        return export_mode;
      }
      // dbg!(&exports, &checked);

      let mut items = exports
        .into_iter()
        .map(|export_name| NormalReexportItem {
          name: export_name.clone(),
          ids: vec![export_name.clone()],
          hidden: false,
          checked: checked
            .as_ref()
            .map(|c| c.contains(&export_name))
            .unwrap_or_default(),
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
    exports_info_id: Option<ExportsInfoId>,
    imported_module_identifier: &ModuleIdentifier,
  ) -> StarReexportsInfo {
    let exports_info = exports_info_id
      .unwrap_or_else(|| {
        // https://github.com/webpack/webpack/blob/ac7e531436b0d47cd88451f497cdfd0dad41535d/lib/dependencies/HarmonyExportImportedSpecifierDependency.js#L425
        let parent_module = module_graph
          .parent_module_by_dependency_id(&self.id)
          .expect("should have parent module");
        module_graph.get_exports_info(&parent_module).id
      })
      .get_exports_info(module_graph);

    let imported_exports_info = module_graph.get_exports_info(imported_module_identifier);
    // dbg!(&imported_exports_info);
    let other_export_info_of_imported =
      module_graph.get_export_info_by_id(&imported_exports_info.other_exports_info);

    let other_exports_info_of_exports_info =
      module_graph.get_export_info_by_id(&exports_info.other_exports_info);

    let no_extra_exports = matches!(
      other_export_info_of_imported.provided,
      Some(ExportInfoProvided::False)
    );

    let no_extra_imports = matches!(
      other_exports_info_of_exports_info.get_used(runtime),
      UsageState::Unused
    );

    let ignored_exports: HashSet<Atom> = {
      let mut e = self.active_exports(module_graph).clone();
      e.insert("default".into());
      e
    };

    let hidden_exports = self
      .discover_active_exports_from_other_star_exports(module_graph)
      .map(|other_star_exports| {
        let mut hide_exports = HashSet::default();
        for i in 0..other_star_exports.names_slice {
          hide_exports.insert(other_star_exports.names[i].clone());
        }
        for e in ignored_exports.iter() {
          hide_exports.remove(e);
        }
        hide_exports
      });
    if !no_extra_exports && !no_extra_imports {
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

    if no_extra_imports {
      for export_info_id in exports_info.get_ordered_exports() {
        let export_info = module_graph.get_export_info_by_id(export_info_id);
        let export_name = export_info.name.clone().unwrap_or_default();
        if ignored_exports.contains(&export_name)
          || matches!(export_info.get_used(runtime), UsageState::Unused)
        {
          continue;
        }

        let imported_export_info = imported_exports_info
          .id
          .get_read_only_export_info(&export_name, module_graph);
        if matches!(
          imported_export_info.provided,
          Some(ExportInfoProvided::False)
        ) {
          continue;
        }

        if hidden_exports
          .as_ref()
          .map(|hidden_exports| hidden_exports.contains(&export_name))
          == Some(true)
        {
          hidden.as_mut().expect("According previous condition if hidden_exports is `Some`, hidden must be `Some(HashSet)").insert(export_name.clone());
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
      for imported_export_info_id in imported_exports_info.get_ordered_exports() {
        let imported_export_info = module_graph.get_export_info_by_id(imported_export_info_id);
        let imported_export_info_name = imported_export_info.name.clone().unwrap_or_default();
        if ignored_exports.contains(&imported_export_info_name)
          || matches!(
            imported_export_info.provided,
            Some(ExportInfoProvided::False)
          )
        {
          continue;
        }
        let export_info = exports_info
          .id
          .get_read_only_export_info(&imported_export_info_name, module_graph);
        if matches!(export_info.get_used(runtime), UsageState::Unused) {
          continue;
        }
        if let Some(hidden) = hidden.as_mut()
          && hidden_exports
            .as_ref()
            .map(|hidden_exports| hidden_exports.contains(&imported_export_info_name))
            == Some(true)
        {
          hidden.insert(imported_export_info_name.clone());
          continue;
        }

        exports.insert(imported_export_info_name.clone());
        if matches!(
          imported_export_info.provided,
          Some(ExportInfoProvided::True)
        ) {
          continue;
        }
        checked.insert(imported_export_info_name);
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
  ) -> Option<DiscoverActiveExportsFromOtherStarExportsRet> {
    if let Some(other_star_exports) = &self.other_star_exports {
      if other_star_exports.is_empty() {
        return None;
      }
    } else {
      return None;
    }
    let i = self.other_star_exports.as_ref()?.len();

    let all_star_exports = self.all_star_exports(module_graph);
    if !all_star_exports.is_empty() {
      let (names, dependency_indices) =
        determine_export_assignments(module_graph, all_star_exports.clone(), None);

      return Some(DiscoverActiveExportsFromOtherStarExportsRet {
        names,
        names_slice: dependency_indices[i - 1],
        dependency_indices,
        dependency_index: i,
      });
    }

    if let Some(other_star_exports) = &self.other_star_exports {
      let (names, dependency_indices) =
        determine_export_assignments(module_graph, other_star_exports.clone(), Some(self.id));
      return Some(DiscoverActiveExportsFromOtherStarExportsRet {
        names,
        names_slice: dependency_indices[i - 1],
        dependency_indices,
        dependency_index: i,
      });
    }
    None
  }

  fn add_export_fragments(&self, ctxt: &mut TemplateContext, mut mode: ExportMode) {
    let TemplateContext {
      compilation,
      module,
      runtime_requirements,
      ..
    } = ctxt;
    let mut fragments = vec![];
    let mg = &compilation.get_module_graph();
    let imported_module = mg
      .module_identifier_by_dependency_id(&self.id)
      .expect("should have imported module identifier");
    let module_identifier = module.identifier();
    let import_var = get_import_var(mg, self.id);
    match mode.ty {
      ExportModeType::Missing | ExportModeType::EmptyStar => {
        fragments.push(
          NormalInitFragment::new(
            "/* empty/unused harmony star reexport */\n".to_string(),
            InitFragmentStage::StageHarmonyExports,
            1,
            InitFragmentKey::unique(),
            None,
          )
          .boxed(),
        );
      }
      ExportModeType::Unused => fragments.push(
        NormalInitFragment::new(
          Template::to_comment(&format!(
            "unused harmony reexport {}",
            mode.name.unwrap_or_default()
          )),
          InitFragmentStage::StageHarmonyExports,
          1,
          InitFragmentKey::unique(),
          None,
        )
        .boxed(),
      ),
      ExportModeType::ReexportDynamicDefault => {
        let used_name = mg.get_exports_info(&module.identifier()).id.get_used_name(
          mg,
          None,
          UsedName::Str(mode.name.expect("should have name")),
        );
        let key = string_of_used_name(used_name.as_ref());

        let init_fragment = self
          .get_reexport_fragment(
            ctxt,
            "reexport default from dynamic".to_string(),
            key,
            &import_var,
            ValueKey::Null,
          )
          .boxed();
        fragments.push(init_fragment);
      }
      ExportModeType::ReexportNamedDefault => {
        let used_name = mg.get_exports_info(&module.identifier()).id.get_used_name(
          mg,
          None,
          UsedName::Str(mode.name.expect("should have name")),
        );
        let key = string_of_used_name(used_name.as_ref());
        let init_fragment = self
          .get_reexport_fragment(
            ctxt,
            "reexport default export from named module".to_string(),
            key,
            &import_var,
            ValueKey::Str("".into()),
          )
          .boxed();
        fragments.push(init_fragment);
      }
      ExportModeType::ReexportNamespaceObject => {
        let used_name = mg.get_exports_info(&module.identifier()).id.get_used_name(
          mg,
          None,
          UsedName::Str(mode.name.expect("should have name")),
        );
        let key = string_of_used_name(used_name.as_ref());

        let init_fragment = self
          .get_reexport_fragment(
            ctxt,
            "reexport module object".to_string(),
            key,
            &import_var,
            ValueKey::Str("".into()),
          )
          .boxed();
        fragments.push(init_fragment);
      }
      ExportModeType::ReexportFakeNamespaceObject => {
        // TODO: reexport fake namespace object
        let used_name = mg.get_exports_info(&module.identifier()).id.get_used_name(
          mg,
          None,
          UsedName::Str(mode.name.expect("should have name")),
        );
        let key = string_of_used_name(used_name.as_ref());
        self.get_reexport_fake_namespace_object_fragments(ctxt, key, &import_var, mode.fake_type);
      }
      ExportModeType::ReexportUndefined => {
        let used_name = mg.get_exports_info(&module.identifier()).id.get_used_name(
          mg,
          None,
          UsedName::Str(mode.name.expect("should have name")),
        );
        let key = string_of_used_name(used_name.as_ref());

        let init_fragment = self
          .get_reexport_fragment(
            ctxt,
            "reexport non-default export from non-harmony".to_string(),
            key,
            "undefined",
            ValueKey::Str("".into()),
          )
          .boxed();
        fragments.push(init_fragment);
      }
      ExportModeType::NormalReexport => {
        for item in mode.items.into_iter().flatten() {
          let NormalReexportItem {
            name,
            ids,
            hidden,
            checked,
            export_info: _,
          } = item;

          if hidden {
            continue;
          }

          let used_name = mg.get_exports_info(&module_identifier).id.get_used_name(
            mg,
            None,
            UsedName::Str(name.clone()),
          );
          let key = string_of_used_name(used_name.as_ref());

          if checked {
            let is_async = mg.is_async(&module_identifier).unwrap_or_default();
            let stmt = self.get_conditional_reexport_statement(
              ctxt,
              name,
              &import_var,
              ids[0].clone(),
              ValueKey::Vec(ids),
            );
            fragments.push(Box::new(NormalInitFragment::new(
              stmt,
              if is_async {
                InitFragmentStage::StageAsyncHarmonyImports
              } else {
                InitFragmentStage::StageHarmonyImports
              },
              self.source_order,
              InitFragmentKey::unique(),
              None,
            )));
          } else {
            let used_name =
              mg.get_exports_info(imported_module)
                .id
                .get_used_name(mg, None, UsedName::Vec(ids));
            let init_fragment = self
              .get_reexport_fragment(
                ctxt,
                "reexport safe".to_string(),
                key,
                &import_var,
                used_name.into(),
              )
              .boxed();
            fragments.push(init_fragment);
          }
        }
      }
      ExportModeType::DynamicReexport => {
        let ignored = match (mode.hidden.take(), mode.ignored.take()) {
          (None, None) => HashSet::default(),
          (None, Some(ignored)) => ignored,
          (Some(hidden), None) => hidden,
          (Some(hidden), Some(ignore)) => hidden.union(&ignore).cloned().collect(),
        };
        // TODO: modern, need runtimeTemplate support https://github.com/webpack/webpack/blob/1f99ad6367f2b8a6ef17cce0e058f7a67fb7db18/lib/dependencies/HarmonyExportImportedSpecifierDependency.js#L1104-L1106
        let mut content = format!(
          r"
/* harmony reexport (unknown) */ var __WEBPACK_REEXPORT_OBJECT__ = {{}};
/* harmony reexport (unknown) */ for( var __WEBPACK_IMPORT_KEY__ in {import_var}) "
        );

        if ignored.len() > 1 {
          content += &format!(
            "if({}.indexOf(__WEBPACK_IMPORT_KEY__) < 0) ",
            serde_json::to_string(&ignored).expect("should serialize to array")
          );
        } else if let Some(item) = ignored.iter().next() {
          content += &format!(
            "if(__WEBPACK_IMPORT_KEY__ !== {}) ",
            serde_json::to_string(item).expect("should serialize to string")
          );
        }
        content += "__WEBPACK_REEXPORT_OBJECT__[__WEBPACK_IMPORT_KEY__] =";
        // TODO should decide if `modern` is true
        content +=
          &format!("function(key) {{ return {import_var}[key]; }}.bind(0, __WEBPACK_IMPORT_KEY__)");
        runtime_requirements.insert(RuntimeGlobals::EXPORTS);
        runtime_requirements.insert(RuntimeGlobals::DEFINE_PROPERTY_GETTERS);

        let module = compilation
          .get_module_graph()
          .module_by_identifier(&module.identifier())
          .expect("should have module graph module");
        let exports_name = module.get_exports_argument();
        let is_async = compilation
          .get_module_graph()
          .is_async(&module.identifier())
          .unwrap_or_default();
        fragments.push(
          NormalInitFragment::new(
            format!(
              "{content}\n/* harmony reexport (unknown) */ {}({}, __WEBPACK_REEXPORT_OBJECT__);\n",
              RuntimeGlobals::DEFINE_PROPERTY_GETTERS,
              exports_name
            ),
            if is_async {
              InitFragmentStage::StageAsyncHarmonyImports
            } else {
              InitFragmentStage::StageHarmonyImports
            },
            self.source_order,
            InitFragmentKey::unique(),
            None,
          )
          .boxed(),
        );
      }
    }
    ctxt.init_fragments.extend(fragments);
  }

  fn get_reexport_fragment(
    &self,
    ctxt: &mut TemplateContext,
    comment: String,
    key: String,
    name: &str,
    value_key: ValueKey,
  ) -> HarmonyExportInitFragment {
    let TemplateContext {
      runtime_requirements,
      module,
      compilation,
      ..
    } = ctxt;
    let return_value = Self::get_return_value(name.to_owned(), value_key);
    runtime_requirements.insert(RuntimeGlobals::EXPORTS);
    runtime_requirements.insert(RuntimeGlobals::DEFINE_PROPERTY_GETTERS);
    let mut export_map = vec![];
    export_map.push((
      key.into(),
      format!("/* {} */ {}", comment, return_value).into(),
    ));
    let module = compilation
      .get_module_graph()
      .module_by_identifier(&module.identifier())
      .expect("should have module graph module");
    HarmonyExportInitFragment::new(module.get_exports_argument(), export_map)
  }

  fn get_reexport_fake_namespace_object_fragments(
    &self,
    ctxt: &mut TemplateContext,
    key: String,
    name: &str,
    fake_type: u8,
  ) {
    let TemplateContext {
      runtime_requirements,
      module,
      compilation,
      ..
    } = ctxt;

    let module = compilation
      .get_module_graph()
      .module_by_identifier(&module.identifier())
      .expect("should have module graph module");
    runtime_requirements.insert(RuntimeGlobals::EXPORTS);
    runtime_requirements.insert(RuntimeGlobals::DEFINE_PROPERTY_GETTERS);
    runtime_requirements.insert(RuntimeGlobals::CREATE_FAKE_NAMESPACE_OBJECT);
    let mut export_map = vec![];
    let value = format!(
      r"/* reexport fake namespace object from non-harmony */ {name}_namespace_cache || ({name}_namespace_cache = {}({name}{}))",
      RuntimeGlobals::CREATE_FAKE_NAMESPACE_OBJECT,
      if fake_type == 0 {
        "".to_string()
      } else {
        format!(", {fake_type}")
      }
    );
    export_map.push((key.into(), value.into()));
    let frags = vec![
      {
        let name = format!("var {name}_namespace_cache;\n");
        NormalInitFragment::new(
          name.clone(),
          InitFragmentStage::StageConstants,
          -1,
          InitFragmentKey::HarmonyFakeNamespaceObjectFragment(name),
          None,
        )
        .boxed()
      },
      HarmonyExportInitFragment::new(module.get_exports_argument(), export_map).boxed(),
    ];
    ctxt.init_fragments.extend_from_slice(&frags);
  }

  fn get_return_value(name: String, value_key: ValueKey) -> String {
    match value_key {
      ValueKey::False => "/* unused export */ undefined".to_string(),
      ValueKey::Null => format!("{}_default.a", name),
      ValueKey::Str(str) if str.is_empty() => name,
      ValueKey::Str(str) => format!("{}{}", name, property_access(vec![str], 0)),
      ValueKey::Vec(value_key) => format!("{}{}", name, property_access(value_key, 0)),
    }
  }

  fn get_conditional_reexport_statement(
    &self,
    ctxt: &mut TemplateContext<'_, '_, '_>,
    key: Atom,
    name: &String,
    first_value_key: Atom,
    value_key: ValueKey,
  ) -> String {
    if matches!(value_key, ValueKey::False) {
      return "/* unused export */\n".to_string();
    }
    let TemplateContext {
      compilation,
      module,
      runtime_requirements,
      ..
    } = ctxt;
    let return_value = Self::get_return_value(name.to_string(), value_key);
    let module = compilation
      .get_module_graph()
      .module_by_identifier(&module.identifier())
      .expect("should have mgm");
    let exports_name = module.get_exports_argument();
    runtime_requirements.insert(RuntimeGlobals::EXPORTS);
    runtime_requirements.insert(RuntimeGlobals::DEFINE_PROPERTY_GETTERS);
    runtime_requirements.insert(RuntimeGlobals::HAS_OWN_PROPERTY);
    format!(
      "if({}({}, {})) {}({}, {{ {}: function() {{ return {}; }} }});\n",
      RuntimeGlobals::HAS_OWN_PROPERTY,
      name,
      serde_json::to_string(&first_value_key.to_string()).expect("should serialize to string"),
      RuntimeGlobals::DEFINE_PROPERTY_GETTERS,
      exports_name,
      property_name(&key).expect("should have property_name"),
      return_value
    )
  }
}

#[derive(Debug)]
pub struct DiscoverActiveExportsFromOtherStarExportsRet {
  names: Vec<Atom>,
  names_slice: usize,
  pub dependency_indices: Vec<usize>,
  pub dependency_index: usize,
}

impl DependencyTemplate for HarmonyExportImportedSpecifierDependency {
  fn apply(
    &self,
    _source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let TemplateContext {
      compilation,
      module,
      runtime,
      concatenation_scope,
      ..
    } = code_generatable_context;

    let mode = self.get_mode(
      self.name.clone(),
      compilation.get_module_graph(),
      &self.id,
      *runtime,
    );

    if let Some(ref mut scope) = concatenation_scope {
      if matches!(mode.ty, ExportModeType::ReexportUndefined) {
        scope.register_raw_export(
          mode.name.clone().expect("should have name"),
          String::from("/* reexport non-default export from non-harmony */ undefined"),
        );
      }
      return;
    }

    let module = compilation
      .get_module_graph()
      .module_by_identifier(&module.identifier())
      .expect("should have module graph module");

    let import_var = get_import_var(compilation.get_module_graph(), self.id);
    let is_new_treeshaking = compilation.options.is_new_tree_shaking();

    let mut used_exports = if is_new_treeshaking {
      let exports_info_id = compilation
        .get_module_graph()
        .get_exports_info(&module.identifier())
        .id;
      let res = self
        .ids
        .iter()
        .filter_map(|(local, _)| {
          exports_info_id
            .get_used_name(
              compilation.get_module_graph(),
              *runtime,
              UsedName::Str(local.clone()),
            )
            .map(|item| match item {
              UsedName::Str(str) => (local.clone(), vec![str]),
              UsedName::Vec(strs) => (local.clone(), strs),
            })
        })
        .collect::<HashMap<_, _>>();
      Some(res)
    } else if compilation.options.builtins.tree_shaking.is_true() {
      Some(
        compilation
          .get_module_graph()
          .get_exports_info(&module.identifier())
          .old_get_used_exports()
          .into_iter()
          .map(|item| (item.clone(), vec![item]))
          .collect::<HashMap<_, _>>(),
      )
    } else {
      None
    };

    if is_new_treeshaking {
      // dbg!(&mode, self.request());
      if !matches!(mode.ty, ExportModeType::Unused | ExportModeType::EmptyStar) {
        harmony_import_dependency_apply(self, self.source_order, code_generatable_context, &[]);
        self.add_export_fragments(code_generatable_context, mode);
      }
      return;
    }

    let mut exports = vec![];
    for id in &self.ids {
      if let Some(used_exports) = used_exports.as_mut() {
        let Some(item) = used_exports.remove(&id.0) else {
          continue;
        };
        // in webpack, id.0 is local binding and it doesn't always equal to used name, because it
        // maybe mangled
        let key = if is_new_treeshaking {
          item[0].clone()
        } else {
          id.0.clone()
        };
        // __webpack_require__.d({
        //  'key' / *key maybe mangled**/: ${export_expr} /**value*/
        // })
        exports.push((
          key,
          Atom::from(export_from_import(
            code_generatable_context,
            true,
            &self.request,
            &import_var,
            id.1.clone().map(|i| vec![i]).unwrap_or_default(),
            &self.id,
            false,
            false,
          )),
        ));
      } else {
        exports.push((
          id.0.clone(),
          Atom::from(export_from_import(
            code_generatable_context,
            true,
            &self.request,
            &import_var,
            id.1.clone().map(|i| vec![i]).unwrap_or_default(),
            &self.id,
            false,
            false,
          )),
        ));
      }
    }

    if !exports.is_empty() {
      code_generatable_context
        .init_fragments
        .push(Box::new(HarmonyExportInitFragment::new(
          module.get_exports_argument(),
          exports,
        )));
    }
  }

  fn dependency_id(&self) -> Option<DependencyId> {
    Some(self.id)
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

  #[allow(clippy::unwrap_in_result)]
  fn get_exports(&self, mg: &ModuleGraph) -> Option<ExportsSpec> {
    let mode = self.get_mode(self.name.clone(), mg, &self.id, None);
    // dbg!(&self.request(), &mode);
    match mode.ty {
      ExportModeType::Missing => None,
      ExportModeType::Unused => {
        // https://github.com/webpack/webpack/blob/ac7e531436b0d47cd88451f497cdfd0dad41535d/lib/dependencies/HarmonyExportImportedSpecifierDependency.js#L630-L742
        unreachable!()
      }
      ExportModeType::EmptyStar => Some(ExportsSpec {
        exports: ExportsOfExportsSpec::Array(vec![]),
        hide_export: mode
          .hidden
          .clone()
          .map(|item| item.into_iter().collect::<Vec<_>>()),
        dependencies: Some(vec![*mg
          .module_identifier_by_dependency_id(self.id())
          .expect("should have module")]),
        ..Default::default()
      }),
      ExportModeType::ReexportDynamicDefault => {
        let from = mg.connection_by_dependency(self.id());
        Some(ExportsSpec {
          exports: ExportsOfExportsSpec::Array(vec![ExportNameOrSpec::ExportSpec(ExportSpec {
            name: mode.name.unwrap_or_default(),
            export: Some(rspack_core::Nullable::Value(vec![Atom::from("default")])),
            from: from.cloned(),
            ..Default::default()
          })]),
          priority: Some(1),
          dependencies: Some(vec![from.expect("should have module").module_identifier]),
          ..Default::default()
        })
      }
      ExportModeType::ReexportNamedDefault => {
        let from = mg.connection_by_dependency(self.id());
        Some(ExportsSpec {
          exports: ExportsOfExportsSpec::Array(vec![ExportNameOrSpec::ExportSpec(ExportSpec {
            name: mode.name.unwrap_or_default(),
            export: Some(rspack_core::Nullable::Value(vec![Atom::from("default")])),
            from: from.cloned(),
            ..Default::default()
          })]),
          priority: Some(1),
          dependencies: Some(vec![from.expect("should have module").module_identifier]),
          ..Default::default()
        })
      }
      ExportModeType::ReexportNamespaceObject => {
        let from = mg.connection_by_dependency(self.id());
        Some(ExportsSpec {
          exports: ExportsOfExportsSpec::Array(vec![ExportNameOrSpec::ExportSpec(ExportSpec {
            name: mode.name.unwrap_or_default(),
            export: Some(rspack_core::Nullable::Null),
            from: from.cloned(),
            ..Default::default()
          })]),
          priority: Some(1),
          dependencies: Some(vec![from.expect("should have module").module_identifier]),
          ..Default::default()
        })
      }
      ExportModeType::ReexportFakeNamespaceObject => {
        let from = mg.connection_by_dependency(self.id());
        Some(ExportsSpec {
          exports: ExportsOfExportsSpec::Array(vec![ExportNameOrSpec::ExportSpec(ExportSpec {
            name: mode.name.unwrap_or_default(),
            export: Some(rspack_core::Nullable::Null),
            exports: Some(vec![ExportNameOrSpec::ExportSpec(ExportSpec {
              name: "default".into(),
              can_mangle: Some(false),
              from: from.cloned(),
              export: Some(rspack_core::Nullable::Null),
              ..Default::default()
            })]),
            from: from.cloned(),
            ..Default::default()
          })]),
          priority: Some(1),
          dependencies: Some(vec![from.expect("should have module").module_identifier]),
          ..Default::default()
        })
      }
      ExportModeType::ReexportUndefined => Some(ExportsSpec {
        exports: ExportsOfExportsSpec::Array(vec![ExportNameOrSpec::String(
          mode.name.unwrap_or_default(),
        )]),
        dependencies: Some(vec![*mg
          .module_identifier_by_dependency_id(self.id())
          .expect("should have module id")]),
        ..Default::default()
      }),
      ExportModeType::NormalReexport => {
        let from = mg.connection_by_dependency(self.id());
        Some(ExportsSpec {
          priority: Some(1),
          exports: ExportsOfExportsSpec::Array(
            mode
              .items
              .map(|items| {
                items
                  .into_iter()
                  .map(|item| {
                    ExportNameOrSpec::ExportSpec(ExportSpec {
                      name: item.name,
                      from: from.cloned(),
                      export: Some(rspack_core::Nullable::Value(item.ids)),
                      hidden: Some(item.hidden),
                      ..Default::default()
                    })
                  })
                  .collect::<Vec<_>>()
              })
              .unwrap_or_default(),
          ),
          dependencies: Some(vec![from.expect("should have module").module_identifier]),
          ..Default::default()
        })
      }
      ExportModeType::DynamicReexport => {
        let from = mg.connection_by_dependency(self.id());
        Some(ExportsSpec {
          exports: ExportsOfExportsSpec::True,
          from: from.cloned(),
          can_mangle: Some(false),
          hide_export: Some(
            mode
              .hidden
              .clone()
              .into_iter()
              .flatten()
              .collect::<Vec<_>>(),
          ),
          exclude_exports: if let Some(hidden) = mode.hidden {
            Some(
              hidden
                .into_iter()
                .chain(mode.ignored.into_iter().flatten())
                .collect::<Vec<_>>(),
            )
          } else {
            Some(mode.ignored.into_iter().flatten().collect::<Vec<_>>())
          },
          dependencies: Some(vec![from.expect("should have module").module_identifier]),
          ..Default::default()
        })
      }
    }
  }

  fn get_module_evaluation_side_effects_state(
    &self,
    _module_graph: &ModuleGraph,
    _module_chain: &mut HashSet<ModuleIdentifier>,
  ) -> ConnectionState {
    ConnectionState::Bool(false)
  }

  fn get_ids(&self, mg: &ModuleGraph) -> Vec<Atom> {
    mg.get_dep_meta_if_existing(self.id)
      .map(|meta| meta.ids.clone())
      .unwrap_or_else(|| {
        self
          .mode_ids
          .iter()
          .map(|(id, orig)| orig.clone().unwrap_or(id.clone()))
          .collect()
      })
  }

  fn dependency_debug_name(&self) -> &'static str {
    "HarmonyExportImportedSpecifierDependency"
  }

  fn resource_identifier(&self) -> Option<&str> {
    Some(&self.resource_identifier)
  }

  fn source_order(&self) -> Option<i32> {
    Some(self.source_order)
  }
}

impl ModuleDependency for HarmonyExportImportedSpecifierDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.request
  }

  fn set_request(&mut self, request: String) {
    self.request = request.into();
  }

  fn is_export_all(&self) -> Option<bool> {
    if self.export_all {
      Some(true)
    } else {
      None
    }
  }

  fn get_condition(&self) -> Option<DependencyCondition> {
    let id = self.id;
    Some(DependencyCondition::Fn(Arc::new(
      move |_mc, runtime, module_graph: &ModuleGraph| {
        let dep = module_graph
          .dependency_by_id(&id)
          .expect("should have dependency");
        let down_casted_dep = dep
          .downcast_ref::<HarmonyExportImportedSpecifierDependency>()
          .expect("should be HarmonyExportImportedSpecifierDependency");
        let mode = down_casted_dep.get_mode(
          down_casted_dep.name.clone(),
          module_graph,
          &down_casted_dep.id,
          runtime,
        );
        ConnectionState::Bool(!matches!(
          mode.ty,
          ExportModeType::Unused | ExportModeType::EmptyStar
        ))
      },
    )))
  }

  fn get_referenced_exports(
    &self,
    module_graph: &ModuleGraph,
    runtime: Option<&RuntimeSpec>,
  ) -> Vec<ExtendedReferencedExport> {
    let mode = self.get_mode(self.name.clone(), module_graph, &self.id, runtime);
    // dbg!(&mode);
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

enum ValueKey {
  False,
  Null,
  Str(Atom),
  Vec(Vec<Atom>),
}

impl From<Option<UsedName>> for ValueKey {
  fn from(value: Option<UsedName>) -> Self {
    match value {
      Some(UsedName::Str(atom)) => Self::Str(atom),
      Some(UsedName::Vec(atoms)) => Self::Vec(atoms),
      None => Self::False,
    }
  }
}

impl AsContextDependency for HarmonyExportImportedSpecifierDependency {}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
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
  pub name: Atom,
  pub ids: Vec<Atom>,
  pub hidden: bool,
  pub checked: bool,
  pub export_info: ExportInfoId,
}

#[derive(Debug)]
pub struct ExportMode {
  /// corresponding to `type` field in webpack's `EpxortMode`
  pub ty: ExportModeType,
  pub items: Option<Vec<NormalReexportItem>>,
  pub name: Option<Atom>,
  pub fake_type: u8,
  pub partial_namespace_export_info: Option<ExportInfoId>,
  pub ignored: Option<HashSet<Atom>>,
  pub hidden: Option<HashSet<Atom>>,
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
  exports: Option<HashSet<Atom>>,
  checked: Option<HashSet<Atom>>,
  ignored_exports: HashSet<Atom>,
  hidden: Option<HashSet<Atom>>,
}

/// return (names, dependency_indices)
fn determine_export_assignments(
  module_graph: &ModuleGraph,
  mut dependencies: Vec<DependencyId>,
  additional_dependency: Option<DependencyId>,
) -> (Vec<Atom>, Vec<usize>) {
  if let Some(additional_dependency) = additional_dependency {
    dependencies.push(additional_dependency);
  }

  // https://github.com/webpack/webpack/blob/ac7e531436b0d47cd88451f497cdfd0dad41535d/lib/dependencies/HarmonyExportImportedSpecifierDependency.js#L109
  // js `Set` keep the insertion order, use `IndexSet` to align there behavior
  let mut names: IndexSet<Atom, BuildHasherDefault<FxHasher>> = IndexSet::default();
  let mut dependency_indices = vec![];

  for dependency in dependencies.iter() {
    dependency_indices.push(names.len());
    if let Some(module_identifier) = module_graph.module_identifier_by_dependency_id(dependency) {
      let exports_info = module_graph.get_exports_info(module_identifier);
      for export_info_id in exports_info.exports.values() {
        let export_info = module_graph.get_export_info_by_id(export_info_id);
        // SAFETY: This is safe because a real export can't export empty string
        let export_info_name = export_info.name.clone().unwrap_or_default();
        if matches!(export_info.provided, Some(ExportInfoProvided::True))
          && &export_info_name != "default"
          && !names.contains(&export_info_name)
        {
          names.insert(export_info_name.clone());
          let cur_len = dependency_indices.len();
          dependency_indices[cur_len - 1] = names.len();
        }
      }
    }
  }

  (names.into_iter().collect::<Vec<Atom>>(), dependency_indices)
}
