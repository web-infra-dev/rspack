use std::sync::Arc;

use rspack_core::{
  ConstDependency, DependencyType, ExportPresenceMode, ImportAttributes, ImportPhase,
};
use swc_core::{
  atoms::Atom,
  common::{Span, Spanned},
  ecma::ast::{BinExpr, BinaryOp, Callee, Expr, Ident, ImportDecl},
};

use super::{InnerGraphPlugin, JavascriptParserPlugin};
use crate::{
  dependency::{ESMImportSideEffectDependency, ESMImportSpecifierDependency},
  utils::object_properties::get_attributes,
  visitors::{
    AllowedMemberTypes, ExportedVariableInfo, JavascriptParser, MemberExpressionInfo, TagInfoData,
    get_non_optional_member_chain_from_expr, get_non_optional_member_chain_from_member,
    get_non_optional_part,
  },
};

pub struct ESMImportDependencyParserPlugin;

pub const ESM_SPECIFIER_TAG: &str = "_identifier__esm_specifier_tag__";

#[derive(Debug, Clone)]
pub struct ESMSpecifierData {
  pub name: Atom,
  pub source: Atom,
  pub ids: Vec<Atom>,
  pub source_order: i32,
  pub phase: ImportPhase,
  pub attributes: Option<ImportAttributes>,
}

impl JavascriptParserPlugin for ESMImportDependencyParserPlugin {
  fn import(
    &self,
    parser: &mut JavascriptParser,
    import_decl: &ImportDecl,
    source: &str,
  ) -> Option<bool> {
    parser.last_esm_import_order += 1;
    let attributes = import_decl.with.as_ref().map(|obj| get_attributes(obj));
    let phase = if parser.javascript_options.defer_import.unwrap_or_default() {
      import_decl.phase.into()
    } else {
      ImportPhase::Evaluation
    };
    if !parser.compiler_options.experiments.defer_import && phase == ImportPhase::Defer {
      parser.add_error(rspack_error::error!("deferImport is still an experimental feature. To continue using it, please enable 'experiments.deferImport'.").into());
    }
    if phase == ImportPhase::Source {
      parser
        .add_error(rspack_error::error!("Source phase imports is not supported in Rspack.").into());
    }
    let dependency = ESMImportSideEffectDependency::new(
      source.into(),
      parser.last_esm_import_order,
      import_decl.span.into(),
      DependencyType::EsmImport,
      phase,
      attributes,
      Some(parser.source().clone()),
      false,
    );

    parser.add_dependency(Box::new(dependency));

    parser.add_presentational_dependency(Box::new(ConstDependency::new(
      import_decl.span.into(),
      if parser.is_asi_position(import_decl.span_lo()) {
        ";".into()
      } else {
        "".into()
      },
      None,
    )));
    parser.unset_asi_position(import_decl.span_hi());
    Some(true)
  }

  fn import_specifier(
    &self,
    parser: &mut JavascriptParser,
    statement: &ImportDecl,
    source: &Atom,
    id: Option<&Atom>,
    name: &Atom,
  ) -> Option<bool> {
    let phase = if parser.javascript_options.defer_import.unwrap_or_default() {
      statement.phase.into()
    } else {
      ImportPhase::Evaluation
    };
    parser.tag_variable::<ESMSpecifierData>(
      name.clone(),
      ESM_SPECIFIER_TAG,
      Some(ESMSpecifierData {
        name: name.clone(),
        source: source.clone(),
        ids: id.map(|id| vec![id.clone()]).unwrap_or_default(),
        source_order: parser.last_esm_import_order,
        phase,
        attributes: statement.with.as_ref().map(|obj| get_attributes(obj)),
      }),
    );
    Some(true)
  }

  fn binary_expression(&self, parser: &mut JavascriptParser, expr: &BinExpr) -> Option<bool> {
    if expr.op != BinaryOp::In {
      return None;
    }
    let right = parser.evaluate_expression(&expr.right);
    if !right.is_identifier() {
      return None;
    }
    let root_info = right.root_info();
    let settings = if let ExportedVariableInfo::VariableInfo(variable) = root_info
      && let Some(variable_name) = &parser.definitions_db.expect_get_variable(*variable).name
      && let Some(data) = parser.get_tag_data(&variable_name.clone(), ESM_SPECIFIER_TAG)
    {
      ESMSpecifierData::downcast(data)
    } else {
      return None;
    };
    let left = parser.evaluate_expression(&expr.left);
    if left.could_have_side_effects() {
      return None;
    }
    let left = left.as_string()?;
    let members = right.members().map(|v| v.as_slice()).unwrap_or_default();
    let direct_import = members.is_empty();
    let mut ids = settings.ids;
    ids.extend(members.iter().cloned());
    ids.push(left.into());

    let mut dep = ESMImportSpecifierDependency::new(
      settings.source,
      settings.name,
      settings.source_order,
      parser.in_short_hand,
      !parser.is_asi_position(expr.span_lo()),
      expr.span.into(),
      ids,
      parser.in_tagged_template_tag,
      direct_import,
      ExportPresenceMode::None,
      None,
      settings.phase,
      settings.attributes,
      Some(parser.source().clone()),
    );
    dep.evaluated_in_operator = true;

    let dep_idx = parser.next_dependency_idx();
    parser.add_dependency(Box::new(dep));

    InnerGraphPlugin::on_usage(
      parser,
      Box::new(move |parser, used_by_exports| {
        if let Some(dep) = parser.get_dependency_mut(dep_idx)
          && let Some(dep) = dep.downcast_mut::<ESMImportSpecifierDependency>()
        {
          dep.set_used_by_exports(used_by_exports);
        }
      }),
    );

    Some(true)
  }

  fn can_collect_destructuring_assignment_properties(
    &self,
    parser: &mut JavascriptParser,
    expr: &Expr,
  ) -> Option<bool> {
    if let MemberExpressionInfo::Expression(info) =
      parser.get_member_expression_info_from_expr(expr, AllowedMemberTypes::Expression)?
      && let ExportedVariableInfo::VariableInfo(id) = &info.root_info
      && let Some(name) = &parser.definitions_db.expect_get_variable(*id).name
      && parser
        .get_tag_data(&name.clone(), ESM_SPECIFIER_TAG)
        .is_some()
    {
      return Some(true);
    }
    None
  }

  fn identifier(
    &self,
    parser: &mut JavascriptParser,
    ident: &Ident,
    for_name: &str,
  ) -> Option<bool> {
    if for_name != ESM_SPECIFIER_TAG {
      return None;
    }
    let tag_info = parser
      .definitions_db
      .expect_get_tag_info(parser.current_tag_info?);
    let settings = ESMSpecifierData::downcast(tag_info.data.clone()?);
    let referenced_properties_in_destructuring = parser
      .destructuring_assignment_properties
      .get(&ident.span())
      .cloned();
    let dep = ESMImportSpecifierDependency::new(
      settings.source,
      settings.name,
      settings.source_order,
      parser.in_short_hand,
      !parser.is_asi_position(ident.span_lo()),
      ident.span.into(),
      settings.ids,
      parser.in_tagged_template_tag,
      true,
      ESMImportSpecifierDependency::create_export_presence_mode(parser.javascript_options),
      referenced_properties_in_destructuring,
      settings.phase,
      settings.attributes,
      Some(parser.source().clone()),
    );
    let dep_idx = parser.next_dependency_idx();
    parser.add_dependency(Box::new(dep));

    InnerGraphPlugin::on_usage(
      parser,
      Box::new(move |parser, used_by_exports| {
        if let Some(dep) = parser.get_dependency_mut(dep_idx)
          && let Some(dep) = dep.downcast_mut::<ESMImportSpecifierDependency>()
        {
          dep.set_used_by_exports(used_by_exports);
        }
      }),
    );

    Some(true)
  }

  fn call_member_chain(
    &self,
    parser: &mut JavascriptParser,
    call_expr: &swc_core::ecma::ast::CallExpr,
    for_name: &str,
    members: &[Atom],
    members_optionals: &[bool],
    _member_ranges: &[Span],
  ) -> Option<bool> {
    let Callee::Expr(callee) = &call_expr.callee else {
      unreachable!()
    };
    if for_name != ESM_SPECIFIER_TAG {
      return None;
    }
    let tag_info = parser
      .definitions_db
      .expect_get_tag_info(parser.current_tag_info?);
    let settings = ESMSpecifierData::downcast(tag_info.data.clone()?);

    let non_optional_members = get_non_optional_part(members, members_optionals);
    let span = if members.len() > non_optional_members.len() {
      let expr = get_non_optional_member_chain_from_expr(
        callee,
        (members.len() - non_optional_members.len()) as i32,
      );
      expr.span()
    } else {
      callee.span()
    };
    let mut ids = settings.ids;
    ids.extend(non_optional_members.iter().cloned());
    let direct_import = members.is_empty();
    let dep = ESMImportSpecifierDependency::new(
      settings.source,
      settings.name,
      settings.source_order,
      false,
      !parser.is_asi_position(call_expr.span_lo()),
      span.into(),
      ids,
      true,
      direct_import,
      ESMImportSpecifierDependency::create_export_presence_mode(parser.javascript_options),
      // we don't need to pass destructuring properties here, since this is a call expr,
      // pass destructuring properties here won't help for tree shaking.
      None,
      settings.phase,
      settings.attributes,
      Some(parser.source().clone()),
    );
    let dep_idx = parser.next_dependency_idx();
    parser.add_dependency(Box::new(dep));

    InnerGraphPlugin::on_usage(
      parser,
      Box::new(move |parser, used_by_exports| {
        if let Some(dep) = parser.get_dependency_mut(dep_idx)
          && let Some(dep) = dep.downcast_mut::<ESMImportSpecifierDependency>()
        {
          dep.set_used_by_exports(used_by_exports);
        }
      }),
    );

    parser.walk_expr_or_spread(&call_expr.args);
    Some(true)
  }

  fn member_chain(
    &self,
    parser: &mut JavascriptParser,
    member_expr: &swc_core::ecma::ast::MemberExpr,
    for_name: &str,
    members: &[Atom],
    members_optionals: &[bool],
    _member_ranges: &[Span],
  ) -> Option<bool> {
    if for_name != ESM_SPECIFIER_TAG {
      return None;
    }
    let tag_info = parser
      .definitions_db
      .expect_get_tag_info(parser.current_tag_info?);
    let settings = ESMSpecifierData::downcast(tag_info.data.clone()?);

    let non_optional_members = get_non_optional_part(members, members_optionals);
    let span = if members.len() > non_optional_members.len() {
      let expr = get_non_optional_member_chain_from_member(
        member_expr,
        (members.len() - non_optional_members.len()) as i32,
      );
      expr.span()
    } else {
      member_expr.span()
    };
    let mut ids = settings.ids;
    ids.extend(non_optional_members.iter().cloned());
    let referenced_properties_in_destructuring = parser
      .destructuring_assignment_properties
      .get(&member_expr.span())
      .cloned();
    let dep = ESMImportSpecifierDependency::new(
      settings.source,
      settings.name,
      settings.source_order,
      false,
      !parser.is_asi_position(member_expr.span_lo()),
      span.into(),
      ids,
      false,
      false, // x.xx()
      ESMImportSpecifierDependency::create_export_presence_mode(parser.javascript_options),
      referenced_properties_in_destructuring,
      settings.phase,
      settings.attributes,
      Some(parser.source().clone()),
    );
    let dep_idx = parser.next_dependency_idx();
    parser.add_dependency(Box::new(dep));

    InnerGraphPlugin::on_usage(
      parser,
      Box::new(move |parser, used_by_exports| {
        if let Some(dep) = parser.get_dependency_mut(dep_idx)
          && let Some(dep) = dep.downcast_mut::<ESMImportSpecifierDependency>()
        {
          dep.set_used_by_exports(used_by_exports);
        }
      }),
    );

    Some(true)
  }
}
