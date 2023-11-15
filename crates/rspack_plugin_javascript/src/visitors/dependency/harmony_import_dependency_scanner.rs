use indexmap::IndexMap;
use rspack_core::{
  extract_member_expression_chain, tree_shaking::symbol::DEFAULT_JS_WORD, BoxDependency,
  BoxDependencyTemplate, BuildInfo, ConstDependency, DependencyType, SpanExt,
};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use swc_core::atoms::JsWord;
use swc_core::common::Span;
use swc_core::ecma::ast::{AssignExpr, AssignOp, MemberExpr, OptChainExpr};
use swc_core::ecma::ast::{Callee, ExportAll, ExportSpecifier, Expr, Id, TaggedTpl};
use swc_core::ecma::ast::{Ident, ImportDecl, Pat, PatOrExpr, Program, Prop};
use swc_core::ecma::ast::{ImportSpecifier, ModuleExportName, NamedExport};
use swc_core::ecma::visit::{noop_visit_type, Visit, VisitWith};

use super::{collect_destructuring_assignment_properties, ExtraSpanInfo};
use crate::dependency::{
  HarmonyExportImportedSpecifierDependency, HarmonyImportSideEffectDependency,
  HarmonyImportSpecifierDependency, Specifier,
};

#[derive(Debug)]
pub struct ImporterReferenceInfo {
  pub request: JsWord,
  pub specifier: Specifier,
  pub names: Option<JsWord>,
  pub source_order: i32,
}

impl ImporterReferenceInfo {
  pub fn new(
    request: JsWord,
    specifier: Specifier,
    names: Option<JsWord>,
    source_order: i32,
  ) -> Self {
    Self {
      request,
      specifier,
      names,
      source_order,
    }
  }
}

pub type ImportMap = HashMap<Id, ImporterReferenceInfo>;

#[derive(Debug)]
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

pub type Imports = IndexMap<(JsWord, DependencyType, i32), ImporterInfo>;

pub struct HarmonyImportDependencyScanner<'a> {
  pub dependencies: &'a mut Vec<BoxDependency>,
  pub presentational_dependencies: &'a mut Vec<BoxDependencyTemplate>,
  pub import_map: &'a mut ImportMap,
  pub imports: Imports,
  pub build_info: &'a mut BuildInfo,
  pub rewrite_usage_span: &'a mut HashMap<Span, ExtraSpanInfo>,
  last_harmony_import_order: i32,
}

impl<'a> HarmonyImportDependencyScanner<'a> {
  pub fn new(
    dependencies: &'a mut Vec<BoxDependency>,
    presentational_dependencies: &'a mut Vec<BoxDependencyTemplate>,
    import_map: &'a mut ImportMap,
    build_info: &'a mut BuildInfo,
    rewrite_usage_span: &'a mut HashMap<Span, ExtraSpanInfo>,
  ) -> Self {
    Self {
      dependencies,
      presentational_dependencies,
      import_map,
      imports: Default::default(),
      build_info,
      rewrite_usage_span,
      last_harmony_import_order: 0,
    }
  }
}

impl Visit for HarmonyImportDependencyScanner<'_> {
  noop_visit_type!();

  fn visit_program(&mut self, program: &Program) {
    // collect import map info
    program.visit_children_with(self);
    for ((request, dependency_type, source_order), importer_info) in
      std::mem::take(&mut self.imports).into_iter()
    {
      if matches!(dependency_type, DependencyType::EsmExport(_))
        && !importer_info.specifiers.is_empty()
      {
        importer_info
          .specifiers
          .iter()
          .for_each(|specifier| match specifier {
            Specifier::Namespace(n) => {
              let ids = vec![(n.clone(), None)];
              self
                .dependencies
                .push(Box::new(HarmonyExportImportedSpecifierDependency::new(
                  request.clone(),
                  source_order,
                  ids,
                  vec![],
                  Some(n.clone()),
                  false,
                  None,
                )));
              self.build_info.harmony_named_exports.insert(n.clone());
            }
            Specifier::Default(_) => {
              unreachable!()
            }
            Specifier::Named(orig, exported) => {
              let name = exported.clone().unwrap_or(orig.clone());
              let ids = vec![(name.clone(), Some(orig.clone()))];
              // TODO: add variable usage
              self
                .dependencies
                .push(Box::new(HarmonyExportImportedSpecifierDependency::new(
                  request.clone(),
                  source_order,
                  ids.clone(),
                  ids,
                  Some(name.clone()),
                  false,
                  None,
                )));
              self.build_info.harmony_named_exports.insert(name);
            }
          });
      }
      if importer_info.exports_all {
        let list = Some(self.build_info.all_star_exports.clone());
        let export_imported_dep = HarmonyExportImportedSpecifierDependency::new(
          request.clone(),
          source_order,
          vec![],
          vec![],
          None,
          true,
          list,
        );

        self
          .build_info
          .all_star_exports
          .push(export_imported_dep.id);
        self.dependencies.push(Box::new(export_imported_dep));
      } else {
        let dependency = HarmonyImportSideEffectDependency::new(
          request.clone(),
          source_order,
          Some(importer_info.span.into()),
          importer_info.specifiers,
          dependency_type,
        );
        self.dependencies.push(Box::new(dependency));
      }
    }

    // collect import reference info
    program.visit_children_with(&mut HarmonyImportRefDependencyScanner::new(
      self.import_map,
      self.dependencies,
      self.rewrite_usage_span,
    ));
  }

  fn visit_import_decl(&mut self, import_decl: &ImportDecl) {
    self.last_harmony_import_order += 1;
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
        self.import_map.insert(
          n.local.to_id(),
          ImporterReferenceInfo::new(
            import_decl.src.value.clone(),
            specifier.clone(),
            Some(match &n.imported {
              Some(ModuleExportName::Ident(ident)) => ident.sym.clone(),
              Some(ModuleExportName::Str(str)) => str.value.clone(),
              None => n.local.sym.clone(),
            }),
            self.last_harmony_import_order,
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
            self.last_harmony_import_order,
          ),
        );
        specifiers.push(specifier);
      }
      ImportSpecifier::Namespace(n) => {
        let specifier = Specifier::Namespace(n.local.sym.clone());
        self.import_map.insert(
          n.local.to_id(),
          ImporterReferenceInfo::new(
            import_decl.src.value.clone(),
            specifier.clone(),
            None,
            self.last_harmony_import_order,
          ),
        );
        specifiers.push(specifier);
      }
    });

    let key = (
      import_decl.src.value.clone(),
      DependencyType::EsmImport(import_decl.span.into()),
      self.last_harmony_import_order,
    );
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
      self.last_harmony_import_order += 1;
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
        self.last_harmony_import_order,
      );
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
    self.last_harmony_import_order += 1;
    let key = (
      export_all.src.value.clone(),
      DependencyType::EsmExport(export_all.span.into()),
      self.last_harmony_import_order,
    );

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
  pub dependencies: &'a mut Vec<BoxDependency>,
  pub properties_in_destructuring: HashMap<JsWord, HashSet<JsWord>>,
  pub rewrite_usage_span: &'a mut HashMap<Span, ExtraSpanInfo>,
}

impl<'a> HarmonyImportRefDependencyScanner<'a> {
  pub fn new(
    import_map: &'a ImportMap,
    dependencies: &'a mut Vec<BoxDependency>,
    rewrite_usage_span: &'a mut HashMap<Span, ExtraSpanInfo>,
  ) -> Self {
    Self {
      import_map,
      dependencies,
      enter_callee: false,
      properties_in_destructuring: HashMap::default(),
      rewrite_usage_span,
    }
  }
}

impl Visit for HarmonyImportRefDependencyScanner<'_> {
  noop_visit_type!();

  // collect referenced properties in destructuring
  // import * as a from 'a';
  // const { value } = a;
  fn visit_assign_expr(&mut self, assign_expr: &AssignExpr) {
    if let PatOrExpr::Pat(box Pat::Object(object_pat)) = &assign_expr.left
      && assign_expr.op == AssignOp::Assign
      && let box Expr::Ident(ident) = &assign_expr.right
      && let Some(reference) = self.import_map.get(&ident.to_id())
      && matches!(reference.specifier, Specifier::Namespace(_))
    {
      if let Some(value) = collect_destructuring_assignment_properties(object_pat) {
        self
          .properties_in_destructuring
          .entry(ident.sym.clone())
          .and_modify(|v| v.extend(value.clone()))
          .or_insert(value);
      }
    }
    assign_expr.visit_children_with(self);
  }

  fn visit_prop(&mut self, n: &Prop) {
    match n {
      Prop::Shorthand(shorthand) => {
        self
          .rewrite_usage_span
          .insert(shorthand.span, ExtraSpanInfo::ReWriteUsedByExports);
        if let Some(reference) = self.import_map.get(&shorthand.to_id()) {
          self
            .dependencies
            .push(Box::new(HarmonyImportSpecifierDependency::new(
              reference.request.clone(),
              reference.source_order,
              true,
              shorthand.span.real_lo(),
              shorthand.span.real_hi(),
              reference.names.clone().map(|f| vec![f]).unwrap_or_default(),
              false,
              false,
              reference.specifier.clone(),
              None,
              shorthand.span,
            )));
        }
      }
      _ => n.visit_children_with(self),
    }
  }

  fn visit_ident(&mut self, ident: &Ident) {
    if let Some(reference) = self.import_map.get(&ident.to_id()) {
      self
        .rewrite_usage_span
        .insert(ident.span, ExtraSpanInfo::ReWriteUsedByExports);
      self
        .dependencies
        .push(Box::new(HarmonyImportSpecifierDependency::new(
          reference.request.clone(),
          reference.source_order,
          false,
          ident.span.real_lo(),
          ident.span.real_hi(),
          reference.names.clone().map(|f| vec![f]).unwrap_or_default(),
          self.enter_callee,
          true, // x()
          reference.specifier.clone(),
          self.properties_in_destructuring.remove(&ident.sym),
          ident.span,
        )));
    }
  }

  fn visit_opt_chain_expr(&mut self, opt_chain_expr: &OptChainExpr) {
    let expression_info = extract_member_expression_chain(opt_chain_expr);
    // dbg!(&expression_info);
    let member_chain = expression_info.members();
    if member_chain.len() > 1
      && let Some(reference) = self.import_map.get(&member_chain[0])
    {
      let mut non_optional_members = expression_info.non_optional_part();
      // dbg!(&non_optional_members);
      let start = opt_chain_expr.span.real_lo();
      let end = if !non_optional_members.is_empty()
        && let Some(span) = expression_info
          .members_spans()
          .get(non_optional_members.len() - 1)
      {
        span.real_hi()
      } else {
        opt_chain_expr.span.real_hi()
      };
      non_optional_members.pop_front();
      let mut ids = reference.names.clone().map(|f| vec![f]).unwrap_or_default();
      ids.extend_from_slice(
        &non_optional_members
          .into_iter()
          .map(|item| item.0.clone())
          .collect::<Vec<_>>(),
      );
      self
        .rewrite_usage_span
        .insert(opt_chain_expr.span, ExtraSpanInfo::ReWriteUsedByExports);
      self
        .dependencies
        .push(Box::new(HarmonyImportSpecifierDependency::new(
          reference.request.clone(),
          reference.source_order,
          false,
          start,
          end,
          ids,
          self.enter_callee,
          !self.enter_callee, // x.xx()
          reference.specifier.clone(),
          None,
          opt_chain_expr.span,
        )));
      return;
    }
    opt_chain_expr.visit_children_with(self);
  }

  fn visit_member_expr(&mut self, member_expr: &MemberExpr) {
    let expression_info = extract_member_expression_chain(member_expr);
    let member_chain = expression_info.members();
    if member_chain.len() > 1
      && let Some(reference) = self.import_map.get(&member_chain[0])
    {
      let mut member_chain = member_chain.clone();
      member_chain.pop_front();
      if !member_chain.is_empty() {
        let mut ids = reference.names.clone().map(|f| vec![f]).unwrap_or_default();
        // dbg!(&member_chain);
        ids.extend_from_slice(
          &member_chain
            .into_iter()
            .map(|item| item.0.clone())
            .collect::<Vec<_>>(),
        );
        // dbg!(&ids);
        self
          .rewrite_usage_span
          .insert(member_expr.span, ExtraSpanInfo::ReWriteUsedByExports);
        self
          .dependencies
          .push(Box::new(HarmonyImportSpecifierDependency::new(
            reference.request.clone(),
            reference.source_order,
            false,
            member_expr.span.real_lo(),
            member_expr.span.real_hi(),
            ids,
            self.enter_callee,
            !self.enter_callee, // x.xx()
            reference.specifier.clone(),
            None,
            member_expr.span,
          )));
        return;
      }
    }
    member_expr.visit_children_with(self);
  }

  fn visit_callee(&mut self, callee: &Callee) {
    self.enter_callee = true;
    callee.visit_children_with(self);
    self.enter_callee = false;
  }

  fn visit_tagged_tpl(&mut self, n: &TaggedTpl) {
    self.enter_callee = true;
    n.visit_children_with(self);
    self.enter_callee = false;
  }

  fn visit_import_decl(&mut self, _decl: &ImportDecl) {}

  fn visit_named_export(&mut self, _named_export: &NamedExport) {}
}
