use indexmap::IndexMap;
use rspack_core::DependencyLocation;
use rspack_core::{extract_member_expression_chain, BoxDependency, DependencyType, SpanExt};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use swc_core::atoms::Atom;
use swc_core::common::Span;
use swc_core::ecma::ast::NamedExport;
use swc_core::ecma::ast::{AssignExpr, AssignOp, CallExpr, MemberExpr, NewExpr, OptChainExpr};
use swc_core::ecma::ast::{Expr, Id, TaggedTpl};
use swc_core::ecma::ast::{Ident, ImportDecl, Pat, PatOrExpr, Prop};
use swc_core::ecma::visit::{noop_visit_type, Visit, VisitWith};

use super::{collect_destructuring_assignment_properties, ExtraSpanInfo, JavascriptParser};
use crate::dependency::{
  HarmonyExportImportedSpecifierDependency, HarmonyImportSideEffectDependency,
  HarmonyImportSpecifierDependency, Specifier,
};
use crate::no_visit_ignored_stmt;

#[derive(Debug)]
pub struct ImporterReferenceInfo {
  pub request: Atom,
  pub specifier: Specifier,
  pub names: Option<Atom>,
  pub source_order: i32,
}

impl ImporterReferenceInfo {
  pub fn new(request: Atom, specifier: Specifier, names: Option<Atom>, source_order: i32) -> Self {
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
  pub source_span: Span,
  pub specifiers: Vec<Specifier>,
  pub exports_all: bool,
}

impl ImporterInfo {
  pub fn new(span: Span, source_span: Span, specifiers: Vec<Specifier>, exports_all: bool) -> Self {
    Self {
      span,
      source_span,
      specifiers,
      exports_all,
    }
  }
}

pub type Imports = IndexMap<(Atom, DependencyType, i32), ImporterInfo>;

pub fn handle_importer_info(parser: &mut JavascriptParser) {
  for ((request, dependency_type, source_order), importer_info) in
    std::mem::take(&mut parser.imports).into_iter()
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
            parser
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
                source_order,
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
    if importer_info.exports_all {
      let list = Some(parser.build_info.all_star_exports.clone());
      let export_imported_dep = HarmonyExportImportedSpecifierDependency::new(
        request.clone(),
        source_order,
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
    }
    let dependency = HarmonyImportSideEffectDependency::new(
      request.clone(),
      source_order,
      Some(importer_info.span.into()),
      Some(importer_info.source_span.into()),
      importer_info.specifiers,
      dependency_type,
      importer_info.exports_all,
    );
    parser.dependencies.push(Box::new(dependency));
  }
}

pub struct HarmonyImportRefDependencyScanner<'a> {
  pub enter_callee: bool,
  pub enter_new_expr: bool,
  pub import_map: &'a ImportMap,
  pub dependencies: &'a mut Vec<BoxDependency>,
  pub properties_in_destructuring: HashMap<Atom, HashSet<Atom>>,
  pub rewrite_usage_span: &'a mut HashMap<Span, ExtraSpanInfo>,
  pub ignored: &'a mut HashSet<DependencyLocation>,
}

impl<'a> HarmonyImportRefDependencyScanner<'a> {
  pub fn new(
    import_map: &'a ImportMap,
    dependencies: &'a mut Vec<BoxDependency>,
    rewrite_usage_span: &'a mut HashMap<Span, ExtraSpanInfo>,
    ignored: &'a mut HashSet<DependencyLocation>,
  ) -> Self {
    Self {
      import_map,
      dependencies,
      enter_callee: false,
      enter_new_expr: false,
      properties_in_destructuring: HashMap::default(),
      rewrite_usage_span,
      ignored,
    }
  }
}

impl Visit for HarmonyImportRefDependencyScanner<'_> {
  noop_visit_type!();
  no_visit_ignored_stmt!();

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
          self.enter_callee && !self.enter_new_expr,
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
          self.enter_callee && !self.enter_new_expr,
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
            self.enter_callee && !self.enter_new_expr,
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

  fn visit_call_expr(&mut self, call_expr: &CallExpr) {
    // every time into new call_expr, reset enter callee
    let old = self.enter_new_expr;

    self.enter_new_expr = false;
    self.enter_callee = true;
    call_expr.callee.visit_with(self);
    self.enter_new_expr = old;
    self.enter_callee = false;

    call_expr.args.visit_with(self);
  }

  fn visit_tagged_tpl(&mut self, n: &TaggedTpl) {
    // every time into new tagged tpl expr, reset enter callee
    let old = self.enter_new_expr;

    self.enter_new_expr = false;
    self.enter_callee = true;
    n.tag.visit_with(self);
    self.enter_new_expr = old;
    self.enter_callee = false;

    n.tpl.visit_with(self);
  }

  fn visit_import_decl(&mut self, _decl: &ImportDecl) {}

  fn visit_named_export(&mut self, _named_export: &NamedExport) {}

  fn visit_new_expr(&mut self, n: &NewExpr) {
    let old = self.enter_new_expr;
    self.enter_new_expr = true;
    n.callee.visit_with(self);
    self.enter_new_expr = old;

    n.args.visit_with(self);
  }
}

// #[cfg(test)]
// mod test {
//   use rspack_ast::javascript::Program;
//   use rspack_core::{BuildInfo, Dependency, ModuleType};
//   use rustc_hash::FxHashSet;

//   use super::HarmonyImportDependencyScanner;
//   use crate::{ast::parse, dependency::HarmonyImportSpecifierDependency};

//   fn scan_dependencies(program: &Program) -> Vec<Box<dyn Dependency>> {
//     let mut build_info = BuildInfo {
//       cacheable: false,
//       hash: None,
//       strict: true,
//       file_dependencies: Default::default(),
//       context_dependencies: Default::default(),
//       missing_dependencies: Default::default(),
//       build_dependencies: Default::default(),
//       asset_filenames: Default::default(),
//       harmony_named_exports: Default::default(),
//       all_star_exports: Default::default(),
//       need_create_require: false,
//       json_data: None,
//     };
//     let mut import_map = Default::default();
//     let mut deps = vec![];
//     let mut presentation_deps = vec![];
//     let mut rewrite_usage_span = Default::default();
//     let mut ignored = FxHashSet::default();
//     let mut scanner = HarmonyImportDependencyScanner::new(
//       &mut deps,
//       &mut presentation_deps,
//       &mut import_map,
//       &mut build_info,
//       &mut rewrite_usage_span,
//       &mut ignored,
//     );

//     program.visit_with(&mut scanner);

//     deps
//   }

//   #[test]
//   fn should_be_call() {
//     let ast = parse(
//       r#"
//       import { foo } from 'bar';
//       foo();
//       foo.bar();
//       foo``;
//       new (foo())();
//     "#
//       .into(),
//       swc_core::ecma::parser::Syntax::Es(Default::default()),
//       "",
//       &ModuleType::Js,
//     )
//     .unwrap();

//     let deps = scan_dependencies(&ast.0.into_program());

//     let specifiers = deps
//       .iter()
//       .filter_map(|dep| dep.downcast_ref::<HarmonyImportSpecifierDependency>())
//       .collect::<Vec<_>>();

//     assert_eq!(specifiers.len(), 4);
//     assert!(specifiers.iter().all(|d| d.call));
//   }

//   #[test]
//   fn should_not_be_call() {
//     let ast = parse(
//       r#"
//       import { foo } from 'bar';
//       new foo();
//       new foo().bar;
//       new foo.bar();
//       `${foo}`;
//       nested(foo).call(true);
//       nested(foo)``
//       (`${foo}`)``
//       (new foo()).bar();
//       new foo().bar();
//       new foo.a().bar();
//       new (foo.bar)().baz();
//     "#
//       .into(),
//       swc_core::ecma::parser::Syntax::Es(Default::default()),
//       "",
//       &ModuleType::Js,
//     )
//     .unwrap();

//     let deps = scan_dependencies(&ast.0.into_program());

//     let specifiers = deps
//       .iter()
//       .filter_map(|dep| dep.downcast_ref::<HarmonyImportSpecifierDependency>())
//       .collect::<Vec<_>>();

//     assert_eq!(specifiers.len(), 11);
//     assert!(specifiers.iter().all(|d| !d.call));
//   }
// }
