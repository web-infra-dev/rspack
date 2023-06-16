use indexmap::IndexMap;
use rspack_core::{
  CodeReplaceSourceDependency, DependencyType, ModuleDependency, ModuleIdentifier,
  ReplaceConstDependency, SpanExt,
};
use rspack_symbol::DEFAULT_JS_WORD;
use rustc_hash::FxHashMap;
use swc_core::{
  common::{Span, SyntaxContext},
  ecma::{
    ast::{
      CallExpr, Callee, ExportAll, ExportSpecifier, Ident, ImportDecl, ImportSpecifier,
      ModuleExportName, NamedExport, Program, Prop,
    },
    atoms::JsWord,
    visit::{noop_visit_type, Visit, VisitWith},
  },
};

pub type ImportMap = FxHashMap<(JsWord, SyntaxContext), Option<(JsWord, Option<JsWord>)>>;

use crate::dependency::{
  HarmonyExportImportedSpecifierDependency, HarmonyImportDependency,
  HarmonyImportSpecifierDependency,
};

pub type Imports = IndexMap<(JsWord, DependencyType), (Span, Vec<(JsWord, Option<JsWord>)>, bool)>;

pub struct HarmonyImportDependencyScanner<'a> {
  pub dependencies: &'a mut Vec<Box<dyn ModuleDependency>>,
  pub code_generable_dependencies: &'a mut Vec<Box<dyn CodeReplaceSourceDependency>>,
  pub import_map: &'a mut ImportMap,
  pub imports: Imports,
  pub module_identifier: ModuleIdentifier,
}

impl<'a> HarmonyImportDependencyScanner<'a> {
  pub fn new(
    dependencies: &'a mut Vec<Box<dyn ModuleDependency>>,
    code_generable_dependencies: &'a mut Vec<Box<dyn CodeReplaceSourceDependency>>,
    import_map: &'a mut ImportMap,
    module_identifier: ModuleIdentifier,
  ) -> Self {
    Self {
      dependencies,
      code_generable_dependencies,
      import_map,
      imports: Default::default(),
      module_identifier,
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

    for ((request, dependency_type), (span, specifiers, exports_all)) in
      std::mem::take(&mut self.imports).into_iter()
    {
      let refs = if matches!(dependency_type, DependencyType::EsmImport) {
        ref_dependencies.remove(&request).unwrap_or_default()
      } else {
        vec![]
      };
      self
        .dependencies
        .push(Box::new(HarmonyImportDependency::new(
          request,
          Some(span.into()),
          refs,
          specifiers,
          dependency_type,
          exports_all,
        )));
    }
  }

  fn visit_import_decl(&mut self, import_decl: &ImportDecl) {
    let mut specifiers = vec![];
    import_decl.specifiers.iter().for_each(|s| match s {
      ImportSpecifier::Named(n) => {
        self.import_map.insert(
          (n.local.sym.clone(), n.local.span.ctxt),
          Some((
            import_decl.src.value.clone(),
            Some(match &n.imported {
              Some(ModuleExportName::Ident(ident)) => ident.sym.clone(),
              _ => n.local.sym.clone(),
            }),
          )),
        );
        specifiers.push((
          n.local.sym.clone(),
          match &n.imported {
            Some(ModuleExportName::Ident(ident)) => Some(ident.sym.clone()),
            _ => None,
          },
        ));
      }
      ImportSpecifier::Default(d) => {
        self.import_map.insert(
          (d.local.sym.clone(), d.local.span.ctxt),
          Some((import_decl.src.value.clone(), Some(DEFAULT_JS_WORD.clone()))),
        );
        specifiers.push((d.local.sym.clone(), Some(DEFAULT_JS_WORD.clone())));
      }
      ImportSpecifier::Namespace(n) => {
        self.import_map.insert(
          (n.local.sym.clone(), n.local.span.ctxt),
          Some((import_decl.src.value.clone(), None)),
        );
        specifiers.push((n.local.sym.clone(), Some("namespace".into())));
      }
    });

    let key = (import_decl.src.value.clone(), DependencyType::EsmImport);
    if let Some((_, s, _)) = self.imports.get_mut(&key) {
      s.extend(specifiers);
    } else {
      self
        .imports
        .insert(key, (import_decl.span, specifiers, false));
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

  fn visit_named_export(&mut self, named_export: &NamedExport) {
    if let Some(src) = &named_export.src {
      let mut ids = vec![];
      let mut specifiers = vec![];
      named_export
        .specifiers
        .iter()
        .for_each(|specifier| match specifier {
          ExportSpecifier::Namespace(n) => {
            if let ModuleExportName::Ident(export) = &n.name {
              ids.push((export.sym.clone(), None));
              specifiers.push((export.sym.clone(), Some("namespace".into())));
            }
          }
          ExportSpecifier::Default(_) => unreachable!(),
          ExportSpecifier::Named(named) => {
            if let ModuleExportName::Ident(orig) = &named.orig {
              let exported = match &named.exported {
                Some(ModuleExportName::Ident(export)) => export.sym.clone(),
                None => orig.sym.clone(),
                _ => unreachable!(),
              };
              ids.push((exported, Some(orig.sym.clone())));
              specifiers.push((
                orig.sym.clone(),
                match &named.exported {
                  Some(ModuleExportName::Ident(export)) => Some(export.sym.clone()),
                  None => None,
                  _ => unreachable!(),
                },
              ))
            }
          }
        });

      self.code_generable_dependencies.push(Box::new(
        HarmonyExportImportedSpecifierDependency::new(
          src.value.clone(),
          ids,
          self.module_identifier,
        ),
      ));
      let key = (src.value.clone(), DependencyType::EsmExport);
      if let Some((_, s, _)) = self.imports.get_mut(&key) {
        s.extend(specifiers);
      } else {
        self
          .imports
          .insert(key, (named_export.span, specifiers, false));
      }
      self
        .code_generable_dependencies
        .push(Box::new(ReplaceConstDependency::new(
          named_export.span.real_lo(),
          named_export.span.real_hi(),
          "".into(),
          None,
        )));
    }
  }

  fn visit_export_all(&mut self, export_all: &ExportAll) {
    let key = (export_all.src.value.clone(), DependencyType::EsmExport);
    if let Some((_, _, exports_all)) = self.imports.get_mut(&key) {
      *exports_all = true;
    } else {
      self.imports.insert(key, (export_all.span, vec![], true));
    }
    self
      .code_generable_dependencies
      .push(Box::new(ReplaceConstDependency::new(
        export_all.span.real_lo(),
        export_all.span.real_hi(),
        "".into(),
        None,
      )));
  }
}

pub struct HarmonyImportRefDependencyScanner<'a> {
  pub enter_callee: bool,
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
      enter_callee: false,
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
            .or_insert(vec![])
            .push(HarmonyImportSpecifierDependency::new(
              true,
              shorthand.span.real_lo(),
              shorthand.span.real_hi(),
              reference.1.clone(),
              false,
            ));
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
          self.enter_callee,
        ));
    }
  }

  fn visit_callee(&mut self, callee: &Callee) {
    self.enter_callee = true;
    callee.visit_children_with(self);
    self.enter_callee = false;
  }

  fn visit_import_decl(&mut self, _decl: &ImportDecl) {}

  fn visit_named_export(&mut self, _named_export: &NamedExport) {}
}
