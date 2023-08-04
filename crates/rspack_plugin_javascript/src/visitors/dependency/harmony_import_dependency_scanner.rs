use indexmap::IndexMap;
use rspack_core::{
  tree_shaking::symbol::DEFAULT_JS_WORD, ConstDependency, DependencyTemplate, DependencyType,
  ModuleDependency, SpanExt,
};
use rustc_hash::FxHashMap;
use swc_core::{
  common::Span,
  ecma::{
    ast::{
      Callee, ExportAll, ExportSpecifier, Expr, Id, Ident, ImportDecl, ImportSpecifier, Lit,
      MemberExpr, MemberProp, ModuleExportName, NamedExport, Program, Prop,
    },
    atoms::JsWord,
    visit::{noop_visit_type, Visit, VisitWith},
  },
};

use crate::dependency::{
  HarmonyExportImportedSpecifierDependency, HarmonyImportDependency,
  HarmonyImportSpecifierDependency, Specifier,
};

pub struct ImporterReferenceInfo {
  pub request: JsWord,
  pub specifier: Specifier,
  pub names: Option<JsWord>,
}

impl ImporterReferenceInfo {
  pub fn new(request: JsWord, specifier: Specifier, names: Option<JsWord>) -> Self {
    Self {
      request,
      specifier,
      names,
    }
  }
}

pub type ImportMap = FxHashMap<Id, ImporterReferenceInfo>;

pub struct ImporterInfo {
  pub span: Span,
  pub specifiers: Vec<Specifier>,
  pub exports_all: bool,
}

impl ImporterInfo {
  pub fn new(span: Span, specifiers: Vec<Specifier>, exports_all: bool) -> Self {
    Self {
      span,
      specifiers,
      exports_all,
    }
  }
}

pub type Imports = IndexMap<(JsWord, DependencyType), ImporterInfo>;

pub struct HarmonyImportDependencyScanner<'a> {
  pub dependencies: &'a mut Vec<Box<dyn ModuleDependency>>,
  pub presentational_dependencies: &'a mut Vec<Box<dyn DependencyTemplate>>,
  pub import_map: &'a mut ImportMap,
  pub imports: Imports,
}

impl<'a> HarmonyImportDependencyScanner<'a> {
  pub fn new(
    dependencies: &'a mut Vec<Box<dyn ModuleDependency>>,
    presentational_dependencies: &'a mut Vec<Box<dyn DependencyTemplate>>,
    import_map: &'a mut ImportMap,
  ) -> Self {
    Self {
      dependencies,
      presentational_dependencies,
      import_map,
      imports: Default::default(),
    }
  }
}

impl Visit for HarmonyImportDependencyScanner<'_> {
  noop_visit_type!();

  fn visit_program(&mut self, program: &Program) {
    // collect import map info
    program.visit_children_with(self);

    for ((request, dependency_type), importer_info) in std::mem::take(&mut self.imports).into_iter()
    {
      if matches!(dependency_type, DependencyType::EsmExport)
        && !importer_info.specifiers.is_empty()
      {
        let ids = importer_info
          .specifiers
          .iter()
          .map(|specifier| match specifier {
            Specifier::Namespace(n) => (n.clone(), None),
            Specifier::Default(_) => {
              unreachable!()
            }
            Specifier::Named(orig, exported) => {
              (exported.clone().unwrap_or(orig.clone()), Some(orig.clone()))
            }
          })
          .collect::<Vec<_>>();
        self
          .dependencies
          .push(Box::new(HarmonyExportImportedSpecifierDependency::new(
            request.clone(),
            ids,
          )));
      }
      self
        .dependencies
        .push(Box::new(HarmonyImportDependency::new(
          request,
          Some(importer_info.span.into()),
          importer_info.specifiers,
          dependency_type,
          importer_info.exports_all,
        )));
    }

    // collect import reference info
    program.visit_children_with(&mut HarmonyImportRefDependencyScanner::new(
      self.import_map,
      self.dependencies,
    ));
  }

  fn visit_import_decl(&mut self, import_decl: &ImportDecl) {
    let mut specifiers = vec![];
    import_decl.specifiers.iter().for_each(|s| match s {
      ImportSpecifier::Named(n) => {
        let specifier = Specifier::Named(
          n.local.sym.clone(),
          match &n.imported {
            Some(ModuleExportName::Ident(ident)) => Some(ident.sym.clone()),
            _ => None,
          },
        );
        self.import_map.insert(
          n.local.to_id(),
          ImporterReferenceInfo::new(
            import_decl.src.value.clone(),
            specifier.clone(),
            Some(match &n.imported {
              Some(ModuleExportName::Ident(ident)) => ident.sym.clone(),
              _ => n.local.sym.clone(),
            }),
          ),
        );

        specifiers.push(specifier);
      }
      ImportSpecifier::Default(d) => {
        let specifier = Specifier::Default(d.local.sym.clone());
        self.import_map.insert(
          d.local.to_id(),
          ImporterReferenceInfo::new(
            import_decl.src.value.clone(),
            specifier.clone(),
            Some(DEFAULT_JS_WORD.clone()),
          ),
        );
        specifiers.push(specifier);
      }
      ImportSpecifier::Namespace(n) => {
        let specifier = Specifier::Namespace(n.local.sym.clone());
        self.import_map.insert(
          n.local.to_id(),
          ImporterReferenceInfo::new(import_decl.src.value.clone(), specifier.clone(), None),
        );
        specifiers.push(specifier);
      }
    });

    let key = (import_decl.src.value.clone(), DependencyType::EsmImport);
    if let Some(importer_info) = self.imports.get_mut(&key) {
      importer_info.specifiers.extend(specifiers);
    } else {
      self
        .imports
        .insert(key, ImporterInfo::new(import_decl.span, specifiers, false));
    }
    self
      .presentational_dependencies
      .push(Box::new(ConstDependency::new(
        import_decl.span.real_lo(),
        import_decl.span.real_hi(),
        "".into(),
        None,
      )));
  }

  fn visit_named_export(&mut self, named_export: &NamedExport) {
    if let Some(src) = &named_export.src {
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
            unreachable!()
          }
          ExportSpecifier::Named(named) => {
            if let ModuleExportName::Ident(orig) = &named.orig {
              specifiers.push(Specifier::Named(
                orig.sym.clone(),
                match &named.exported {
                  Some(ModuleExportName::Ident(export)) => Some(export.sym.clone()),
                  None => None,
                  _ => unreachable!(),
                },
              ));
            }
          }
        });
      let key = (src.value.clone(), DependencyType::EsmExport);
      if let Some(importer_info) = self.imports.get_mut(&key) {
        importer_info.specifiers.extend(specifiers);
      } else {
        self
          .imports
          .insert(key, ImporterInfo::new(named_export.span, specifiers, false));
      }
      self
        .presentational_dependencies
        .push(Box::new(ConstDependency::new(
          named_export.span.real_lo(),
          named_export.span.real_hi(),
          "".into(),
          None,
        )));
    }
  }

  fn visit_export_all(&mut self, export_all: &ExportAll) {
    let key = (export_all.src.value.clone(), DependencyType::EsmExport);
    if let Some(importer_info) = self.imports.get_mut(&key) {
      importer_info.exports_all = true;
    } else {
      self
        .imports
        .insert(key, ImporterInfo::new(export_all.span, vec![], true));
    }
    self
      .presentational_dependencies
      .push(Box::new(ConstDependency::new(
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
  pub dependencies: &'a mut Vec<Box<dyn ModuleDependency>>,
}

impl<'a> HarmonyImportRefDependencyScanner<'a> {
  pub fn new(
    import_map: &'a ImportMap,
    dependencies: &'a mut Vec<Box<dyn ModuleDependency>>,
  ) -> Self {
    Self {
      import_map,
      dependencies,
      enter_callee: false,
    }
  }
}

impl Visit for HarmonyImportRefDependencyScanner<'_> {
  noop_visit_type!();

  fn visit_prop(&mut self, n: &Prop) {
    match n {
      Prop::Shorthand(shorthand) => {
        if let Some(reference) = self.import_map.get(&shorthand.to_id()) {
          self
            .dependencies
            .push(Box::new(HarmonyImportSpecifierDependency::new(
              reference.request.clone(),
              true,
              shorthand.span.real_lo(),
              shorthand.span.real_hi(),
              reference.names.clone().map(|f| vec![f]).unwrap_or_default(),
              false,
              false,
              reference.specifier.clone(),
            )));
        }
      }
      _ => n.visit_children_with(self),
    }
  }

  fn visit_ident(&mut self, ident: &Ident) {
    if let Some(reference) = self.import_map.get(&ident.to_id()) {
      self
        .dependencies
        .push(Box::new(HarmonyImportSpecifierDependency::new(
          reference.request.clone(),
          false,
          ident.span.real_lo(),
          ident.span.real_hi(),
          reference.names.clone().map(|f| vec![f]).unwrap_or_default(),
          self.enter_callee,
          true, // x()
          reference.specifier.clone(),
        )));
    }
  }

  fn visit_member_expr(&mut self, member_expr: &MemberExpr) {
    if let Expr::Ident(ident) = &*member_expr.obj {
      if let Some(reference) = self.import_map.get(&ident.to_id()) {
        let prop = match &member_expr.prop {
          MemberProp::Ident(ident) => Some(ident.sym.clone()),
          MemberProp::Computed(c) => {
            if let Expr::Lit(Lit::Str(str)) = &*c.expr {
              Some(str.value.clone())
            } else {
              None
            }
          }
          _ => None,
        };
        if let Some(prop) = prop {
          let mut ids = reference.names.clone().map(|f| vec![f]).unwrap_or_default();
          ids.push(prop);
          self
            .dependencies
            .push(Box::new(HarmonyImportSpecifierDependency::new(
              reference.request.clone(),
              false,
              member_expr.span.real_lo(),
              member_expr.span.real_hi(),
              ids,
              self.enter_callee,
              if self.enter_callee { false } else { true }, // x.xx()
              reference.specifier.clone(),
            )));
          return;
        }
      }
    }
    member_expr.visit_children_with(self);
  }

  fn visit_callee(&mut self, callee: &Callee) {
    self.enter_callee = true;
    callee.visit_children_with(self);
    self.enter_callee = false;
  }

  fn visit_import_decl(&mut self, _decl: &ImportDecl) {}

  fn visit_named_export(&mut self, _named_export: &NamedExport) {}
}
