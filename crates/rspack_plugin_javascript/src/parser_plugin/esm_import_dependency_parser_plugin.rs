use std::sync::Arc;

use rspack_core::{
  BoxDependency, ConstDependency, Dependency, DependencyRange, DependencyType, ExportPresenceMode,
  ImportAttributes, ImportPhase,
};
use rspack_util::SpanExt;
use swc_next_ecma_ast::{
  BinaryExpression, BinaryOperator, CallExpression, Expr, GetSpan, ImportDeclaration, Span,
};

use super::{
  InnerGraphParserPlugin, JavascriptParserPlugin,
  common_js_imports_parse_plugin::{is_create_require_import, tag_create_require},
  import_phase::get_import_phase,
  inner_graph::state::InnerGraphUsageOperation,
};
use crate::{
  Atom,
  dependency::{ESMImportSideEffectDependency, ESMImportSpecifierDependency},
  utils::{
    eval::{BasicEvaluatedExpression, DependencyData},
    object_properties::get_import_attributes,
  },
  visitors::{
    AllowedMemberTypes, AtomMembers, ExportedVariableInfo, ExpressionExpressionInfo,
    HookMemberExpression, Identifier, JavascriptParser, MemberExpressionInfo, TagInfoData,
    get_non_optional_member_chain_from_expr, get_non_optional_member_chain_from_member,
    get_non_optional_part, iter_arguments,
  },
};

pub struct ESMImportDependencyParserPlugin;

pub const ESM_SPECIFIER_TAG: &str = "_identifier__esm_specifier_tag__";

fn is_esm_specifier_reference(
  parser: &mut JavascriptParser,
  for_name: &str,
  member_expr_info: Option<&ExpressionExpressionInfo>,
) -> bool {
  if for_name == ESM_SPECIFIER_TAG {
    return true;
  }

  let Some(member_expr_info) = member_expr_info else {
    return false;
  };
  let ExportedVariableInfo::VariableInfo(root_info_id) = &member_expr_info.root_info else {
    return false;
  };

  parser
    .get_variable_tag_data::<ESMSpecifierData>(*root_info_id, ESM_SPECIFIER_TAG)
    .is_some()
}

fn check_import_phase(parser: &mut JavascriptParser, phase: ImportPhase) {
  if !parser.compiler_options.experiments.defer_import && phase == ImportPhase::Defer {
    parser.add_error(rspack_error::error!("deferImport is still an experimental feature. To continue using it, please enable 'experiments.deferImport'.").into());
  }
  if !parser.compiler_options.experiments.source_import && phase == ImportPhase::Source {
    parser.add_error(rspack_error::error!("sourceImport is still an experimental feature. To continue using it, please enable 'experiments.sourceImport'.").into());
  }
}

#[derive(Debug, Clone)]
pub struct ESMSpecifierData {
  pub name: Atom,
  pub source: Atom,
  pub ids: AtomMembers,
  pub namespace_import: bool,
  pub source_order: i32,
  pub phase: ImportPhase,
  pub attributes: Option<ImportAttributes>,
}

#[rspack_macros::implemented_javascript_parser_hooks]
impl<'p, 'a> JavascriptParserPlugin<'p, 'a> for ESMImportDependencyParserPlugin {
  fn import(
    &self,
    parser: &mut JavascriptParser<'p>,
    import_decl: ImportDeclaration,
    source: &str,
  ) -> Option<bool> {
    let ast = parser.ast.ast;
    parser.last_esm_import_order += 1;
    let attributes = get_import_attributes(ast, import_decl.attributes(ast));
    let phase = get_import_phase(parser, import_decl.phase(ast));
    check_import_phase(parser, phase);
    let import_span = import_decl.span(ast);
    let dependency = ESMImportSideEffectDependency::new(
      source.into(),
      parser.last_esm_import_order,
      import_span.into(),
      DependencyType::EsmImport,
      phase,
      attributes,
      parser.to_dependency_location(DependencyRange::from(import_span)),
      false,
    );

    parser.add_dependency(BoxDependency::new(dependency));

    parser.add_presentational_dependency(Arc::new(ConstDependency::new(
      import_span.into(),
      if parser.is_asi_position(import_span.real_lo()) {
        ";".into()
      } else {
        "".into()
      },
    )));
    parser.unset_asi_position(import_span.real_hi());
    Some(true)
  }

  fn import_specifier(
    &self,
    parser: &mut JavascriptParser<'p>,
    statement: ImportDeclaration,
    source: &Atom,
    id: Option<&Atom>,
    name: &Atom,
  ) -> Option<bool> {
    let ast = parser.ast.ast;
    let is_create_require = is_create_require_import(parser, source, id);
    let phase = get_import_phase(parser, statement.phase(ast));
    parser.tag_variable::<ESMSpecifierData>(
      name.clone(),
      ESM_SPECIFIER_TAG,
      Some(ESMSpecifierData {
        name: name.clone(),
        source: source.clone(),
        ids: id.into_iter().cloned().collect(),
        namespace_import: id.is_none(),
        source_order: parser.last_esm_import_order,
        phase,
        attributes: get_import_attributes(ast, statement.attributes(ast)),
      }),
    );
    if is_create_require {
      tag_create_require(parser, name.clone());
    }
    Some(true)
  }

  fn binary_expression(
    &self,
    parser: &mut JavascriptParser<'p>,
    expr: BinaryExpression,
  ) -> Option<bool> {
    let ast = parser.ast.ast;
    if expr.operator(ast) != BinaryOperator::In {
      return None;
    }
    let right = parser.evaluate_expression(expr.right(ast));
    if !right.is_identifier() {
      return None;
    }
    let root_info = right.root_info();
    let ESMSpecifierData {
      source,
      name,
      mut ids,
      namespace_import,
      source_order,
      phase,
      attributes,
    } = if let ExportedVariableInfo::VariableInfo(variable) = root_info {
      parser
        .get_variable_tag_data::<ESMSpecifierData>(*variable, ESM_SPECIFIER_TAG)?
        .clone()
    } else {
      return None;
    };
    let left = parser.evaluate_expression(expr.left(ast));
    if left.could_have_side_effects() {
      return None;
    }
    let left = left.as_string()?;
    let members = right.members().map(|v| v.as_slice()).unwrap_or_default();
    let direct_import = members.is_empty();
    ids.extend(members.iter().cloned());
    ids.push(left.into());

    let expr_span = expr.span(ast);
    let range = DependencyRange::from(expr_span);
    let loc = parser.to_dependency_location(range);
    let mut dep = ESMImportSpecifierDependency::new(
      source,
      name,
      source_order,
      parser.in_short_hand,
      !parser.is_asi_position(expr_span.real_lo()),
      range,
      ids.into_vec(),
      parser.in_tagged_template_tag,
      direct_import,
      namespace_import,
      ExportPresenceMode::None,
      None,
      phase,
      attributes,
      loc,
    );
    dep.evaluated_in_operator = true;

    let dep_id = *dep.id();
    let dep_idx = parser.next_dependency_idx();
    parser.add_dependency(BoxDependency::new(dep));

    if let Some(in_guard) = parser.dependencies_in_branch_guard.as_mut() {
      in_guard.insert(range, dep_id);
    }

    InnerGraphParserPlugin::on_usage(
      parser,
      InnerGraphUsageOperation::ESMImportSpecifier(dep_idx),
    );

    Some(true)
  }

  fn evaluate_binary_expression(
    &self,
    parser: &mut JavascriptParser<'p>,
    expr: BinaryExpression,
    left: &BasicEvaluatedExpression<'p>,
  ) -> Option<BasicEvaluatedExpression<'p>> {
    let ast = parser.ast.ast;
    if expr.operator(ast) != BinaryOperator::In {
      return None;
    }
    let dep_id = parser
      .dependencies_in_branch_guard
      .as_ref()?
      .get(&DependencyRange::from(expr.span(ast)))
      .copied()?;
    let right = parser.evaluate_expression(expr.right(ast));
    if !right.is_identifier() {
      return None;
    }
    let ExportedVariableInfo::VariableInfo(root) = right.root_info() else {
      return None;
    };
    parser.get_variable_tag_data::<ESMSpecifierData>(*root, ESM_SPECIFIER_TAG)?;
    let span = expr.span(ast);
    let mut res = BasicEvaluatedExpression::with_range(span.real_lo(), span.real_hi());
    res.set_dependency(DependencyData::Dependency(dep_id));
    res.set_side_effects(left.could_have_side_effects());
    Some(res)
  }

  fn can_collect_destructuring_assignment_properties(
    &self,
    parser: &mut JavascriptParser<'p>,
    expr: Expr,
  ) -> Option<bool> {
    if let MemberExpressionInfo::Expression(info) =
      parser.get_member_expression_info_from_expr(expr, AllowedMemberTypes::Expression)?
      && let ExportedVariableInfo::VariableInfo(id) = &info.root_info
      && parser
        .get_variable_tag_data::<ESMSpecifierData>(*id, ESM_SPECIFIER_TAG)
        .is_some()
    {
      return Some(true);
    }
    None
  }

  fn identifier(
    &self,
    parser: &mut JavascriptParser<'p>,
    ident: &Identifier,
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
    let range = DependencyRange::from(ident.span());
    let loc = parser.to_dependency_location(range);
    let dep = ESMImportSpecifierDependency::new(
      settings.source,
      settings.name,
      settings.source_order,
      parser.in_short_hand,
      !parser.is_asi_position(ident.span().real_lo()),
      range,
      settings.ids.into_vec(),
      parser.in_tagged_template_tag,
      true,
      settings.namespace_import && referenced_properties_in_destructuring.is_some(),
      ESMImportSpecifierDependency::create_export_presence_mode(parser.javascript_options),
      referenced_properties_in_destructuring,
      settings.phase,
      settings.attributes,
      loc,
    );
    let dep_id = *dep.id();
    let dep_idx = parser.next_dependency_idx();
    parser.add_dependency(BoxDependency::new(dep));

    if let Some(in_guard) = parser.dependencies_in_branch_guard.as_mut() {
      in_guard.insert(range, dep_id);
    }

    InnerGraphParserPlugin::on_usage(
      parser,
      InnerGraphUsageOperation::ESMImportSpecifier(dep_idx),
    );

    Some(true)
  }

  fn call_member_chain(
    &self,
    parser: &mut JavascriptParser<'p>,
    call_expr: CallExpression,
    for_name: &str,
    members: &[Atom],
    members_optionals: &[bool],
    _member_ranges: &[Span],
  ) -> Option<bool> {
    let ast = parser.ast.ast;
    let callee = call_expr.callee(ast);
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
        ast,
        callee,
        (members.len() - non_optional_members.len()) as i32,
      );
      expr.span(ast)
    } else {
      callee.span(ast)
    };
    let mut ids = settings.ids;
    ids.extend(non_optional_members.iter().cloned());
    let direct_import = members.is_empty();
    let range = DependencyRange::from(span);
    let ns_access = settings.namespace_import && !ids.is_empty();
    let mut dep = ESMImportSpecifierDependency::new(
      settings.source,
      settings.name,
      settings.source_order,
      false,
      !parser.is_asi_position(call_expr.span(ast).real_lo()),
      range,
      ids.into_vec(),
      true,
      direct_import,
      ns_access,
      ESMImportSpecifierDependency::create_export_presence_mode(parser.javascript_options),
      // we don't need to pass destructuring properties here, since this is a call expr,
      // pass destructuring properties here won't help for tree shaking.
      None,
      settings.phase,
      settings.attributes,
      parser.to_dependency_location(range),
    );
    dep.namespace_object_as_context = parser
      .javascript_options
      .strict_this_context_on_imports
      .unwrap_or(false)
      && !direct_import;
    let dep_id = *dep.id();
    let dep_idx = parser.next_dependency_idx();
    parser.add_dependency(BoxDependency::new(dep));

    if let Some(in_guard) = parser.dependencies_in_branch_guard.as_mut() {
      in_guard.insert(range, dep_id);
    }

    InnerGraphParserPlugin::on_usage(
      parser,
      InnerGraphUsageOperation::ESMImportSpecifier(dep_idx),
    );

    parser.walk_arguments(iter_arguments(ast, call_expr.arguments(ast)));
    Some(true)
  }

  fn member_chain(
    &self,
    parser: &mut JavascriptParser<'p>,
    member_expr: HookMemberExpression,
    for_name: &str,
    members: &[Atom],
    members_optionals: &[bool],
    _member_ranges: &[Span],
  ) -> Option<bool> {
    let ast = parser.ast.ast;
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
        ast,
        member_expr.ecma()?,
        (members.len() - non_optional_members.len()) as i32,
      );
      expr.span(ast)
    } else {
      member_expr.span(ast)
    };
    let mut ids = settings.ids;
    ids.extend(non_optional_members.iter().cloned());
    let range = DependencyRange::from(span);
    let ns_access = settings.namespace_import && !ids.is_empty();
    let referenced_properties_in_destructuring = parser
      .destructuring_assignment_properties
      .get(&member_expr.span(ast))
      .cloned();
    let dep = ESMImportSpecifierDependency::new(
      settings.source,
      settings.name,
      settings.source_order,
      false,
      !parser.is_asi_position(member_expr.span(ast).real_lo()),
      range,
      ids.into_vec(),
      false,
      false, // x.xx()
      ns_access,
      ESMImportSpecifierDependency::create_export_presence_mode(parser.javascript_options),
      referenced_properties_in_destructuring,
      settings.phase,
      settings.attributes,
      parser.to_dependency_location(range),
    );
    let dep_id = *dep.id();
    let dep_idx = parser.next_dependency_idx();
    parser.add_dependency(BoxDependency::new(dep));

    if let Some(in_guard) = parser.dependencies_in_branch_guard.as_mut() {
      in_guard.insert(range, dep_id);
    }

    InnerGraphParserPlugin::on_usage(
      parser,
      InnerGraphUsageOperation::ESMImportSpecifier(dep_idx),
    );

    Some(true)
  }

  fn evaluate_identifier(
    &self,
    parser: &mut JavascriptParser<'p>,
    for_name: &str,
    member_expr_info: Option<&ExpressionExpressionInfo>,
    start: u32,
    end: u32,
  ) -> Option<BasicEvaluatedExpression<'p>> {
    if is_esm_specifier_reference(parser, for_name, member_expr_info)
      && let Some(deps_in_guard) = &parser.dependencies_in_branch_guard
      && let Some(dep) = deps_in_guard.get(&DependencyRange::new(start, end))
    {
      let mut res = BasicEvaluatedExpression::with_range(start, end);
      res.set_dependency(DependencyData::Dependency(*dep));
      return Some(res);
    }
    None
  }
}
