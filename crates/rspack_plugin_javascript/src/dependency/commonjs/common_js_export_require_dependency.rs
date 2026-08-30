use itertools::Itertools;
use rspack_cacheable::{
  cacheable, cacheable_dyn,
  with::{AsPreset, AsVec},
};
use rspack_collections::{IdentifierMap, IdentifierSet};
use rspack_core::{
  AsContextDependency, ConnectionState, Dependency, DependencyCategory, DependencyCodeGeneration,
  DependencyCondition, DependencyConditionFn, DependencyId, DependencyRange, DependencyTemplate,
  DependencyTemplateType, DependencyType, ExportNameOrSpec, ExportProvided, ExportSpec,
  ExportsInfoArtifact, ExportsOfExportsSpec, ExportsSpec, ExportsType, ModuleDependency,
  ModuleGraph, ModuleGraphCacheArtifact, ModuleIdentifier, Nullable, ReferencedExport, RuntimeSpec,
  SideEffectsStateArtifact, TemplateContext, TemplateReplaceSource, UsageState, UsedName,
  collect_referenced_export_items, create_exports_object_referenced, create_no_exports_referenced,
  property_access, to_normal_comment,
};
use rspack_util::json_stringify_str;
use rustc_hash::FxHashSet;
use swc_atoms::Atom;

use super::ExportsBase;
use crate::dependency::commonjs::OBJECT_PROTOTYPE_METHODS;

#[cacheable]
#[allow(unused)]
#[derive(Debug)]
pub struct CommonJsExportRequireDependency {
  id: DependencyId,
  request: String,
  optional: bool,
  range: DependencyRange,
  value_range: Option<DependencyRange>,
  define_property_base_range: Option<DependencyRange>,
  define_property_name_range: Option<DependencyRange>,
  base: ExportsBase,
  #[cacheable(with=AsVec<AsPreset>)]
  names: Vec<Atom>,
  #[cacheable(with=AsVec<AsPreset>)]
  ids: Vec<Atom>,
  result_used: bool,
  getter: bool,
}

impl CommonJsExportRequireDependency {
  #[allow(clippy::too_many_arguments)]
  pub fn new(
    request: String,
    optional: bool,
    range: DependencyRange,
    value_range: Option<DependencyRange>,
    define_property_base_range: Option<DependencyRange>,
    define_property_name_range: Option<DependencyRange>,
    base: ExportsBase,
    names: Vec<Atom>,
    ids: Vec<Atom>,
    result_used: bool,
    getter: bool,
  ) -> Self {
    Self {
      id: DependencyId::new(),
      request,
      optional,
      range,
      value_range,
      define_property_base_range,
      define_property_name_range,
      base,
      names,
      ids,
      result_used,
      getter,
    }
  }
}

impl CommonJsExportRequireDependency {
  // NOTE:
  // webpack return checked set but never use it
  // https://github.com/webpack/webpack/blob/08770761c8c7aa1e6e18b77d3deee8cc9871bd87/lib/dependencies/CommonJsExportRequireDependency.js#L283
  fn get_star_reexports(
    &self,
    mg: &ModuleGraph,
    mg_cache: &ModuleGraphCacheArtifact,
    exports_info_artifact: &ExportsInfoArtifact,
    runtime: Option<&RuntimeSpec>,
    imported_module: &ModuleIdentifier,
  ) -> Option<FxHashSet<Atom>> {
    let ids = self.get_ids(mg);
    let mut imported_exports_info =
      Some(exports_info_artifact.get_exports_info_data(imported_module));

    if !ids.is_empty() {
      let Some(nested_exports_info) = &imported_exports_info else {
        unreachable!();
      };
      let nested = nested_exports_info
        .get_nested_exports_info(exports_info_artifact, Some(ids))
        .map(|data| data.id());

      imported_exports_info = nested.map(|id| exports_info_artifact.get_exports_info_by_id(&id));
    }

    let mut exports_info = Some(
      exports_info_artifact.get_exports_info_data(
        mg.get_parent_module(&self.id)
          .expect("Should get parent module"),
      ),
    );

    if !self.names.is_empty() {
      let Some(nested_exports_info) = &exports_info else {
        unreachable!();
      };
      let nested = nested_exports_info
        .get_nested_exports_info(exports_info_artifact, Some(&self.names))
        .map(|data| data.id());
      exports_info = nested.map(|id| exports_info_artifact.get_exports_info_by_id(&id));
    };

    let no_extra_exports = imported_exports_info.as_ref().is_some_and(|data| {
      let provided = data.other_exports_info().provided();
      matches!(provided, Some(ExportProvided::NotProvided))
    });

    let no_extra_imports = exports_info.as_ref().is_some_and(|data| {
      matches!(
        data.other_exports_info().get_used(runtime),
        UsageState::Unused
      )
    });

    if !no_extra_exports && !no_extra_imports {
      return None;
    }

    let is_namespace_import = matches!(
      mg.module_by_identifier(imported_module)
        .expect("Should get imported module")
        .get_exports_type(mg, mg_cache, exports_info_artifact, false),
      ExportsType::Namespace
    );

    let mut exports = FxHashSet::default();

    if no_extra_imports {
      let Some(exports_info) = &exports_info else {
        unreachable!();
      };
      for export_info in exports_info.exports().values() {
        let name = export_info.name();
        if matches!(export_info.get_used(runtime), UsageState::Unused) {
          continue;
        }
        if let Some(name) = name {
          if name == "__esModule" && is_namespace_import {
            exports.insert(name.to_owned());
          } else if let Some(imported_exports_info) = &imported_exports_info {
            let imported_export_info = imported_exports_info.get_read_only_export_info(name);
            if matches!(
              imported_export_info.provided(),
              Some(ExportProvided::NotProvided)
            ) {
              continue;
            }
            exports.insert(name.to_owned());
          } else {
            exports.insert(name.to_owned());
          }
        }
      }
    } else if no_extra_exports {
      let Some(imported_exports_info) = &imported_exports_info else {
        unreachable!();
      };
      for imported_export_info in imported_exports_info.exports().values() {
        let name = imported_export_info.name();
        if let Some(name) = name {
          if matches!(
            imported_export_info.provided(),
            Some(ExportProvided::NotProvided)
          ) {
            continue;
          }
          if let Some(exports_info) = &exports_info {
            let export_info = exports_info.get_read_only_export_info(name);
            let used = export_info.get_used(runtime);
            if matches!(used, UsageState::Unused) {
              continue;
            }
            exports.insert(name.to_owned());
          }
        }
      }
      if is_namespace_import {
        exports.insert(Atom::from("__esModule"));
      }
    }

    Some(exports)
  }

  pub fn get_ids<'a>(&'a self, mg: &'a ModuleGraph) -> &'a [Atom] {
    mg.get_dep_meta_if_existing(&self.id)
      .map_or_else(|| self.ids.as_slice(), |meta| meta.ids.as_slice())
  }

  fn is_all_exported_by_module_exports(&self) -> bool {
    self.base.is_module_exports() && self.names.is_empty()
  }
}

#[cacheable_dyn]
impl Dependency for CommonJsExportRequireDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::CommonJS
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::CjsExportRequire
  }

  fn get_exports(
    &self,
    mg: &ModuleGraph,
    mg_cache: &ModuleGraphCacheArtifact,
    exports_info_artifact: &ExportsInfoArtifact,
  ) -> Option<ExportsSpec> {
    let ids = self.get_ids(mg);

    if self.names.len() == 1 {
      let Some(name) = self.names.first() else {
        unreachable!();
      };
      let from = mg.connection_by_dependency_id(&self.id)?;
      Some(ExportsSpec {
        exports: ExportsOfExportsSpec::Names(vec![ExportNameOrSpec::ExportSpec(ExportSpec {
          name: name.to_owned(),
          from: Some(from.to_owned()),
          can_mangle: Some(!OBJECT_PROTOTYPE_METHODS.contains(&name.as_str())),
          export: Some(if ids.is_empty() {
            Nullable::Null
          } else {
            Nullable::Value(ids.to_vec())
          }),
          ..Default::default()
        })]),
        dependencies: Some(vec![*from.module_identifier()]),
        ..Default::default()
      })
    } else if self.names.is_empty() {
      let from = mg.connection_by_dependency_id(&self.id)?;
      if let Some(reexport_info) = self.get_star_reexports(
        mg,
        mg_cache,
        exports_info_artifact,
        None,
        from.module_identifier(),
      ) {
        Some(ExportsSpec {
          exports: ExportsOfExportsSpec::Names(
            reexport_info
              .iter()
              .map(|name| {
                let mut export = ids.to_vec();
                export.extend(vec![name.to_owned()]);
                ExportNameOrSpec::ExportSpec(ExportSpec {
                  name: name.to_owned(),
                  from: Some(from.to_owned()),
                  export: Some(Nullable::Value(export)),
                  // `module.exports = require("./m")` can't be mangled
                  can_mangle: Some(!self.is_all_exported_by_module_exports()),
                  ..Default::default()
                })
              })
              .collect_vec(),
          ),
          dependencies: Some(vec![*from.module_identifier()]),
          ..Default::default()
        })
      } else {
        Some(ExportsSpec {
          exports: ExportsOfExportsSpec::UnknownExports,
          from: if ids.is_empty() {
            Some(from.to_owned())
          } else {
            None
          },
          // `module.exports = require("./m")` can't be mangled
          can_mangle: Some(!self.is_all_exported_by_module_exports()),
          dependencies: Some(vec![*from.module_identifier()]),
          ..Default::default()
        })
      }
    } else {
      let Some(name) = self.names.first() else {
        unreachable!();
      };
      Some(ExportsSpec {
        exports: ExportsOfExportsSpec::Names(vec![ExportNameOrSpec::ExportSpec(ExportSpec {
          name: name.to_owned(),
          can_mangle: Some(!OBJECT_PROTOTYPE_METHODS.contains(&name.as_str())),
          ..Default::default()
        })]),
        dependencies: None,
        ..Default::default()
      })
    }
  }

  fn get_referenced_exports(
    &self,
    mg: &ModuleGraph,
    _module_graph_cache: &ModuleGraphCacheArtifact,
    exports_info_artifact: &ExportsInfoArtifact,
    runtime: Option<&RuntimeSpec>,
  ) -> Vec<ReferencedExport> {
    let ids = self.get_ids(mg);
    let get_full_result = || {
      if ids.is_empty() {
        create_exports_object_referenced()
      } else {
        vec![
          ReferencedExport::from(ids)
            // `module.exports = require("./m")` can't be mangled
            .with_can_mangle(!self.is_all_exported_by_module_exports())
            .with_can_inline(false),
        ]
      }
    };
    if self.result_used {
      return get_full_result();
    }
    let mut exports_info = exports_info_artifact.get_exports_info_data(
      mg.get_parent_module(&self.id)
        .expect("Can not get parent module"),
    );

    for name in &self.names {
      let export_info = exports_info.get_read_only_export_info(name);
      let used = export_info.get_used(runtime);
      if matches!(used, UsageState::Unused) {
        return create_no_exports_referenced();
      }
      if !matches!(used, UsageState::OnlyPropertiesUsed) {
        return get_full_result();
      }

      match export_info.exports_info() {
        Some(v) => exports_info = v.as_data(exports_info_artifact),
        None => return get_full_result(),
      };
    }

    if !matches!(
      exports_info.other_exports_info().get_used(runtime),
      UsageState::Unused
    ) {
      return get_full_result();
    }

    let mut referenced_exports = vec![];
    for export_info in exports_info.exports().values() {
      let prefix = ids
        .iter()
        .chain(if let Some(name) = export_info.name() {
          vec![name]
        } else {
          vec![]
        })
        .collect_vec();
      collect_referenced_export_items(
        exports_info_artifact,
        runtime,
        &mut referenced_exports,
        prefix,
        Some(export_info),
        false,
        &mut Default::default(),
      )
    }

    referenced_exports
      .into_iter()
      .map(|name| {
        ReferencedExport::from(name)
          // `module.exports = require("./m")` can't be mangled
          .with_can_mangle(!self.is_all_exported_by_module_exports())
          .with_can_inline(false)
      })
      .collect_vec()
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::Transitive
  }
}

#[cacheable_dyn]
impl ModuleDependency for CommonJsExportRequireDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.request
  }

  fn get_optional(&self) -> bool {
    self.optional
  }

  fn get_condition(&self) -> Option<DependencyCondition> {
    (!self.result_used && !self.names.is_empty())
      .then(|| DependencyCondition::new(CommonJsExportRequireDependencyCondition))
  }
}

struct CommonJsExportRequireDependencyCondition;

impl DependencyConditionFn for CommonJsExportRequireDependencyCondition {
  fn get_connection_state(
    &self,
    connection: &rspack_core::ModuleGraphConnection,
    runtime: Option<&RuntimeSpec>,
    module_graph: &ModuleGraph,
    module_graph_cache: &ModuleGraphCacheArtifact,
    side_effects_state_artifact: &SideEffectsStateArtifact,
    exports_info_artifact: &ExportsInfoArtifact,
  ) -> ConnectionState {
    let dependency = module_graph.dependency_by_id(&connection.dependency_id);
    let dependency = dependency
      .downcast_ref::<CommonJsExportRequireDependency>()
      .expect("should be CommonJsExportRequireDependency");
    let Some(parent_module) = module_graph.get_parent_module(&dependency.id) else {
      return ConnectionState::Active(true);
    };
    if exports_info_artifact
      .get_exports_info_data(parent_module)
      .get_used_name(exports_info_artifact, runtime, &dependency.names)
      .is_some()
    {
      return ConnectionState::Active(true);
    }
    if dependency.getter {
      return ConnectionState::Active(false);
    }
    let module_identifier = *connection.module_identifier();
    if let Some(state) =
      side_effects_state_artifact.module_evaluation_side_effects_state(&module_identifier)
    {
      return state;
    }
    if let Some(module) = module_graph.module_by_identifier(&module_identifier) {
      module.get_side_effects_connection_state(
        module_graph,
        module_graph_cache,
        side_effects_state_artifact,
        &mut IdentifierSet::default(),
        &mut IdentifierMap::default(),
      )
    } else {
      ConnectionState::Active(true)
    }
  }
}
impl AsContextDependency for CommonJsExportRequireDependency {}

#[cacheable_dyn]
impl DependencyCodeGeneration for CommonJsExportRequireDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(CommonJsExportRequireDependencyTemplate::template_type())
  }
}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct CommonJsExportRequireDependencyTemplate;

impl CommonJsExportRequireDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Dependency(DependencyType::CjsExportRequire)
  }
}

impl DependencyTemplate for CommonJsExportRequireDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<CommonJsExportRequireDependency>()
      .expect("CommonJsExportRequireDependencyTemplate should only be used for CommonJsExportRequireDependency");

    let TemplateContext {
      compilation,
      module,
      runtime,
      runtime_template,
      ..
    } = code_generatable_context;

    let mg = &compilation.get_module_graph();

    let module = mg
      .module_by_identifier(&module.identifier())
      .expect("should have mgm");

    let exports_argument = module.get_exports_argument();
    let module_argument = module.get_module_argument();

    let exports_info = compilation
      .exports_info_artifact
      .get_exports_info_data(&module.identifier());
    let used = exports_info.get_used_name(&compilation.exports_info_artifact, *runtime, &dep.names);

    if used.is_none()
      && let Some(connection) = mg.connection_by_dependency_id(&dep.id)
      && !connection.is_target_active(
        mg,
        *runtime,
        &compilation.module_graph_cache_artifact,
        &compilation
          .build_module_graph_artifact
          .side_effects_state_artifact,
        &compilation.exports_info_artifact,
      )
    {
      source.replace(
        dep.range.start,
        dep.range.end,
        "/* unused reexport */ 0".to_string(),
        None,
      );
      return;
    }

    let base = if dep.base.is_exports() {
      runtime_template.render_exports_argument(exports_argument)
    } else if dep.base.is_module_exports() {
      format!(
        "{}.exports",
        runtime_template.render_module_argument(module_argument)
      )
    } else if dep.base.is_this() {
      runtime_template.render_this_exports()
    } else {
      unreachable!()
    };

    let require_expr = if let Some(imported_module) = mg.get_module_by_dependency_id(&dep.id)
      && let ids = dep.get_ids(mg)
      && let Some(used_imported) = compilation
        .exports_info_artifact
        .get_exports_info_data(&imported_module.identifier())
        .get_used_name(&compilation.exports_info_artifact, *runtime, ids)
    {
      match used_imported {
        UsedName::Normal(used_imported) => {
          format!(
            "{}{}{}",
            runtime_template.module_raw(compilation, &dep.id, &dep.request, false,),
            to_normal_comment(&property_access(ids, 0)),
            property_access(used_imported, 0)
          )
        }
        UsedName::Inlined(inlined) => inlined.render(&to_normal_comment(&format!(
          "inlined export {}",
          property_access(ids, 0)
        ))),
      }
    } else {
      runtime_template.module_raw(compilation, &dep.id, &dep.request, false)
    };

    if dep.base.is_expression() {
      let expr = match used {
        Some(UsedName::Normal(used)) => {
          format!("{base}{} = {require_expr}", property_access(used, 0))
        }
        Some(UsedName::Inlined(_)) => {
          // Export a inlinable const from cjs is not possible for now but we compat it here
          format!("/* inlined reexport */ {require_expr}")
        }
        None => format!("/* unused reexport */ {require_expr}"),
      };
      source.replace(dep.range.start, dep.range.end, expr, None)
    } else if dep.base.is_define_property() {
      let value_range = dep
        .value_range
        .expect("define property reexport should have a value range");
      let Some(UsedName::Normal(used)) = used else {
        source.replace(
          dep.range.start,
          dep.range.end,
          format!("/* unused reexport */ {require_expr}"),
          None,
        );
        return;
      };
      let (last, parent) = used
        .split_last()
        .expect("define property reexport should have an export name");
      let base_range = dep
        .define_property_base_range
        .expect("define property reexport should have a base range");
      let name_range = dep
        .define_property_name_range
        .expect("define property reexport should have a name range");
      source.replace(
        base_range.start,
        base_range.end,
        format!("{base}{}", property_access(parent, 0)),
        None,
      );
      source.replace(
        name_range.start,
        name_range.end,
        json_stringify_str(last),
        None,
      );
      source.replace(value_range.start, value_range.end, require_expr, None);
    } else {
      panic!("Unexpected type");
    }
  }
}
