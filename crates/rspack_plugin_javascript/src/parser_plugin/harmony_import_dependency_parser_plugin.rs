use rspack_core::tree_shaking::symbol::DEFAULT_JS_WORD;
use rspack_core::{extract_member_expression_chain, ConstDependency, DependencyType, SpanExt};
use swc_core::atoms::Atom;
use swc_core::common::Span;
use swc_core::ecma::ast::{AssignExpr, AssignOp, ImportSpecifier, ModuleExportName, OptChainExpr};
use swc_core::ecma::ast::{Expr, Ident, ImportDecl, Pat, PatOrExpr};

use super::JavascriptParserPlugin;
use crate::dependency::{
  HarmonyImportSideEffectDependency, HarmonyImportSpecifierDependency, Specifier,
};
use crate::visitors::{collect_destructuring_assignment_properties, JavascriptParser, TagInfoData};
use crate::visitors::{ExtraSpanInfo, ImporterReferenceInfo};

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

const HARMONY_SPECIFIER_TAG: &str = "_identifier__harmony_specifier_tag__";

#[derive(serde::Deserialize, serde::Serialize, Clone)]
struct MockData;

impl TagInfoData for MockData {
  fn serialize(_: &Self) -> serde_json::Value {
    serde_json::Value::Null
  }

  fn deserialize(_: serde_json::Value) -> Self {
    MockData
  }
}

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

  fn import_specifier(
    &self,
    parser: &mut JavascriptParser,
    _statement: &ImportDecl,
    _source: &Atom,
    _export_name: Option<&str>,
    identifier_name: &str,
  ) -> Option<bool> {
    // TODO: fill data with `Some({name, source, ids, source_order, assertions })`
    parser.tag_variable::<MockData>(identifier_name.to_string(), HARMONY_SPECIFIER_TAG, None);
    Some(true)
  }

  fn identifier(
    &self,
    parser: &mut JavascriptParser,
    ident: &Ident,
    _for_name: &str,
  ) -> Option<bool> {
    if parser.in_short_hand {
      parser
        .rewrite_usage_span
        .insert(ident.span, ExtraSpanInfo::ReWriteUsedByExports);
      if let Some(reference) = parser.import_map.get(&ident.to_id()) {
        parser
          .dependencies
          .push(Box::new(HarmonyImportSpecifierDependency::new(
            reference.request.clone(),
            reference.source_order,
            true,
            ident.span.real_lo(),
            ident.span.real_hi(),
            reference.names.clone().map(|f| vec![f]).unwrap_or_default(),
            false,
            false,
            reference.specifier.clone(),
            None,
            ident.span,
          )));
      }
      Some(true)
    } else if let Some(reference) = parser.import_map.get(&ident.to_id()) {
      parser
        .rewrite_usage_span
        .insert(ident.span, ExtraSpanInfo::ReWriteUsedByExports);
      parser
        .dependencies
        .push(Box::new(HarmonyImportSpecifierDependency::new(
          reference.request.clone(),
          reference.source_order,
          false,
          ident.span.real_lo(),
          ident.span.real_hi(),
          reference.names.clone().map(|f| vec![f]).unwrap_or_default(),
          parser.enter_callee && !parser.enter_new_expr,
          true, // x()
          reference.specifier.clone(),
          parser.properties_in_destructuring.remove(&ident.sym),
          ident.span,
        )));
      Some(true)
    } else {
      None
    }
  }

  fn member(
    &self,
    parser: &mut JavascriptParser,
    member_expr: &swc_core::ecma::ast::MemberExpr,
    _for_name: &str,
  ) -> Option<bool> {
    let expression_info = extract_member_expression_chain(member_expr);
    let member_chain = expression_info.members();
    if member_chain.len() > 1
      && let Some(reference) = parser.import_map.get(&member_chain[0])
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
        parser
          .rewrite_usage_span
          .insert(member_expr.span, ExtraSpanInfo::ReWriteUsedByExports);
        parser
          .dependencies
          .push(Box::new(HarmonyImportSpecifierDependency::new(
            reference.request.clone(),
            reference.source_order,
            false,
            member_expr.span.real_lo(),
            member_expr.span.real_hi(),
            ids,
            parser.enter_callee && !parser.enter_new_expr,
            !parser.enter_callee, // x.xx()
            reference.specifier.clone(),
            None,
            member_expr.span,
          )));
        return Some(true);
      }
    }
    None
  }

  // collect referenced properties in destructuring
  // import * as a from 'a';
  // const { value } = a;
  fn assign(&self, parser: &mut JavascriptParser, assign_expr: &AssignExpr) -> Option<bool> {
    if let PatOrExpr::Pat(box Pat::Object(object_pat)) = &assign_expr.left
      && assign_expr.op == AssignOp::Assign
      && let box Expr::Ident(ident) = &assign_expr.right
      && let Some(reference) = parser.import_map.get(&ident.to_id())
      && matches!(reference.specifier, Specifier::Namespace(_))
    {
      if let Some(value) = collect_destructuring_assignment_properties(object_pat) {
        parser
          .properties_in_destructuring
          .entry(ident.sym.clone())
          .and_modify(|v| v.extend(value.clone()))
          .or_insert(value);
      }
    }
    None
  }

  fn optional_chaining(
    &self,
    parser: &mut JavascriptParser,
    opt_chain_expr: &OptChainExpr,
  ) -> Option<bool> {
    let expression_info = extract_member_expression_chain(opt_chain_expr);
    // dbg!(&expression_info);
    let member_chain = expression_info.members();
    if member_chain.len() > 1
      && let Some(reference) = parser.import_map.get(&member_chain[0])
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
      parser
        .rewrite_usage_span
        .insert(opt_chain_expr.span, ExtraSpanInfo::ReWriteUsedByExports);
      parser
        .dependencies
        .push(Box::new(HarmonyImportSpecifierDependency::new(
          reference.request.clone(),
          reference.source_order,
          false,
          start,
          end,
          ids,
          parser.enter_callee && !parser.enter_new_expr,
          !parser.enter_callee, // x.xx()
          reference.specifier.clone(),
          None,
          opt_chain_expr.span,
        )));
      return Some(true);
    }
    None
  }
}
