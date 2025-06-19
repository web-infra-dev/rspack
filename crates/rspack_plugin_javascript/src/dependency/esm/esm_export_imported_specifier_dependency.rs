use std::hash::BuildHasherDefault;

use indexmap::{IndexMap, IndexSet};
use rspack_cacheable::{
  cacheable, cacheable_dyn,
  with::{AsOption, AsPreset, AsVec, Skip},
};
use rspack_collections::{IdentifierMap, IdentifierSet};
use rspack_core::{
  collect_referenced_export_items, create_exports_object_referenced, create_no_exports_referenced,
  filter_runtime, get_exports_type, get_runtime_key, get_terminal_binding, property_access,
  property_name, to_normal_comment, AsContextDependency, ConditionalInitFragment, ConnectionState,
  Dependency, DependencyCategory, DependencyCodeGeneration, DependencyCondition,
  DependencyConditionFn, DependencyId, DependencyLocation, DependencyRange, DependencyTemplate,
  DependencyTemplateType, DependencyType, DetermineExportAssignmentsKey, ESMExportInitFragment,
  ExportMode, ExportModeDynamicReexport, ExportModeEmptyStar, ExportModeFakeNamespaceObject,
  ExportModeNormalReexport, ExportModeReexportDynamicDefault, ExportModeReexportNamedDefault,
  ExportModeReexportNamespaceObject, ExportModeReexportUndefined, ExportModeUnused,
  ExportNameOrSpec, ExportPresenceMode, ExportProvided, ExportSpec, ExportsInfo, ExportsInfoGetter,
  ExportsOfExportsSpec, ExportsSpec, ExportsType, ExtendedReferencedExport, FactorizeInfo,
  GetUsedNameParam, ImportAttributes, InitFragmentExt, InitFragmentKey, InitFragmentStage,
  JavascriptParserOptions, ModuleDependency, ModuleGraph, ModuleGraphCacheArtifact,
  ModuleIdentifier, NormalInitFragment, NormalReexportItem, PrefetchExportsInfoMode,
  RuntimeCondition, RuntimeGlobals, RuntimeSpec, SharedSourceMap, StarReexportsInfo,
  TemplateContext, TemplateReplaceSource, UsageState, UsedName,
};
use rspack_error::{
  miette::{MietteDiagnostic, Severity},
  Diagnostic, DiagnosticExt, TraceableError,
};
use rustc_hash::{FxHashSet as HashSet, FxHasher};
use swc_core::ecma::atoms::Atom;

use super::{
  create_resource_identifier_for_esm_dependency,
  esm_import_dependency::esm_import_dependency_get_linking_error, esm_import_dependency_apply,
};

// Create _webpack_require__.d(__webpack_exports__, {}).
// case1: `import { a } from 'a'; export { a }`
// case2: `export { a } from 'a';`
// case3: `export * from 'a'`
#[cacheable]
#[derive(Debug, Clone)]
pub struct ESMExportImportedSpecifierDependency {
  pub id: DependencyId,
  #[cacheable(with=AsVec<AsPreset>)]
  ids: Vec<Atom>,
  #[cacheable(with=AsOption<AsPreset>)]
  pub name: Option<Atom>,
  #[cacheable(with=AsPreset)]
  pub request: Atom,
  source_order: i32,
  pub other_star_exports: Option<Vec<DependencyId>>,
  range: DependencyRange,
  attributes: Option<ImportAttributes>,
  resource_identifier: String,
  export_presence_mode: ExportPresenceMode,
  #[cacheable(with=Skip)]
  source_map: Option<SharedSourceMap>,
  factorize_info: FactorizeInfo,
}

impl ESMExportImportedSpecifierDependency {
  #[allow(clippy::too_many_arguments)]
  pub fn new(
    request: Atom,
    source_order: i32,
    ids: Vec<Atom>,
    name: Option<Atom>,
    other_star_exports: Option<Vec<DependencyId>>,
    range: DependencyRange,
    export_presence_mode: ExportPresenceMode,
    attributes: Option<ImportAttributes>,
    source_map: Option<SharedSourceMap>,
  ) -> Self {
    let resource_identifier =
      create_resource_identifier_for_esm_dependency(&request, attributes.as_ref());
    Self {
      id: DependencyId::new(),
      source_order,
      name,
      request,
      ids,
      resource_identifier,
      other_star_exports,
      range,
      export_presence_mode,
      attributes,
      source_map,
      factorize_info: Default::default(),
    }
  }

  // Because it is shared by multiply ESMExportImportedSpecifierDependency, so put it to `BuildInfo`
  pub fn active_exports<'a>(&self, module_graph: &'a ModuleGraph) -> &'a HashSet<Atom> {
    let build_info = module_graph
      .get_parent_module(&self.id)
      .and_then(|ident| module_graph.module_by_identifier(ident))
      .expect("should have mgm")
      .build_info();
    &build_info.esm_named_exports
  }

  // Because it is shared by multiply ESMExportImportedSpecifierDependency, so put it to `BuildInfo`
  pub fn all_star_exports<'a>(
    &self,
    module_graph: &'a ModuleGraph,
  ) -> Option<(ModuleIdentifier, &'a Vec<DependencyId>)> {
    let module = module_graph
      .get_parent_module(&self.id)
      .and_then(|ident| module_graph.module_by_identifier(ident));

    if let Some(module) = module {
      let build_info = module.build_info();
      Some((module.identifier(), &build_info.all_star_exports))
    } else {
      None
    }
  }

  pub fn get_ids<'a>(&'a self, mg: &'a ModuleGraph) -> &'a [Atom] {
    mg.get_dep_meta_if_existing(&self.id)
      .map(|meta| meta.ids.as_slice())
      .unwrap_or_else(|| self.ids.as_slice())
  }

  fn get_mode(
    &self,
    module_graph: &ModuleGraph,
    runtime: Option<&RuntimeSpec>,
    module_graph_cache: &ModuleGraphCacheArtifact,
  ) -> ExportMode {
    let key = (
      self.id,
      runtime.map(|runtime| get_runtime_key(runtime).to_owned()),
    );
    module_graph_cache.cached_get_mode(key, || {
      self.get_mode_inner(module_graph, module_graph_cache, runtime)
    })
  }

  fn get_mode_inner(
    &self,
    module_graph: &ModuleGraph,
    module_graph_cache: &ModuleGraphCacheArtifact,
    runtime: Option<&RuntimeSpec>,
  ) -> ExportMode {
    let id = &self.id;
    let name = self.name.clone();
    let imported_module_identifier = if let Some(imported_module_identifier) =
      module_graph.module_identifier_by_dependency_id(id)
    {
      imported_module_identifier
    } else {
      return ExportMode::Missing;
    };

    let parent_module = module_graph
      .get_parent_module(id)
      .expect("should have parent module");
    let exports_info = module_graph.get_exports_info(parent_module);
    let exports_info_data = ExportsInfoGetter::prefetch(
      &exports_info,
      module_graph,
      PrefetchExportsInfoMode::Default,
    );

    let is_name_unused = if let Some(ref name) = name {
      exports_info_data.get_used(std::slice::from_ref(name), runtime) == UsageState::Unused
    } else {
      !ExportsInfoGetter::prefetch_used_info_without_name(
        &exports_info,
        module_graph,
        runtime,
        false,
      )
      .is_used()
    };
    if is_name_unused {
      return ExportMode::Unused(ExportModeUnused { name: "*".into() });
    }
    let imported_exports_type =
      get_exports_type(module_graph, module_graph_cache, id, parent_module);
    let ids = self.get_ids(module_graph);

    // Special handling for reexporting the default export
    // from non-namespace modules
    if let Some(name) = name.as_ref()
      && ids.first().map(|item| item.as_ref()) == Some("default")
    {
      match imported_exports_type {
        ExportsType::Dynamic => {
          return ExportMode::ReexportDynamicDefault(ExportModeReexportDynamicDefault {
            name: name.clone(),
          });
        }
        ExportsType::DefaultOnly | ExportsType::DefaultWithNamed => {
          return ExportMode::ReexportNamedDefault(ExportModeReexportNamedDefault {
            name: name.clone(),
            partial_namespace_export_info: exports_info_data.get_read_only_export_info(name).id(),
          });
        }
        _ => {}
      }
    }

    // reexporting with a fixed name
    if let Some(name) = name {
      let export_info = exports_info_data.get_read_only_export_info(&name).id();
      if !ids.is_empty() {
        // export { name as name }
        match imported_exports_type {
          ExportsType::DefaultOnly => {
            return ExportMode::ReexportUndefined(ExportModeReexportUndefined { name });
          }
          _ => {
            return ExportMode::NormalReexport(ExportModeNormalReexport {
              items: vec![NormalReexportItem {
                name,
                ids: ids.to_vec(),
                hidden: false,
                checked: false,
                export_info,
              }],
            });
          }
        }
      } else {
        // export * as name
        match imported_exports_type {
          ExportsType::DefaultOnly => {
            return ExportMode::ReexportFakeNamespaceObject(ExportModeFakeNamespaceObject {
              name,
              partial_namespace_export_info: export_info,
              fake_type: 0,
            });
          }
          ExportsType::DefaultWithNamed => {
            return ExportMode::ReexportFakeNamespaceObject(ExportModeFakeNamespaceObject {
              name,
              partial_namespace_export_info: export_info,
              fake_type: 2,
            });
          }
          _ => {
            return ExportMode::ReexportNamespaceObject(ExportModeReexportNamespaceObject {
              name,
              partial_namespace_export_info: export_info,
            });
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
      module_graph_cache,
      runtime,
      Some(exports_info),
      imported_module_identifier,
    );

    if let Some(exports) = exports {
      if exports.is_empty() {
        return ExportMode::EmptyStar(ExportModeEmptyStar { hidden });
      }

      let exports_info_data =
        module_graph.get_prefetched_exports_info(parent_module, PrefetchExportsInfoMode::Default);

      let mut items = exports
        .iter()
        .map(|export_name| NormalReexportItem {
          name: export_name.clone(),
          ids: vec![export_name.clone()],
          hidden: false,
          checked: checked
            .as_ref()
            .map(|c| c.contains(export_name))
            .unwrap_or_default(),
          export_info: exports_info_data
            .get_read_only_export_info(export_name)
            .id(),
        })
        .collect::<Vec<_>>();

      if let Some(hidden) = &hidden {
        for export_name in hidden.iter() {
          items.push(NormalReexportItem {
            name: export_name.clone(),
            ids: vec![export_name.clone()],
            hidden: true,
            checked: false,
            export_info: exports_info_data
              .get_read_only_export_info(export_name)
              .id(),
          });
        }
      }
      ExportMode::NormalReexport(ExportModeNormalReexport { items })
    } else {
      ExportMode::DynamicReexport(Box::new(ExportModeDynamicReexport {
        ignored: ignored_exports,
        hidden,
      }))
    }
  }

  pub fn get_star_reexports(
    &self,
    module_graph: &ModuleGraph,
    module_graph_cache: &ModuleGraphCacheArtifact,
    runtime: Option<&RuntimeSpec>,
    exports_info: Option<ExportsInfo>,
    imported_module_identifier: &ModuleIdentifier,
  ) -> StarReexportsInfo {
    let exports_info = exports_info
      .map(|exports_info_id| {
        ExportsInfoGetter::prefetch(
          &exports_info_id,
          module_graph,
          PrefetchExportsInfoMode::Default,
        )
      })
      .unwrap_or_else(|| {
        // https://github.com/webpack/webpack/blob/ac7e531436b0d47cd88451f497cdfd0dad41535d/lib/dependencies/HarmonyExportImportedSpecifierDependency.js#L425
        module_graph.get_prefetched_exports_info(
          module_graph
            .get_parent_module(&self.id)
            .expect("should have parent module"),
          PrefetchExportsInfoMode::Default,
        )
      });
    let imported_exports_info = module_graph
      .get_prefetched_exports_info(imported_module_identifier, PrefetchExportsInfoMode::Default);

    let no_extra_exports = matches!(
      imported_exports_info.other_exports_info().provided(),
      Some(ExportProvided::NotProvided)
    );
    let no_extra_imports = matches!(
      exports_info.other_exports_info().get_used(runtime),
      UsageState::Unused
    );

    let ignored_exports: HashSet<Atom> = {
      let mut e = self.active_exports(module_graph).clone();
      e.insert("default".into());
      e
    };

    let hidden_exports = self
      .discover_active_exports_from_other_star_exports(module_graph, module_graph_cache)
      .map(|other_star_exports| {
        other_star_exports
          .names
          .into_iter()
          .take(other_star_exports.names_slice)
          .filter(|name| !ignored_exports.contains(name))
          .collect::<HashSet<_>>()
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
      for (_name, export_info) in exports_info.exports() {
        let export_name = export_info.name().cloned().unwrap_or_default();
        if ignored_exports.contains(&export_name)
          || matches!(export_info.get_used(runtime), UsageState::Unused)
        {
          continue;
        }

        let imported_export_info = imported_exports_info.get_read_only_export_info(&export_name);
        if matches!(
          imported_export_info.provided(),
          Some(ExportProvided::NotProvided)
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
          imported_export_info.provided(),
          Some(ExportProvided::Provided)
        ) {
          continue;
        }
        checked.insert(export_name);
      }
    } else if no_extra_exports {
      for (_name, imported_export_info) in imported_exports_info.exports() {
        let imported_export_info_name = imported_export_info.name().cloned().unwrap_or_default();
        if ignored_exports.contains(&imported_export_info_name)
          || matches!(
            imported_export_info.provided(),
            Some(ExportProvided::NotProvided)
          )
        {
          continue;
        }
        let export_info = exports_info.get_read_only_export_info(&imported_export_info_name);
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
          imported_export_info.provided(),
          Some(ExportProvided::Provided)
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
    module_graph_cache: &ModuleGraphCacheArtifact,
  ) -> Option<DiscoverActiveExportsFromOtherStarExportsRet> {
    if let Some(other_star_exports) = &self.other_star_exports {
      if other_star_exports.is_empty() {
        return None;
      }
    } else {
      return None;
    }
    let i = self.other_star_exports.as_ref()?.len();

    if let Some((module_identifier, all_star_exports)) = self.all_star_exports(module_graph)
      && !all_star_exports.is_empty()
    {
      let (names, dependency_indices) = module_graph_cache.cached_determine_export_assignments(
        DetermineExportAssignmentsKey::All(module_identifier),
        || determine_export_assignments(module_graph, all_star_exports, None),
      );

      return Some(DiscoverActiveExportsFromOtherStarExportsRet {
        names,
        names_slice: dependency_indices[i - 1],
        dependency_indices,
        dependency_index: i,
      });
    }

    if let Some(other_star_exports) = &self.other_star_exports {
      let (names, dependency_indices) = module_graph_cache
        .cached_determine_export_assignments(DetermineExportAssignmentsKey::Other(self.id), || {
          determine_export_assignments(module_graph, other_star_exports, Some(self.id))
        });
      return Some(DiscoverActiveExportsFromOtherStarExportsRet {
        names,
        names_slice: dependency_indices[i - 1],
        dependency_indices,
        dependency_index: i,
      });
    }
    None
  }

  pub fn add_export_fragments(&self, ctxt: &mut TemplateContext, mode: ExportMode) {
    let TemplateContext {
      module,
      runtime_requirements,
      ..
    } = ctxt;
    let compilation = ctxt.compilation;
    let mut fragments = vec![];
    let mg = &compilation.get_module_graph();
    let mg_cache = &compilation.module_graph_cache_artifact;
    let module_identifier = module.identifier();
    let import_var = compilation.get_import_var(&self.id);
    match mode {
      ExportMode::Missing | ExportMode::EmptyStar(_) => {
        fragments.push(
          NormalInitFragment::new(
            "/* empty/unused ESM star reexport */\n".to_string(),
            InitFragmentStage::StageESMExports,
            1,
            InitFragmentKey::unique(),
            None,
          )
          .boxed(),
        );
      }
      ExportMode::Unused(ExportModeUnused { name }) => fragments.push(
        NormalInitFragment::new(
          to_normal_comment(&format!("unused ESM reexport {name}")),
          InitFragmentStage::StageESMExports,
          1,
          InitFragmentKey::unique(),
          None,
        )
        .boxed(),
      ),
      ExportMode::ReexportDynamicDefault(ExportModeReexportDynamicDefault { name }) => {
        let exports_info =
          mg.get_prefetched_exports_info(&module_identifier, PrefetchExportsInfoMode::Default);
        let used_name = ExportsInfoGetter::get_used_name(
          GetUsedNameParam::WithNames(&exports_info),
          None,
          std::slice::from_ref(&name),
        );
        let key = render_used_name(used_name.as_ref());

        let init_fragment = self
          .get_reexport_fragment(
            ctxt,
            "reexport default from dynamic",
            key,
            &import_var,
            ValueKey::Null,
          )
          .boxed();
        fragments.push(init_fragment);
      }
      ExportMode::ReexportNamedDefault(mode) => {
        let exports_info =
          mg.get_prefetched_exports_info(&module_identifier, PrefetchExportsInfoMode::Default);
        let used_name = ExportsInfoGetter::get_used_name(
          GetUsedNameParam::WithNames(&exports_info),
          None,
          std::slice::from_ref(&mode.name),
        );
        let key = render_used_name(used_name.as_ref());
        let init_fragment = self
          .get_reexport_fragment(
            ctxt,
            "reexport default export from named module",
            key,
            &import_var,
            ValueKey::Name,
          )
          .boxed();
        fragments.push(init_fragment);
      }
      ExportMode::ReexportNamespaceObject(mode) => {
        let exports_info =
          mg.get_prefetched_exports_info(&module_identifier, PrefetchExportsInfoMode::Default);
        let used_name = ExportsInfoGetter::get_used_name(
          GetUsedNameParam::WithNames(&exports_info),
          None,
          std::slice::from_ref(&mode.name),
        );
        let key = render_used_name(used_name.as_ref());

        let init_fragment = self
          .get_reexport_fragment(
            ctxt,
            "reexport module object",
            key,
            &import_var,
            ValueKey::Name,
          )
          .boxed();
        fragments.push(init_fragment);
      }
      ExportMode::ReexportFakeNamespaceObject(mode) => {
        // TODO: reexport fake namespace object
        let exports_info =
          mg.get_prefetched_exports_info(&module_identifier, PrefetchExportsInfoMode::Default);
        let used_name = ExportsInfoGetter::get_used_name(
          GetUsedNameParam::WithNames(&exports_info),
          None,
          std::slice::from_ref(&mode.name),
        );
        let key = render_used_name(used_name.as_ref());
        self.get_reexport_fake_namespace_object_fragments(ctxt, key, &import_var, mode.fake_type);
      }
      ExportMode::ReexportUndefined(mode) => {
        let exports_info =
          mg.get_prefetched_exports_info(&module_identifier, PrefetchExportsInfoMode::Default);
        let used_name = ExportsInfoGetter::get_used_name(
          GetUsedNameParam::WithNames(&exports_info),
          None,
          std::slice::from_ref(&mode.name),
        );
        let key = render_used_name(used_name.as_ref());

        let init_fragment = self
          .get_reexport_fragment(
            ctxt,
            "reexport non-default export from non-ESM",
            key,
            "undefined",
            ValueKey::Name,
          )
          .boxed();
        fragments.push(init_fragment);
      }
      ExportMode::NormalReexport(mode) => {
        let imported_module = mg
          .module_identifier_by_dependency_id(&self.id)
          .expect("should have imported module identifier");
        let exports_info =
          mg.get_prefetched_exports_info(&module_identifier, PrefetchExportsInfoMode::Default);
        for item in mode.items.into_iter() {
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

          let used_name = ExportsInfoGetter::get_used_name(
            GetUsedNameParam::WithNames(&exports_info),
            None,
            std::slice::from_ref(&name),
          );
          let key = render_used_name(used_name.as_ref());

          if checked {
            let key =
              InitFragmentKey::ESMImport(format!("ESM reexport (checked) {import_var} {name}"));
            let runtime_condition = if self.weak() {
              RuntimeCondition::Boolean(false)
            } else if let Some(connection) = mg.connection_by_dependency_id(self.id()) {
              filter_runtime(ctxt.runtime, |r| {
                connection.is_target_active(mg, r, mg_cache)
              })
            } else {
              RuntimeCondition::Boolean(true)
            };
            let stmt = self.get_conditional_reexport_statement(
              ctxt,
              name,
              &import_var,
              ids[0].clone(),
              ValueKey::UsedName(UsedName::Normal(ids)),
            );
            let is_async = ModuleGraph::is_async(compilation, &module_identifier);
            fragments.push(
              ConditionalInitFragment::new(
                stmt,
                if is_async {
                  InitFragmentStage::StageAsyncESMImports
                } else {
                  InitFragmentStage::StageESMImports
                },
                self.source_order,
                key,
                None,
                runtime_condition,
              )
              .boxed(),
            );
          } else {
            let exports_info = mg.get_exports_info(imported_module);
            let used_name = if ids.is_empty() {
              let exports_info =
                ExportsInfoGetter::prefetch_used_info_without_name(&exports_info, mg, None, false);
              ExportsInfoGetter::get_used_name(
                GetUsedNameParam::WithoutNames(&exports_info),
                None,
                &ids,
              )
            } else {
              let exports_info = ExportsInfoGetter::prefetch(
                &exports_info,
                mg,
                PrefetchExportsInfoMode::Nested(&ids),
              );
              ExportsInfoGetter::get_used_name(
                GetUsedNameParam::WithNames(&exports_info),
                None,
                &ids,
              )
            };
            let init_fragment = self
              .get_reexport_fragment(ctxt, "reexport safe", key, &import_var, used_name.into())
              .boxed();
            fragments.push(init_fragment);
          }
        }
      }
      ExportMode::DynamicReexport(mode) => {
        let mut ignored = mode.ignored;
        if let Some(hidden) = mode.hidden {
          ignored.extend(hidden);
        }

        // TODO: modern, need runtimeTemplate support https://github.com/webpack/webpack/blob/1f99ad6367f2b8a6ef17cce0e058f7a67fb7db18/lib/dependencies/HarmonyExportImportedSpecifierDependency.js#L1104-L1106
        let mut content = format!(
          r"
/* ESM reexport (unknown) */ var __WEBPACK_REEXPORT_OBJECT__ = {{}};
/* ESM reexport (unknown) */ for( var __WEBPACK_IMPORT_KEY__ in {import_var}) "
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

        let module = mg
          .module_by_identifier(&module.identifier())
          .expect("should have module graph module");
        let exports_name = module.get_exports_argument();
        let is_async = ModuleGraph::is_async(compilation, &module.identifier());
        fragments.push(
          NormalInitFragment::new(
            format!(
              "{content}\n/* ESM reexport (unknown) */ {}({}, __WEBPACK_REEXPORT_OBJECT__);\n",
              RuntimeGlobals::DEFINE_PROPERTY_GETTERS,
              exports_name
            ),
            if is_async {
              InitFragmentStage::StageAsyncESMImports
            } else {
              InitFragmentStage::StageESMImports
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
    comment: &str,
    key: String,
    name: &str,
    value_key: ValueKey,
  ) -> ESMExportInitFragment {
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
    export_map.push((key.into(), format!("/* {comment} */ {return_value}").into()));
    let module_graph = compilation.get_module_graph();
    let module = module_graph
      .module_by_identifier(&module.identifier())
      .expect("should have mgm");
    ESMExportInitFragment::new(module.get_exports_argument(), export_map)
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
    let module_graph = compilation.get_module_graph();
    let module = module_graph
      .module_by_identifier(&module.identifier())
      .expect("should have module graph module");
    runtime_requirements.insert(RuntimeGlobals::EXPORTS);
    runtime_requirements.insert(RuntimeGlobals::DEFINE_PROPERTY_GETTERS);
    runtime_requirements.insert(RuntimeGlobals::CREATE_FAKE_NAMESPACE_OBJECT);
    let mut export_map = vec![];
    let value = format!(
      r"/* reexport fake namespace object from non-ESM */ {name}_namespace_cache || ({name}_namespace_cache = {}({name}{}))",
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
          InitFragmentKey::ESMFakeNamespaceObjectFragment(name),
          None,
        )
        .boxed()
      },
      ESMExportInitFragment::new(module.get_exports_argument(), export_map).boxed(),
    ];
    ctxt.init_fragments.extend_from_slice(&frags);
  }

  fn get_return_value(name: String, value_key: ValueKey) -> String {
    match value_key {
      ValueKey::False => "/* unused export */ undefined".to_string(),
      ValueKey::Null => format!("{name}_default.a"),
      ValueKey::Name => name,
      ValueKey::UsedName(used) => match used {
        UsedName::Normal(used) => format!("{}{}", name, property_access(used, 0)),
        UsedName::Inlined(inlined) => inlined.render().into_owned(),
      },
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
    let module_graph = compilation.get_module_graph();
    let module = module_graph
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

  pub fn create_export_presence_mode(options: &JavascriptParserOptions) -> ExportPresenceMode {
    options
      .reexport_exports_presence
      .or(options.exports_presence)
      .unwrap_or(if let Some(true) = options.strict_export_presence {
        ExportPresenceMode::Error
      } else {
        ExportPresenceMode::Auto
      })
  }

  fn get_conflicting_star_exports_errors(
    &self,
    ids: &[Atom],
    module_graph: &ModuleGraph,
    module_graph_cache: &ModuleGraphCacheArtifact,
    should_error: bool,
  ) -> Option<Vec<Diagnostic>> {
    let create_error = |message: String| {
      let (severity, title) = if should_error {
        (Severity::Error, "ESModulesLinkingError")
      } else {
        (Severity::Warning, "ESModulesLinkingWarning")
      };
      let parent_module_identifier = module_graph
        .get_parent_module(&self.id)
        .expect("should have parent module for dependency");
      let mut diagnostic = if let Some(span) = self.range()
        && let Some(parent_module) = module_graph.module_by_identifier(parent_module_identifier)
        && let Some(source) = parent_module.source()
      {
        Diagnostic::from(
          TraceableError::from_file(
            source.source().into_owned(),
            span.start as usize,
            span.end as usize,
            title.to_string(),
            message,
          )
          .with_severity(severity)
          .boxed(),
        )
        .with_hide_stack(Some(true))
      } else {
        Diagnostic::from(
          MietteDiagnostic::new(message)
            .with_code(title)
            .with_severity(severity)
            .boxed(),
        )
        .with_hide_stack(Some(true))
      };
      diagnostic = diagnostic.with_module_identifier(Some(*parent_module_identifier));
      diagnostic
    };

    if ids.is_empty()
      && self.name.is_none()
      && let Some(potential_conflicts) =
        self.discover_active_exports_from_other_star_exports(module_graph, module_graph_cache)
      && potential_conflicts.names_slice > 0
    {
      let own_names = HashSet::from_iter(
        &potential_conflicts.names[potential_conflicts.names_slice
          ..potential_conflicts.dependency_indices[potential_conflicts.dependency_index]],
      );
      let imported_module = module_graph.get_module_by_dependency_id(&self.id)?;
      let exports_info = module_graph.get_prefetched_exports_info(
        &imported_module.identifier(),
        PrefetchExportsInfoMode::Default,
      );
      let mut conflicts: IndexMap<&str, Vec<&Atom>, BuildHasherDefault<FxHasher>> =
        IndexMap::default();
      for (_name, export_info) in exports_info.exports() {
        if !matches!(export_info.provided(), Some(ExportProvided::Provided)) {
          continue;
        }
        let Some(name) = export_info.name() else {
          continue;
        };
        if name == "default" {
          continue;
        }
        if self.active_exports(module_graph).contains(name) {
          continue;
        }
        if own_names.contains(&name) {
          continue;
        }

        let dependencies = if let Some((_, all_star_exports)) = self.all_star_exports(module_graph)
        {
          all_star_exports
            .iter()
            .filter_map(|id| module_graph.dependency_by_id(id))
            .filter_map(|dep| dep.as_module_dependency())
            .collect::<Vec<_>>()
        } else {
          Vec::new()
        };
        let Some(conflicting_dependency) = find_dependency_for_name(
          potential_conflicts.names.iter().enumerate(),
          potential_conflicts.dependency_indices.iter(),
          name,
          dependencies.iter().copied(),
        ) else {
          continue;
        };
        let Some(target) = get_terminal_binding(export_info, module_graph) else {
          continue;
        };
        let Some(conflicting_module) =
          module_graph.get_module_by_dependency_id(conflicting_dependency.id())
        else {
          continue;
        };
        if conflicting_module.identifier() == imported_module.identifier() {
          continue;
        }
        let exports_info = module_graph.get_prefetched_exports_info(
          &conflicting_module.identifier(),
          PrefetchExportsInfoMode::Default,
        );
        let Some(conflicting_export_info) = exports_info.data().named_exports(name) else {
          continue;
        };
        let Some(conflicting_target) = get_terminal_binding(conflicting_export_info, module_graph)
        else {
          continue;
        };
        if target == conflicting_target {
          continue;
        }
        if let Some(list) = conflicts.get_mut(conflicting_dependency.request()) {
          list.push(name);
        } else {
          conflicts.insert(conflicting_dependency.request(), vec![name]);
        }
      }
      if !conflicts.is_empty() {
        return Some(conflicts.iter().map(|(request, exports)| {
          let msg = format!(
            "The requested module '{}' contains conflicting star exports for the {} {} with the previous requested module '{request}'",
            self.request(),
            if exports.len() > 1 { "names" } else { "name" },
            exports.iter().map(|e| format!("'{e}'")).collect::<Vec<_>>().join(", "),
          );
          create_error(msg)
        }).collect());
      }
    }
    None
  }
}

#[derive(Debug)]
pub struct DiscoverActiveExportsFromOtherStarExportsRet {
  names: Vec<Atom>,
  names_slice: usize,
  pub dependency_indices: Vec<usize>,
  pub dependency_index: usize,
}

#[cacheable_dyn]
impl DependencyCodeGeneration for ESMExportImportedSpecifierDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(ESMExportImportedSpecifierDependencyTemplate::template_type())
  }
}

#[cacheable_dyn]
impl Dependency for ESMExportImportedSpecifierDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn loc(&self) -> Option<DependencyLocation> {
    self.range.to_loc(self.source_map.as_ref())
  }

  fn range(&self) -> Option<&DependencyRange> {
    Some(&self.range)
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Esm
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::EsmExportImportedSpecifier
  }

  fn get_attributes(&self) -> Option<&ImportAttributes> {
    self.attributes.as_ref()
  }

  fn get_exports(
    &self,
    mg: &ModuleGraph,
    module_graph_cache: &ModuleGraphCacheArtifact,
  ) -> Option<ExportsSpec> {
    let mode = self.get_mode(mg, None, module_graph_cache);
    match mode {
      ExportMode::Missing => None,
      ExportMode::Unused(_) => {
        // https://github.com/webpack/webpack/blob/ac7e531436b0d47cd88451f497cdfd0dad41535d/lib/dependencies/HarmonyExportImportedSpecifierDependency.js#L630-L742
        unreachable!()
      }
      ExportMode::EmptyStar(mode) => Some(ExportsSpec {
        exports: ExportsOfExportsSpec::Names(vec![]),
        hide_export: mode.hidden,
        dependencies: Some(vec![*mg
          .module_identifier_by_dependency_id(self.id())
          .expect("should have module")]),
        ..Default::default()
      }),
      ExportMode::ReexportDynamicDefault(mode) => {
        let from = mg.connection_by_dependency_id(self.id());
        Some(ExportsSpec {
          exports: ExportsOfExportsSpec::Names(vec![ExportNameOrSpec::ExportSpec(ExportSpec {
            name: mode.name,
            export: Some(rspack_core::Nullable::Value(vec![Atom::from("default")])),
            from: from.cloned(),
            ..Default::default()
          })]),
          priority: Some(1),
          dependencies: Some(vec![*from.expect("should have module").module_identifier()]),
          ..Default::default()
        })
      }
      ExportMode::ReexportNamedDefault(mode) => {
        let from = mg.connection_by_dependency_id(self.id());
        Some(ExportsSpec {
          exports: ExportsOfExportsSpec::Names(vec![ExportNameOrSpec::ExportSpec(ExportSpec {
            name: mode.name,
            export: Some(rspack_core::Nullable::Value(vec![Atom::from("default")])),
            from: from.cloned(),
            ..Default::default()
          })]),
          priority: Some(1),
          dependencies: Some(vec![*from.expect("should have module").module_identifier()]),
          ..Default::default()
        })
      }
      ExportMode::ReexportNamespaceObject(mode) => {
        let from = mg.connection_by_dependency_id(self.id());
        Some(ExportsSpec {
          exports: ExportsOfExportsSpec::Names(vec![ExportNameOrSpec::ExportSpec(ExportSpec {
            name: mode.name,
            export: Some(rspack_core::Nullable::Null),
            from: from.cloned(),
            ..Default::default()
          })]),
          priority: Some(1),
          dependencies: Some(vec![*from.expect("should have module").module_identifier()]),
          ..Default::default()
        })
      }
      ExportMode::ReexportFakeNamespaceObject(mode) => {
        let from = mg.connection_by_dependency_id(self.id());
        Some(ExportsSpec {
          exports: ExportsOfExportsSpec::Names(vec![ExportNameOrSpec::ExportSpec(ExportSpec {
            name: mode.name,
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
          dependencies: Some(vec![*from.expect("should have module").module_identifier()]),
          ..Default::default()
        })
      }
      ExportMode::ReexportUndefined(mode) => Some(ExportsSpec {
        exports: ExportsOfExportsSpec::Names(vec![ExportNameOrSpec::String(mode.name)]),
        dependencies: Some(vec![*mg
          .module_identifier_by_dependency_id(self.id())
          .expect("should have module id")]),
        ..Default::default()
      }),
      ExportMode::NormalReexport(mode) => {
        let from = mg.connection_by_dependency_id(self.id());
        Some(ExportsSpec {
          priority: Some(1),
          exports: ExportsOfExportsSpec::Names(
            mode
              .items
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
              .collect::<Vec<_>>(),
          ),
          dependencies: Some(vec![*from.expect("should have module").module_identifier()]),
          ..Default::default()
        })
      }
      ExportMode::DynamicReexport(mode) => {
        let from = mg.connection_by_dependency_id(self.id());
        Some(ExportsSpec {
          exports: ExportsOfExportsSpec::UnknownExports,
          from: from.cloned(),
          can_mangle: Some(false),
          hide_export: mode.hidden.clone(),
          exclude_exports: {
            let mut exclude_exports = mode.ignored;
            if let Some(hidden) = mode.hidden {
              exclude_exports.extend(hidden);
            }
            Some(exclude_exports)
          },
          dependencies: Some(vec![*from.expect("should have module").module_identifier()]),
          ..Default::default()
        })
      }
    }
  }

  fn get_module_evaluation_side_effects_state(
    &self,
    _module_graph: &ModuleGraph,
    _module_graph_cache: &ModuleGraphCacheArtifact,
    _module_chain: &mut IdentifierSet,
    _connection_state_cache: &mut IdentifierMap<ConnectionState>,
  ) -> ConnectionState {
    ConnectionState::Active(false)
  }

  fn _get_ids<'a>(&'a self, mg: &'a ModuleGraph) -> &'a [Atom] {
    self.get_ids(mg)
  }

  fn resource_identifier(&self) -> Option<&str> {
    Some(&self.resource_identifier)
  }

  fn source_order(&self) -> Option<i32> {
    Some(self.source_order)
  }

  // #[tracing::instrument(skip_all)]
  fn get_diagnostics(
    &self,
    module_graph: &ModuleGraph,
    module_graph_cache: &ModuleGraphCacheArtifact,
  ) -> Option<Vec<Diagnostic>> {
    let module = module_graph.get_parent_module(&self.id)?;
    let module = module_graph.module_by_identifier(module)?;
    let ids = self.get_ids(module_graph);
    if let Some(should_error) = self
      .export_presence_mode
      .get_effective_export_presence(&**module)
    {
      let mut diagnostics = Vec::new();
      // don't need to check the import specifier is existed or not when name is None (export *)
      if let Some(name) = &self.name
        && let Some(error) = esm_import_dependency_get_linking_error(
          self,
          ids,
          module_graph,
          module_graph_cache,
          name,
          true,
          should_error,
        )
      {
        diagnostics.push(error);
      }
      if let Some(errors) = self.get_conflicting_star_exports_errors(
        ids,
        module_graph,
        module_graph_cache,
        should_error,
      ) {
        diagnostics.extend(errors);
      }
      return Some(diagnostics);
    }
    None
  }

  fn get_referenced_exports(
    &self,
    module_graph: &ModuleGraph,
    module_graph_cache: &ModuleGraphCacheArtifact,
    runtime: Option<&RuntimeSpec>,
  ) -> Vec<ExtendedReferencedExport> {
    let mode = self.get_mode(module_graph, runtime, module_graph_cache);
    match mode {
      ExportMode::Missing
      | ExportMode::Unused(_)
      | ExportMode::EmptyStar(_)
      | ExportMode::ReexportUndefined(_) => create_no_exports_referenced(),
      ExportMode::ReexportDynamicDefault(_) | ExportMode::DynamicReexport(_) => {
        create_exports_object_referenced()
      }
      ExportMode::ReexportNamedDefault(ExportModeReexportNamedDefault {
        ref partial_namespace_export_info,
        ..
      })
      | ExportMode::ReexportNamespaceObject(ExportModeReexportNamespaceObject {
        ref partial_namespace_export_info,
        ..
      })
      | ExportMode::ReexportFakeNamespaceObject(ExportModeFakeNamespaceObject {
        ref partial_namespace_export_info,
        ..
      }) => {
        let mut referenced_exports = vec![];
        collect_referenced_export_items(
          module_graph,
          runtime,
          &mut referenced_exports,
          vec![],
          Some(partial_namespace_export_info.as_data(module_graph)),
          matches!(mode, ExportMode::ReexportFakeNamespaceObject(_)),
          &mut Default::default(),
        );
        referenced_exports
          .into_iter()
          .map(|i| ExtendedReferencedExport::Array(i.into_iter().map(|i| i.to_owned()).collect()))
          .collect::<Vec<_>>()
      }
      ExportMode::NormalReexport(mode) => {
        let mut referenced_exports = vec![];
        for item in &mode.items {
          if item.hidden {
            continue;
          }
          collect_referenced_export_items(
            module_graph,
            runtime,
            &mut referenced_exports,
            item.ids.iter().collect(),
            Some(item.export_info.as_data(module_graph)),
            false,
            &mut Default::default(),
          );
        }
        referenced_exports
          .into_iter()
          .map(|i| ExtendedReferencedExport::Array(i.into_iter().map(|i| i.to_owned()).collect()))
          .collect::<Vec<_>>()
      }
    }
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::Transitive
  }
}

struct ESMExportImportedSpecifierDependencyCondition(DependencyId);

impl DependencyConditionFn for ESMExportImportedSpecifierDependencyCondition {
  fn get_connection_state(
    &self,
    _conn: &rspack_core::ModuleGraphConnection,
    runtime: Option<&RuntimeSpec>,
    module_graph: &ModuleGraph,
    module_graph_cache: &ModuleGraphCacheArtifact,
  ) -> ConnectionState {
    let dep = module_graph
      .dependency_by_id(&self.0)
      .expect("should have dependency");
    let down_casted_dep = dep
      .downcast_ref::<ESMExportImportedSpecifierDependency>()
      .expect("should be ESMExportImportedSpecifierDependency");
    let mode = down_casted_dep.get_mode(module_graph, runtime, module_graph_cache);
    ConnectionState::Active(!matches!(
      mode,
      ExportMode::Unused(_) | ExportMode::EmptyStar(_)
    ))
  }
}

#[cacheable_dyn]
impl ModuleDependency for ESMExportImportedSpecifierDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.request
  }

  fn set_request(&mut self, request: String) {
    self.request = request.into();
  }

  fn get_condition(&self) -> Option<DependencyCondition> {
    let id = self.id;
    Some(DependencyCondition::new_fn(
      ESMExportImportedSpecifierDependencyCondition(id),
    ))
  }

  fn factorize_info(&self) -> &FactorizeInfo {
    &self.factorize_info
  }

  fn factorize_info_mut(&mut self) -> &mut FactorizeInfo {
    &mut self.factorize_info
  }
}

enum ValueKey {
  False,
  Null,
  Name,
  UsedName(UsedName),
}

impl From<Option<UsedName>> for ValueKey {
  fn from(value: Option<UsedName>) -> Self {
    match value {
      None => Self::False,
      Some(used) => Self::UsedName(used),
    }
  }
}

impl AsContextDependency for ESMExportImportedSpecifierDependency {}

/// return (names, dependency_indices)
fn determine_export_assignments(
  module_graph: &ModuleGraph,
  dependencies: &[DependencyId],
  additional_dependency: Option<DependencyId>,
) -> (Vec<Atom>, Vec<usize>) {
  // https://github.com/webpack/webpack/blob/ac7e531436b0d47cd88451f497cdfd0dad41535d/lib/dependencies/HarmonyExportImportedSpecifierDependency.js#L109
  // js `Set` keep the insertion order, use `IndexSet` to align there behavior
  let mut names: IndexSet<Atom, BuildHasherDefault<FxHasher>> = IndexSet::default();
  let mut dependency_indices =
    Vec::with_capacity(dependencies.len() + usize::from(additional_dependency.is_some()));

  for dependency in dependencies.iter().chain(additional_dependency.iter()) {
    if let Some(module_identifier) = module_graph.module_identifier_by_dependency_id(dependency) {
      let exports_info = module_graph
        .get_prefetched_exports_info(module_identifier, PrefetchExportsInfoMode::Default);
      for (_name, export_info) in exports_info.exports() {
        // SAFETY: This is safe because a real export can't export empty string
        let export_info_name = export_info.name().expect("export name is empty");
        if matches!(export_info.provided(), Some(ExportProvided::Provided))
          && export_info_name != "default"
          && !names.contains(export_info_name)
        {
          names.insert(export_info_name.clone());
        }
      }
    }
    dependency_indices.push(names.len());
  }

  (names.into_iter().collect(), dependency_indices)
}

fn find_dependency_for_name<'a>(
  names: impl Iterator<Item = (usize, &'a Atom)>,
  mut dependency_indices: impl Iterator<Item = &'a usize>,
  name: &Atom,
  mut dependencies: impl Iterator<Item = &'a dyn ModuleDependency>,
) -> Option<&'a dyn ModuleDependency> {
  let mut idx = *dependency_indices.next()?;
  let mut dependency = dependencies.next();
  for (i, n) in names {
    while i >= idx {
      dependency = dependencies.next();
      idx = *dependency_indices.next()?;
    }
    if n == name {
      return dependency;
    }
  }
  None
}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct ESMExportImportedSpecifierDependencyTemplate;

impl ESMExportImportedSpecifierDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Dependency(DependencyType::EsmExportImportedSpecifier)
  }
}

impl DependencyTemplate for ESMExportImportedSpecifierDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    _source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<ESMExportImportedSpecifierDependency>()
      .expect("ESMExportImportedSpecifierDependencyTemplate should only be used for ESMExportImportedSpecifierDependency");
    let TemplateContext {
      compilation,
      runtime,
      concatenation_scope,
      ..
    } = code_generatable_context;

    let module_graph = compilation.get_module_graph();
    let module_graph_cache = &compilation.module_graph_cache_artifact;
    let mode = dep.get_mode(&module_graph, *runtime, module_graph_cache);

    if let Some(ref mut scope) = concatenation_scope {
      if let ExportMode::ReexportUndefined(mode) = mode {
        scope.register_raw_export(
          mode.name.clone(),
          String::from("/* reexport non-default export from non-ESM */ undefined"),
        );
      }
      return;
    }

    if !matches!(mode, ExportMode::Unused(_) | ExportMode::EmptyStar(_)) {
      esm_import_dependency_apply(dep, dep.source_order, code_generatable_context);
      dep.add_export_fragments(code_generatable_context, mode);
    }
  }
}

fn render_used_name(used: Option<&UsedName>) -> String {
  match used {
    None => "/* unused export */ undefined".to_string(),
    Some(UsedName::Normal(value_key)) if value_key.len() == 1 => value_key[0].to_string(),
    _ => unreachable!("export should only have one name"),
  }
}
