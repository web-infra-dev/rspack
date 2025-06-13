use itertools::Itertools;
use rspack_cacheable::{
  cacheable, cacheable_dyn,
  with::{AsPreset, AsVec},
};
use rspack_core::{
  module_raw, process_export_info, property_access, to_normal_comment, AsContextDependency,
  Dependency, DependencyCategory, DependencyCodeGeneration, DependencyId, DependencyRange,
  DependencyTemplate, DependencyTemplateType, DependencyType, ExportInfoGetter, ExportNameOrSpec,
  ExportProvided, ExportSpec, ExportsInfoGetter, ExportsOfExportsSpec, ExportsSpec, ExportsType,
  ExtendedReferencedExport, FactorizeInfo, ModuleDependency, ModuleGraph, ModuleGraphCacheArtifact,
  ModuleIdentifier, Nullable, PrefetchExportsInfoMode, ReferencedExport, RuntimeGlobals,
  RuntimeSpec, TemplateContext, TemplateReplaceSource, UsageState, UsedName,
};
use rustc_hash::FxHashSet;
use swc_core::atoms::Atom;

use super::ExportsBase;

#[cacheable]
#[allow(unused)]
#[derive(Debug, Clone)]
pub struct CommonJsExportRequireDependency {
  id: DependencyId,
  request: String,
  optional: bool,
  range: DependencyRange,
  base: ExportsBase,
  #[cacheable(with=AsVec<AsPreset>)]
  names: Vec<Atom>,
  #[cacheable(with=AsVec<AsPreset>)]
  ids: Vec<Atom>,
  result_used: bool,
  factorize_info: FactorizeInfo,
}

impl CommonJsExportRequireDependency {
  pub fn new(
    request: String,
    optional: bool,
    range: DependencyRange,
    base: ExportsBase,
    names: Vec<Atom>,
    result_used: bool,
  ) -> Self {
    Self {
      id: DependencyId::new(),
      request,
      optional,
      range,
      base,
      names,
      ids: vec![],
      result_used,
      factorize_info: Default::default(),
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
    runtime: Option<&RuntimeSpec>,
    imported_module: &ModuleIdentifier,
  ) -> Option<FxHashSet<Atom>> {
    let ids = self.get_ids(mg);
    let mut imported_exports_info = Some(mg.get_prefetched_exports_info(
      imported_module,
      PrefetchExportsInfoMode::NamedNestedAllExports(ids),
    ));

    if !ids.is_empty() {
      let Some(nested_exports_info) = &imported_exports_info else {
        unreachable!();
      };
      let nested = nested_exports_info
        .get_nested_exports_info(Some(ids))
        .map(|data| data.id);

      imported_exports_info =
        nested.map(|id| ExportsInfoGetter::prefetch(&id, mg, PrefetchExportsInfoMode::AllExports));
    }

    let mut exports_info = Some(
      mg.get_prefetched_exports_info(
        mg.get_parent_module(&self.id)
          .expect("Should get parent module"),
        PrefetchExportsInfoMode::NamedNestedAllExports(&self.names),
      ),
    );

    if !self.names.is_empty() {
      let Some(nested_exports_info) = &exports_info else {
        unreachable!();
      };
      let nested = nested_exports_info
        .get_nested_exports_info(Some(&self.names))
        .map(|data| data.id);
      exports_info =
        nested.map(|id| ExportsInfoGetter::prefetch(&id, mg, PrefetchExportsInfoMode::AllExports));
    };

    let no_extra_exports = imported_exports_info.as_ref().is_some_and(|data| {
      let provided = ExportInfoGetter::provided(data.other_exports_info());
      matches!(provided, Some(ExportProvided::NotProvided))
    });

    let no_extra_imports = exports_info.as_ref().is_some_and(|data| {
      matches!(
        ExportInfoGetter::get_used(data.other_exports_info(), runtime),
        UsageState::Unused
      )
    });

    if !no_extra_exports && !no_extra_imports {
      return None;
    }

    let is_namespace_import = matches!(
      mg.module_by_identifier(imported_module)
        .expect("Should get imported module")
        .get_exports_type(mg, false),
      ExportsType::Namespace
    );

    let mut exports = FxHashSet::default();

    if no_extra_imports {
      let Some(exports_info) = &exports_info else {
        unreachable!();
      };
      for (_, export_info) in exports_info.exports() {
        let name = ExportInfoGetter::name(export_info);
        if matches!(
          ExportInfoGetter::get_used(export_info, runtime),
          UsageState::Unused
        ) {
          continue;
        }
        if let Some(name) = name {
          if name == "__esModule" && is_namespace_import {
            exports.insert(name.to_owned());
          } else if let Some(imported_exports_info) = &imported_exports_info {
            let imported_export_info = imported_exports_info.get_read_only_export_info(name);
            if matches!(
              ExportInfoGetter::provided(imported_export_info),
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
      for (_, imported_export_info) in imported_exports_info.exports() {
        let name = ExportInfoGetter::name(imported_export_info);
        if let Some(name) = name {
          if matches!(
            ExportInfoGetter::provided(imported_export_info),
            Some(ExportProvided::NotProvided)
          ) {
            continue;
          }
          if let Some(exports_info) = &exports_info {
            let export_info = exports_info.get_read_only_export_info(name);
            let used = ExportInfoGetter::get_used(export_info, runtime);
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
      .map(|meta| meta.ids.as_slice())
      .unwrap_or_else(|| self.ids.as_slice())
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
    _mg_cache: &ModuleGraphCacheArtifact,
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
          can_mangle: Some(false),
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
      if let Some(reexport_info) = self.get_star_reexports(mg, None, from.module_identifier()) {
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
                  can_mangle: Some(false),
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
          can_mangle: Some(false),
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
          can_mangle: Some(false),
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
    runtime: Option<&RuntimeSpec>,
  ) -> Vec<ExtendedReferencedExport> {
    let ids = self.get_ids(mg);
    let get_full_result = || {
      if ids.is_empty() {
        vec![ExtendedReferencedExport::Array(vec![])]
      } else {
        vec![ExtendedReferencedExport::Export(ReferencedExport {
          name: ids.to_vec(),
          can_mangle: false,
          can_inline: false,
        })]
      }
    };
    if self.result_used {
      return get_full_result();
    }
    let mut exports_info = mg.get_prefetched_exports_info(
      mg.get_parent_module(&self.id)
        .expect("Can not get parent module"),
      PrefetchExportsInfoMode::NamedNestedAllExports(&self.names),
    );

    for name in &self.names {
      let export_info = exports_info.get_read_only_export_info(name);
      let used = ExportInfoGetter::get_used(export_info, runtime);
      if matches!(used, UsageState::Unused) {
        return vec![ExtendedReferencedExport::Array(vec![])];
      }
      if !matches!(used, UsageState::OnlyPropertiesUsed) {
        return get_full_result();
      }

      match ExportInfoGetter::exports_info(export_info) {
        Some(v) => exports_info = exports_info.redirect(v, false),
        None => return get_full_result(),
      };
    }

    if !matches!(
      ExportInfoGetter::get_used(exports_info.other_exports_info(), runtime),
      UsageState::Unused
    ) {
      return get_full_result();
    }

    let mut referenced_exports = vec![];
    for (_, export_info) in exports_info.exports() {
      let prefix = ids
        .iter()
        .chain(if let Some(name) = ExportInfoGetter::name(export_info) {
          vec![name]
        } else {
          vec![]
        })
        .map(|i| i.to_owned())
        .collect_vec();
      process_export_info(
        mg,
        runtime,
        &mut referenced_exports,
        prefix,
        Some(export_info.id()),
        false,
        &mut Default::default(),
      )
    }

    referenced_exports
      .iter()
      .map(|name| {
        ExtendedReferencedExport::Export(ReferencedExport {
          name: name.to_owned(),
          can_mangle: false,
          can_inline: false,
        })
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

  fn set_request(&mut self, request: String) {
    self.request = request;
  }

  fn factorize_info(&self) -> &FactorizeInfo {
    &self.factorize_info
  }

  fn factorize_info_mut(&mut self) -> &mut FactorizeInfo {
    &mut self.factorize_info
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
      runtime_requirements,
      ..
    } = code_generatable_context;

    let mg = &compilation.get_module_graph();

    let module = mg
      .module_by_identifier(&module.identifier())
      .expect("should have mgm");

    let exports_argument = module.get_exports_argument();
    let module_argument = module.get_module_argument();

    let used = ExportsInfoGetter::get_used_name(
      &mg.get_prefetched_exports_info(
        &module.identifier(),
        if dep.names.is_empty() {
          PrefetchExportsInfoMode::AllExports
        } else {
          PrefetchExportsInfoMode::NamedNestedExports(&dep.names)
        },
      ),
      *runtime,
      &dep.names,
    );

    let base = if dep.base.is_exports() {
      runtime_requirements.insert(RuntimeGlobals::EXPORTS);
      exports_argument.to_string()
    } else if dep.base.is_module_exports() {
      runtime_requirements.insert(RuntimeGlobals::MODULE);
      format!("{module_argument}.exports")
    } else if dep.base.is_this() {
      runtime_requirements.insert(RuntimeGlobals::THIS_AS_EXPORTS);
      "this".to_string()
    } else {
      unreachable!()
    };

    let require_expr = if let Some(imported_module) = mg.get_module_by_dependency_id(&dep.id)
      && let ids = dep.get_ids(mg)
      && let Some(used_imported) = ExportsInfoGetter::get_used_name(
        &mg.get_prefetched_exports_info(
          &imported_module.identifier(),
          if ids.is_empty() {
            PrefetchExportsInfoMode::AllExports
          } else {
            PrefetchExportsInfoMode::NamedNestedExports(ids)
          },
        ),
        *runtime,
        ids,
      ) {
      let comment = to_normal_comment(&property_access(ids, 0));
      match used_imported {
        UsedName::Normal(used_imported) => {
          format!(
            "{}{}{}",
            module_raw(
              compilation,
              runtime_requirements,
              &dep.id,
              &dep.request,
              false,
            ),
            comment,
            property_access(used_imported, 0)
          )
        }
        UsedName::Inlined(inlined) => format!("{}{}", comment, inlined.render()),
      }
    } else {
      module_raw(
        compilation,
        runtime_requirements,
        &dep.id,
        &dep.request,
        false,
      )
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
      source.replace(dep.range.start, dep.range.end, expr.as_str(), None)
    } else if dep.base.is_define_property() {
      panic!("TODO")
    } else {
      panic!("Unexpected type");
    }
  }
}
