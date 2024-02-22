use rspack_core::{ConstDependency, DependencyType, SpanExt};
use swc_core::atoms::Atom;
use swc_core::common::Span;
use swc_core::ecma::ast::{ExportAll, ExportSpecifier, ModuleExportName, NamedExport};

use super::harmony_import_dependency_parser_plugin::handle_harmony_import_side_effects_dep;
use super::JavascriptParserPlugin;
use crate::dependency::{HarmonyExportImportedSpecifierDependency, Specifier};
use crate::visitors::JavascriptParser;

fn handle_esm_export_harmony_import_side_effects_dep(
  parser: &mut JavascriptParser,
  request: Atom,
  span: Span,
  source_span: Span,
  specifiers: Vec<Specifier>,
  dep_type: DependencyType,
  export_all: bool,
) {
  assert!(matches!(dep_type, DependencyType::EsmExport(_)));
  if !specifiers.is_empty() {
    specifiers.iter().for_each(|specifier| match specifier {
      Specifier::Namespace(n) => {
        let ids = vec![(n.clone(), None)];
        parser
          .dependencies
          .push(Box::new(HarmonyExportImportedSpecifierDependency::new(
            request.clone(),
            parser.last_harmony_import_order,
            ids,
            vec![],
            Some(n.clone()),
            false,
            None,
          )));
        parser.build_info.harmony_named_exports.insert(n.clone());
      }
      Specifier::Default(_) => {
        unreachable!()
      }
      Specifier::Named(orig, exported) => {
        let name = exported.clone().unwrap_or(orig.clone());
        let ids = vec![(name.clone(), Some(orig.clone()))];
        // TODO: add variable usage
        parser
          .dependencies
          .push(Box::new(HarmonyExportImportedSpecifierDependency::new(
            request.clone(),
            parser.last_harmony_import_order,
            ids.clone(),
            ids,
            Some(name.clone()),
            false,
            None,
          )));
        parser.build_info.harmony_named_exports.insert(name);
      }
    });
  }

  handle_harmony_import_side_effects_dep(
    parser,
    request,
    span,
    source_span,
    specifiers,
    dep_type,
    export_all,
  )
}
pub struct HarmonyExportDependencyParserPlugin;

impl JavascriptParserPlugin for HarmonyExportDependencyParserPlugin {
  fn named_export_import(
    &self,
    parser: &mut JavascriptParser,
    named_export: &NamedExport,
    _source: &str,
  ) -> Option<bool> {
    let Some(src) = &named_export.src else {
      unreachable!()
    };
    parser.last_harmony_import_order += 1;
    let mut specifiers = vec![];
    named_export
      .specifiers
      .iter()
      .for_each(|specifier| match specifier {
        ExportSpecifier::Namespace(n) if let ModuleExportName::Ident(export) = &n.name => {
          specifiers.push(Specifier::Namespace(export.sym.clone()));
        }
        ExportSpecifier::Named(named) if let ModuleExportName::Ident(orig) = &named.orig => {
          specifiers.push(Specifier::Named(
            orig.sym.clone(),
            match &named.exported {
              Some(ModuleExportName::Str(export)) => Some(export.value.clone()),
              Some(ModuleExportName::Ident(export)) => Some(export.sym.clone()),
              None => None,
            },
          ));
        }
        ExportSpecifier::Default(_) => {
          // export a from "./a"; is a syntax error
          unreachable!()
        }
        _ => {}
      });

    handle_esm_export_harmony_import_side_effects_dep(
      parser,
      src.value.clone(),
      named_export.span,
      src.span,
      specifiers,
      DependencyType::EsmExport(named_export.span.into()),
      false,
    );

    parser
      .presentational_dependencies
      .push(Box::new(ConstDependency::new(
        named_export.span.real_lo(),
        named_export.span.real_hi(),
        "".into(),
        None,
      )));
    Some(true)
  }

  fn all_export_import(
    &self,
    parser: &mut JavascriptParser,
    export_all: &ExportAll,
    _source: &str,
  ) -> Option<bool> {
    parser.last_harmony_import_order += 1;

    handle_esm_export_harmony_import_side_effects_dep(
      parser,
      export_all.src.value.clone(),
      export_all.span,
      export_all.src.span,
      vec![],
      DependencyType::EsmExport(export_all.span.into()),
      true,
    );

    let list = Some(parser.build_info.all_star_exports.clone());
    let export_imported_dep = HarmonyExportImportedSpecifierDependency::new(
      export_all.src.value.clone(),
      parser.last_harmony_import_order,
      vec![],
      vec![],
      None,
      true,
      list,
    );

    parser
      .build_info
      .all_star_exports
      .push(export_imported_dep.id);
    parser.dependencies.push(Box::new(export_imported_dep));

    parser
      .presentational_dependencies
      .push(Box::new(ConstDependency::new(
        export_all.span.real_lo(),
        export_all.span.real_hi(),
        "".into(),
        None,
      )));
    Some(true)
  }
}
