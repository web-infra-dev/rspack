use rspack_core::tree_shaking::symbol::DEFAULT_JS_WORD;
use rspack_core::{ConstDependency, DependencyType, SpanExt};
use swc_core::atoms::Atom;
use swc_core::common::Span;
use swc_core::ecma::ast::{ImportDecl, ImportSpecifier, ModuleExportName};

use super::JavascriptParserPlugin;
use crate::dependency::{HarmonyImportSideEffectDependency, Specifier};
use crate::visitors::harmony_import_dependency_scanner::ImporterReferenceInfo;
use crate::visitors::JavascriptParser;

pub(super) fn handle_harmony_import_side_effects_dep(
  parser: &mut JavascriptParser,
  request: Atom,
  span: Span,
  source_span: Span,
  specifiers: Vec<Specifier>,
  dep_type: DependencyType,
  exports_all: bool,
) {
  let dependency = HarmonyImportSideEffectDependency::new(
    request,
    parser.last_harmony_import_order,
    Some(span.into()),
    Some(source_span.into()),
    specifiers,
    dep_type,
    exports_all,
  );
  parser.dependencies.push(Box::new(dependency));
}

pub struct HarmonyImportDependencyParserPlugin;

impl JavascriptParserPlugin for HarmonyImportDependencyParserPlugin {
  fn import(
    &self,
    parser: &mut JavascriptParser,
    import_decl: &ImportDecl,
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

    handle_harmony_import_side_effects_dep(
      parser,
      import_decl.src.value.clone(),
      import_decl.span,
      import_decl.src.span,
      specifiers,
      DependencyType::EsmImport(import_decl.span.into()),
      false,
    );

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
}
