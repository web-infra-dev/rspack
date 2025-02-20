use itertools::Itertools;
use rspack_cacheable::{
  cacheable, cacheable_dyn,
  with::{AsPreset, AsVec},
};
use rspack_core::{
  module_raw, process_export_info, property_access, AsContextDependency, Compilation, Dependency,
  DependencyCategory, DependencyId, DependencyRange, DependencyTemplate, DependencyType,
  ExportInfoProvided, ExportNameOrSpec, ExportSpec, ExportsOfExportsSpec, ExportsSpec, ExportsType,
  ExtendedReferencedExport, FactorizeInfo, ModuleDependency, ModuleGraph, ModuleIdentifier,
  Nullable, ReferencedExport, RuntimeGlobals, RuntimeSpec, TemplateContext, TemplateReplaceSource,
  UsageState, UsedName,
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
    let mut imported_exports_info = Some(mg.get_exports_info(imported_module));
    let ids = self.get_ids(mg);
    if !ids.is_empty() {
      imported_exports_info = imported_exports_info
        .expect("Should get exports info from imported module")
        .get_nested_exports_info(mg, Some(ids));
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
        .get_nested_exports_info(mg, Some(&self.names));
    }

    let no_extra_exports = imported_exports_info.is_some_and(|imported_exports_info| {
      imported_exports_info
        .other_exports_info(mg)
        .provided(mg)
        .is_some_and(|provided| matches!(provided, ExportInfoProvided::False))
    });

    let no_extra_imports = exports_info.is_some_and(|exports_info| {
      exports_info.other_exports_info(mg).get_used(mg, runtime) == UsageState::Unused
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
      let Some(exports_info) = exports_info else {
        unreachable!();
      };
      for export_info in exports_info.ordered_exports(mg) {
        let name = export_info.name(mg);
        if matches!(export_info.get_used(mg, runtime), UsageState::Unused) {
          continue;
        }
        if let Some(name) = name {
          if name == "__esModule" && is_namespace_import {
            exports.insert(name.to_owned());
          } else if let Some(imported_exports_info) = imported_exports_info {
            let imported_export_info = imported_exports_info.get_read_only_export_info(mg, name);
            if matches!(
              imported_export_info.provided(mg),
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
      for imported_export_info in imported_exports_info.ordered_exports(mg) {
        let name = imported_export_info.name(mg);
        if let Some(name) = name {
          if matches!(
            imported_export_info.provided(mg),
            Some(ExportInfoProvided::False)
          ) {
            continue;
          }
          if let Some(exports_info) = exports_info {
            let export_info = exports_info.get_read_only_export_info(mg, name);
            if matches!(export_info.get_used(mg, runtime), UsageState::Unused) {
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

  fn get_exports(&self, mg: &ModuleGraph) -> Option<ExportsSpec> {
    let ids = self.get_ids(mg);

    if self.names.len() == 1 {
      let Some(name) = self.names.first() else {
        unreachable!();
      };
      let from = mg.connection_by_dependency_id(&self.id)?;
      Some(ExportsSpec {
        exports: ExportsOfExportsSpec::Array(vec![ExportNameOrSpec::ExportSpec(ExportSpec {
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
          exports: ExportsOfExportsSpec::Array(
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
          name: ids.to_vec(),
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
      let export_info = exports_info.get_read_only_export_info(mg, name);
      let used = export_info.get_used(mg, runtime);
      if matches!(used, UsageState::Unused) {
        return vec![ExtendedReferencedExport::Array(vec![])];
      }
      if !matches!(used, UsageState::OnlyPropertiesUsed) {
        return get_full_result();
      }

      match export_info.exports_info(mg) {
        Some(v) => exports_info = v,
        None => return get_full_result(),
      };
    }

    if !matches!(
      exports_info.other_exports_info(mg).get_used(mg, runtime),
      UsageState::Unused
    ) {
      return get_full_result();
    }

    let mut referenced_exports = vec![];
    for export_info in exports_info.ordered_exports(mg) {
      let prefix = ids
        .iter()
        .chain(if let Some(name) = export_info.name(mg) {
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
        Some(export_info),
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

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::Transitive
  }
}

#[cacheable_dyn]
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

    let used = mg.get_exports_info(&module.identifier()).get_used_name(
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
        .get_used_name(mg, *runtime, UsedName::Vec(ids.to_vec()))
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
      source.replace(self.range.start, self.range.end, expr.as_str(), None)
    } else if self.base.is_define_property() {
      panic!("TODO")
    } else {
      panic!("Unexpected type");
    }
  }

  fn dependency_id(&self) -> Option<DependencyId> {
    Some(self.id)
  }

  fn update_hash(
    &self,
    _hasher: &mut dyn std::hash::Hasher,
    _compilation: &Compilation,
    _runtime: Option<&RuntimeSpec>,
  ) {
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
