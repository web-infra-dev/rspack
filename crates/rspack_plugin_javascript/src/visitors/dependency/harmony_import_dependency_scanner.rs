use rspack_core::{CodeReplaceSourceDependency, ModuleDependency, ReplaceConstDependency, SpanExt};
use rustc_hash::FxHashMap;
use swc_core::{
  common::{Span, SyntaxContext},
  ecma::{
    ast::{
      ExportSpecifier, Ident, ImportDecl, ImportSpecifier, ModuleExportName, NamedExport, Program,
      Prop,
    },
    atoms::JsWord,
    visit::{noop_visit_type, Visit, VisitWith},
  },
};

pub type ImportMap = FxHashMap<(JsWord, SyntaxContext), Option<(JsWord, Option<String>)>>;

use crate::dependency::{HarmonyImportDependency, HarmonyImportSpecifierDependency};

pub struct HarmonyImportDependencyScanner<'a> {
  pub dependencies: &'a mut Vec<Box<dyn ModuleDependency>>,
  pub code_generable_dependencies: &'a mut Vec<Box<dyn CodeReplaceSourceDependency>>,
  pub import_map: &'a mut ImportMap,
  pub span_map: FxHashMap<JsWord, Span>,
}

impl<'a> HarmonyImportDependencyScanner<'a> {
  pub fn new(
    dependencies: &'a mut Vec<Box<dyn ModuleDependency>>,
    code_generable_dependencies: &'a mut Vec<Box<dyn CodeReplaceSourceDependency>>,
    import_map: &'a mut ImportMap,
  ) -> Self {
    Self {
      dependencies,
      code_generable_dependencies,
      import_map,
      span_map: Default::default(),
    }
  }
}

impl Visit for HarmonyImportDependencyScanner<'_> {
  noop_visit_type!();

  fn visit_program(&mut self, program: &Program) {
    // collect import map info
    program.visit_children_with(self);
    // collect import reference info
    let mut ref_dependencies = Default::default();
    program.visit_children_with(&mut HarmonyImportRefDependencyScanner::new(
      self.import_map,
      &mut ref_dependencies,
    ));

    for (request, span) in std::mem::take(&mut self.span_map) {
      let refs = ref_dependencies.remove(&request).unwrap_or_default();
      self
        .dependencies
        .push(Box::new(HarmonyImportDependency::new(
          request,
          Some(span.into()),
          refs,
        )));
    }
  }

  fn visit_import_decl(&mut self, import_decl: &ImportDecl) {
    import_decl.specifiers.iter().for_each(|s| match s {
      ImportSpecifier::Named(n) => {
        self.import_map.insert(
          (n.local.sym.clone(), n.local.span.ctxt),
          match &n.imported {
            Some(ModuleExportName::Ident(ident)) => {
              Some((import_decl.src.value.clone(), Some(ident.sym.to_string())))
            }
            _ => Some((import_decl.src.value.clone(), Some(n.local.sym.to_string()))),
          },
        );
      }
      ImportSpecifier::Default(d) => {
        self.import_map.insert(
          (d.local.sym.clone(), d.local.span.ctxt),
          Some((import_decl.src.value.clone(), Some("default".to_string()))),
        );
      }
      ImportSpecifier::Namespace(n) => {
        self.import_map.insert(
          (n.local.sym.clone(), n.local.span.ctxt),
          Some((import_decl.src.value.clone(), None)),
        );
      }
    });

    if !self.span_map.contains_key(&import_decl.src.value) {
      self
        .span_map
        .insert(import_decl.src.value.clone(), import_decl.span);
    }

    self
      .code_generable_dependencies
      .push(Box::new(ReplaceConstDependency::new(
        import_decl.span.real_lo(),
        import_decl.span.real_hi(),
        "".into(),
        None,
      )));
  }
}

pub struct HarmonyImportRefDependencyScanner<'a> {
  pub import_map: &'a ImportMap,
  pub ref_dependencies: &'a mut FxHashMap<JsWord, Vec<HarmonyImportSpecifierDependency>>,
}

impl<'a> HarmonyImportRefDependencyScanner<'a> {
  pub fn new(
    import_map: &'a ImportMap,
    ref_dependencies: &'a mut FxHashMap<JsWord, Vec<HarmonyImportSpecifierDependency>>,
  ) -> Self {
    Self {
      import_map,
      ref_dependencies,
    }
  }
}

impl Visit for HarmonyImportRefDependencyScanner<'_> {
  noop_visit_type!();

  fn visit_prop(&mut self, n: &Prop) {
    match n {
      Prop::Shorthand(shorthand) => {
        if let Some(Some(reference)) = self.import_map.get(&shorthand.to_id()) {
          self
            .ref_dependencies
            .entry(reference.0.clone())
            .and_modify(|d| {
              d.push(HarmonyImportSpecifierDependency::new(
                true,
                shorthand.span.real_lo(),
                shorthand.span.real_hi(),
                reference.1.clone(),
                None,
              ))
            })
            .or_default();
        }
      }
      _ => n.visit_children_with(self),
    }
  }

  fn visit_ident(&mut self, ident: &Ident) {
    if let Some(Some(reference)) = self.import_map.get(&ident.to_id()) {
      self
        .ref_dependencies
        .entry(reference.0.clone())
        .or_insert(vec![])
        .push(HarmonyImportSpecifierDependency::new(
          false,
          ident.span.real_lo(),
          ident.span.real_hi(),
          reference.1.clone(),
          None,
        ));
    }
  }

  fn visit_import_decl(&mut self, _decl: &ImportDecl) {}

  fn visit_named_export(&mut self, named_export: &NamedExport) {
    if named_export.src.is_none() {
      named_export
        .specifiers
        .iter()
        .for_each(|specifier| match specifier {
          ExportSpecifier::Namespace(_ns) => {
            unreachable!()
          }
          ExportSpecifier::Default(_default) => {
            unreachable!();
          }
          ExportSpecifier::Named(named) => {
            if let ModuleExportName::Ident(orig) = &named.orig {
              if let Some(Some(reference)) = self.import_map.get(&orig.to_id()) {
                self
                  .ref_dependencies
                  .entry(reference.0.clone())
                  .or_insert(vec![])
                  .push(HarmonyImportSpecifierDependency::new(
                    false,
                    named.span.real_lo(),
                    named.span.real_hi(),
                    reference.1.clone(),
                    Some(match &named.exported {
                      Some(ModuleExportName::Ident(export)) => export.sym.to_string(),
                      None => orig.sym.to_string(),
                      _ => unreachable!(),
                    }),
                  ));
              }
            }
          }
        });
    }
  }
}
