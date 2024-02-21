use rspack_core::{
  tree_shaking::symbol::DEFAULT_JS_WORD, ConstDependency, DependencyType, SpanExt,
};
use swc_core::ecma::ast::{ExportSpecifier, ImportSpecifier, ModuleExportName};

use super::JavascriptParserPlugin;
use crate::dependency::Specifier;
use crate::visitors::harmony_import_dependency_scanner::{ImporterInfo, ImporterReferenceInfo};
use crate::visitors::JavascriptParser;

pub struct HarmonyImportDependencyParserPlugin;

impl JavascriptParserPlugin for HarmonyImportDependencyParserPlugin {
  fn import(
    &self,
    parser: &mut crate::visitors::JavascriptParser,
    import_decl: &swc_core::ecma::ast::ImportDecl,
    _source: &str,
  ) -> Option<bool> {
    parser.last_harmony_import_order += 1;
    let mut specifiers = vec![];
    import_decl.specifiers.iter().for_each(|s| match s {
      ImportSpecifier::Named(n) => {
        let specifier = Specifier::Named(
          n.local.sym.clone(),
          match &n.imported {
            Some(ModuleExportName::Ident(ident)) => Some(ident.sym.clone()),
            Some(ModuleExportName::Str(str)) => Some(str.value.clone()),
            None => None,
          },
        );
        parser.import_map.insert(
          n.local.to_id(),
          ImporterReferenceInfo::new(
            import_decl.src.value.clone(),
            specifier.clone(),
            Some(match &n.imported {
              Some(ModuleExportName::Ident(ident)) => ident.sym.clone(),
              Some(ModuleExportName::Str(str)) => str.value.clone(),
              None => n.local.sym.clone(),
            }),
            parser.last_harmony_import_order,
          ),
        );

        specifiers.push(specifier);
      }
      ImportSpecifier::Default(d) => {
        let specifier = Specifier::Default(d.local.sym.clone());
        parser.import_map.insert(
          d.local.to_id(),
          ImporterReferenceInfo::new(
            import_decl.src.value.clone(),
            specifier.clone(),
            Some(DEFAULT_JS_WORD.clone()),
            parser.last_harmony_import_order,
          ),
        );
        specifiers.push(specifier);
      }
      ImportSpecifier::Namespace(n) => {
        let specifier = Specifier::Namespace(n.local.sym.clone());
        parser.import_map.insert(
          n.local.to_id(),
          ImporterReferenceInfo::new(
            import_decl.src.value.clone(),
            specifier.clone(),
            None,
            parser.last_harmony_import_order,
          ),
        );
        specifiers.push(specifier);
      }
    });

    let key = (
      import_decl.src.value.clone(),
      DependencyType::EsmImport(import_decl.span.into()),
      parser.last_harmony_import_order,
    );
    if let Some(importer_info) = parser.imports.get_mut(&key) {
      importer_info.specifiers.extend(specifiers);
    } else {
      parser.imports.insert(
        key,
        ImporterInfo::new(import_decl.span, import_decl.src.span, specifiers, false),
      );
    }
    parser
      .presentational_dependencies
      .push(Box::new(ConstDependency::new(
        import_decl.span.real_lo(),
        import_decl.span.real_hi(),
        "".into(),
        None,
      )));
    Some(true)
  }

  fn named_export_import(
    &self,
    parser: &mut JavascriptParser,
    named_export: &swc_core::ecma::ast::NamedExport,
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
        ExportSpecifier::Namespace(n) => {
          if let ModuleExportName::Ident(export) = &n.name {
            specifiers.push(Specifier::Namespace(export.sym.clone()));
          }
        }
        ExportSpecifier::Default(_) => {
          // export a from "./a"; is a syntax error
          unreachable!()
        }
        ExportSpecifier::Named(named) => {
          if let ModuleExportName::Ident(orig) = &named.orig {
            specifiers.push(Specifier::Named(
              orig.sym.clone(),
              match &named.exported {
                Some(ModuleExportName::Str(export)) => Some(export.value.clone()),
                Some(ModuleExportName::Ident(export)) => Some(export.sym.clone()),
                None => None,
              },
            ));
          }
        }
      });
    let key = (
      src.value.clone(),
      DependencyType::EsmExport(named_export.span.into()),
      parser.last_harmony_import_order,
    );
    if let Some(importer_info) = parser.imports.get_mut(&key) {
      importer_info.specifiers.extend(specifiers);
    } else {
      parser.imports.insert(
        key,
        ImporterInfo::new(named_export.span, src.span, specifiers, false),
      );
    }
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
    export_all: &swc_core::ecma::ast::ExportAll,
    _source: &str,
  ) -> Option<bool> {
    parser.last_harmony_import_order += 1;
    let key = (
      export_all.src.value.clone(),
      DependencyType::EsmExport(export_all.span.into()),
      parser.last_harmony_import_order,
    );

    if let Some(importer_info) = parser.imports.get_mut(&key) {
      importer_info.exports_all = true;
    } else {
      parser.imports.insert(
        key,
        ImporterInfo::new(export_all.span, export_all.src.span, vec![], true),
      );
    }

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
