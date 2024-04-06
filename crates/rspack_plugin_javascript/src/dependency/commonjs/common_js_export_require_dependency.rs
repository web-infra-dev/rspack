use itertools::Itertools;
use rspack_core::{
  module_raw, process_export_info, property_access, AsContextDependency, Dependency,
  DependencyCategory, DependencyId, DependencyTemplate, DependencyType, ErrorSpan,
  ExportInfoProvided, ExportNameOrSpec, ExportSpec, ExportsOfExportsSpec, ExportsSpec, ExportsType,
  ExtendedReferencedExport, ModuleDependency, ModuleGraph, ModuleIdentifier, Nullable,
  ReferencedExport, RuntimeGlobals, RuntimeSpec, TemplateContext, TemplateReplaceSource,
  UsageState, UsedName,
};
use rustc_hash::FxHashSet;
use swc_core::atoms::Atom;

use super::ExportsBase;

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct CommonJsExportRequireDependency {
  id: DependencyId,
  request: String,
  optional: bool,
  span: Option<ErrorSpan>,
  range: (u32, u32),
  base: ExportsBase,
  names: Vec<Atom>,
  ids: Vec<Atom>,
  result_used: bool,
}

impl CommonJsExportRequireDependency {
  pub fn new(
    request: String,
    optional: bool,
    span: Option<ErrorSpan>,
    range: (u32, u32),
    base: ExportsBase,
    names: Vec<Atom>,
    result_used: bool,
  ) -> Self {
    Self {
      id: DependencyId::new(),
      request,
      optional,
      span,
      range,
      base,
      names,
      ids: vec![],
      result_used,
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
    let mut imported_exports_info = Some(mg.get_exports_info(imported_module));
    let ids = self.get_ids(mg);
    if !ids.is_empty() {
      imported_exports_info = imported_exports_info
        .expect("Should get exports info from imported module")
        .id
        .get_nested_exports_info(Some(ids), mg)
        .map(|id| id.get_exports_info(mg));
    }

    let mut exports_info = Some(
      mg.get_exports_info(
        mg.get_parent_module(&self.id)
          .expect("Should get parent module"),
      ),
    );

    if !self.names.is_empty() {
      exports_info = exports_info
        .expect("Should get exports info from imported module")
        .id
        .get_nested_exports_info(Some(self.names.clone()), mg)
        .map(|id| id.get_exports_info(mg));
    }

    let no_extra_exports = imported_exports_info.is_some_and(|imported_exports_info| {
      imported_exports_info
        .other_exports_info
        .get_export_info(mg)
        .provided
        .is_some_and(|provided| matches!(provided, ExportInfoProvided::False))
    });

    let no_extra_imports = exports_info.is_some_and(|exports_info| {
      exports_info.other_exports_info.get_used(mg, runtime) == UsageState::Unused
    });

    if !no_extra_exports && !no_extra_imports {
      return None;
    }

    let is_namespace_import = matches!(
      mg.module_by_identifier(imported_module)
        .expect("Should get imported module")
        .get_exports_type_readonly(mg, false),
      ExportsType::Namespace
    );

    let mut exports = FxHashSet::default();

    if no_extra_imports {
      let Some(exports_info) = exports_info else {
        unreachable!();
      };
      for export_info_id in exports_info.get_ordered_exports() {
        let export_info = export_info_id.get_export_info(mg);
        let name = &export_info.name;
        if matches!(export_info.get_used(runtime), UsageState::Unused) {
          continue;
        }
        if let Some(name) = name {
          if name == "__esModule" && is_namespace_import {
            exports.insert(name.to_owned());
          } else if let Some(imported_exports_info) = imported_exports_info {
            let imported_export_info = imported_exports_info.id.get_read_only_export_info(name, mg);
            if matches!(
              imported_export_info.provided,
              Some(ExportInfoProvided::False)
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
      let Some(imported_exports_info) = imported_exports_info else {
        unreachable!();
      };
      for imported_export_info_id in imported_exports_info.get_ordered_exports() {
        let imported_export_info = imported_export_info_id.get_export_info(mg);
        let name = &imported_export_info.name;
        if let Some(name) = name {
          if matches!(
            imported_export_info.provided,
            Some(ExportInfoProvided::False)
          ) {
            continue;
          }
          if let Some(exports_info) = exports_info {
            let export_info = exports_info.id.get_read_only_export_info(name, mg);
            if matches!(export_info.get_used(runtime), UsageState::Unused) {
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
}

impl Dependency for CommonJsExportRequireDependency {
  fn dependency_debug_name(&self) -> &'static str {
    "CommonJsExportRequireDependency"
  }

  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::CommonJS
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::CjsExportRequire
  }

  fn get_exports(&self, mg: &ModuleGraph) -> Option<ExportsSpec> {
    let ids = self.get_ids(mg);

    if self.names.len() == 1 {
      let Some(name) = self.names.first() else {
        unreachable!();
      };
      let Some(from) = mg.connection_by_dependency(&self.id) else {
        return None;
      };
      Some(ExportsSpec {
        exports: ExportsOfExportsSpec::Array(vec![ExportNameOrSpec::ExportSpec(ExportSpec {
          name: name.to_owned(),
          from: Some(from.to_owned()),
          can_mangle: Some(false),
          export: Some(if ids.is_empty() {
            Nullable::Null
          } else {
            Nullable::Value(ids)
          }),
          ..Default::default()
        })]),
        dependencies: Some(vec![*from.module_identifier()]),
        ..Default::default()
      })
    } else if self.names.is_empty() {
      let Some(from) = mg.connection_by_dependency(&self.id) else {
        return None;
      };
      if let Some(reexport_info) = self.get_star_reexports(mg, None, from.module_identifier()) {
        Some(ExportsSpec {
          exports: ExportsOfExportsSpec::Array(
            reexport_info
              .iter()
              .map(|name| {
                let mut export = ids.clone();
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
          exports: ExportsOfExportsSpec::True,
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
        exports: ExportsOfExportsSpec::Array(vec![ExportNameOrSpec::ExportSpec(ExportSpec {
          name: name.to_owned(),
          can_mangle: Some(false),
          ..Default::default()
        })]),
        dependencies: None,
        ..Default::default()
      })
    }
  }

  fn get_ids(&self, mg: &ModuleGraph) -> Vec<Atom> {
    mg.get_dep_meta_if_existing(&self.id)
      .map(|meta| meta.ids.clone())
      .unwrap_or_else(|| self.ids.clone())
  }
}

impl DependencyTemplate for CommonJsExportRequireDependency {
  fn apply(
    &self,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
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

    let used = mg.get_exports_info(&module.identifier()).id.get_used_name(
      mg,
      *runtime,
      UsedName::Vec(self.names.clone()),
    );

    let base = if self.base.is_exports() {
      runtime_requirements.insert(RuntimeGlobals::EXPORTS);
      exports_argument.to_string()
    } else if self.base.is_module_exports() {
      runtime_requirements.insert(RuntimeGlobals::MODULE);
      format!("{}.exports", module_argument)
    } else if self.base.is_this() {
      runtime_requirements.insert(RuntimeGlobals::THIS_AS_EXPORTS);
      "this".to_string()
    } else {
      unreachable!()
    };

    let mut require_expr = module_raw(
      compilation,
      runtime_requirements,
      &self.id,
      &self.request,
      false,
    );

    if let Some(imported_module) = mg.get_module_by_dependency_id(&self.id) {
      let ids = self.get_ids(mg);
      if let Some(used_imported) = mg
        .get_exports_info(&imported_module.identifier())
        .id
        .get_used_name(mg, *runtime, UsedName::Vec(ids))
      {
        require_expr = format!(
          "{}{}",
          require_expr,
          property_access(
            match used_imported {
              UsedName::Str(name) => vec![name].into_iter(),
              UsedName::Vec(names) => names.into_iter(),
            },
            0
          )
        )
      }
    }

    if self.base.is_expression() {
      let expr = match used {
        Some(used) => format!(
          "{base}{} = {require_expr}",
          property_access(
            match used {
              UsedName::Str(name) => vec![name].into_iter(),
              UsedName::Vec(names) => names.into_iter(),
            },
            0
          )
        ),
        None => format!("/* unused reexport */ {}", require_expr),
      };
      source.replace(self.range.0, self.range.1, expr.as_str(), None)
    } else if self.base.is_define_property() {
      panic!("TODO")
    } else {
      panic!("Unexpected type");
    }
  }

  fn dependency_id(&self) -> Option<DependencyId> {
    Some(self.id)
  }
}

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

  fn get_referenced_exports(
    &self,
    mg: &ModuleGraph,
    runtime: Option<&RuntimeSpec>,
  ) -> Vec<ExtendedReferencedExport> {
    let ids = self.get_ids(mg);
    let get_full_result = || {
      if ids.is_empty() {
        vec![ExtendedReferencedExport::Array(vec![])]
      } else {
        vec![ExtendedReferencedExport::Export(ReferencedExport {
          name: ids.clone(),
          can_mangle: false,
        })]
      }
    };
    if self.result_used {
      return get_full_result();
    }
    let mut exports_info = mg.get_exports_info(
      mg.get_parent_module(&self.id)
        .expect("Can not get parent module"),
    );

    for name in &self.names {
      let export_info = exports_info.id.get_read_only_export_info(name, mg);
      let used = export_info.get_used(runtime);
      if matches!(used, UsageState::Unused) {
        return vec![ExtendedReferencedExport::Array(vec![])];
      }
      if !matches!(used, UsageState::OnlyPropertiesUsed) {
        return get_full_result();
      }

      match export_info.exports_info {
        Some(v) => exports_info = v.get_exports_info(mg),
        None => return get_full_result(),
      };
    }

    if !matches!(
      exports_info.other_exports_info.get_used(mg, runtime),
      UsageState::Unused
    ) {
      return get_full_result();
    }

    let mut referenced_exports = vec![];
    for export_info_id in exports_info.get_ordered_exports() {
      let export_info = export_info_id.get_export_info(mg);
      let prefix = ids
        .iter()
        .chain(if let Some(name) = &export_info.name {
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
        Some(*export_info_id),
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
        })
      })
      .collect_vec()
  }
}
impl AsContextDependency for CommonJsExportRequireDependency {}
