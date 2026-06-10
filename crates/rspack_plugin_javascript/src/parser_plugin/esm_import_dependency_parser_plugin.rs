use rspack_core::{
  ConstDependency, DependencyRange, DependencyType, ExportPresenceMode, ImportAttributes,
  ImportPhase,
};
use swc_atoms::Atom;
use swc_experimental_ecma_ast::{
  BinExpr, BinaryOp, CallExpr, Callee, Expr, GetSpan, Ident, ImportDecl, MemberExpr, Span,
};

use super::{
  InnerGraphParserPlugin, JavascriptParserPlugin,
  common_js_imports_parse_plugin::{is_create_require_import, tag_create_require},
  import_phase::get_import_phase,
  inner_graph::state::InnerGraphUsageOperation,
};
use crate::{
  dependency::{ESMImportSideEffectDependency, ESMImportSpecifierDependency},
  utils::object_properties::get_attributes,
  visitors::{
    AllowedMemberTypes, AtomMembers, ExportedVariableInfo, JavascriptParser, MemberExpressionInfo,
    TagInfoData, get_non_optional_member_chain_from_expr,
    get_non_optional_member_chain_from_member, get_non_optional_part,
  },
};

pub struct ESMImportDependencyParserPlugin;

pub const ESM_SPECIFIER_TAG: &str = "_identifier__esm_specifier_tag__";

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
    import_decl: &ImportDecl,
    source: &str,
  ) -> Option<bool> {
    parser.last_esm_import_order += 1;
    let attributes = import_decl.with.as_ref().map(|obj| get_attributes(obj));
    let phase = get_import_phase(parser, import_decl.phase, None, None);
    check_import_phase(parser, phase);
    let import_span = import_decl.span;
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

    parser.add_dependency(Box::new(dependency));

    parser.add_presentational_dependency(Box::new(ConstDependency::new(
      import_span.into(),
      if parser.is_asi_position(import_decl.span_lo()) {
        ";".into()
      } else {
        "".into()
      },
    )));
    parser.unset_asi_position(import_decl.span_hi());
    Some(true)
  }

  fn import_specifier(
    &self,
    parser: &mut JavascriptParser<'p>,
    statement: &ImportDecl,
    source: &Atom,
    id: Option<&Atom>,
    name: &Atom,
  ) -> Option<bool> {
    let is_create_require = is_create_require_import(parser, source, id);
    let phase = get_import_phase(parser, statement.phase, None, None);
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
        attributes: statement.with.as_ref().map(|obj| get_attributes(obj)),
      }),
    );
    if is_create_require {
      tag_create_require(parser, name.clone());
    }
    Some(true)
  }

  fn binary_expression(&self, parser: &mut JavascriptParser<'p>, expr: &BinExpr) -> Option<bool> {
    if expr.op != BinaryOp::In {
      return None;
    }
    let right = parser.evaluate_expression(&expr.right);
    if !right.is_identifier() {
      return None;
    }
    let root_info = right.root_info();
    let (source, name, source_order, phase, attributes, namespace_import, mut ids) =
      if let ExportedVariableInfo::VariableInfo(variable) = root_info
        && let Some(settings) =
          parser.get_variable_tag_data::<ESMSpecifierData>(*variable, ESM_SPECIFIER_TAG)
      {
        (
          settings.source.clone(),
          settings.name.clone(),
          settings.source_order,
          settings.phase,
          settings.attributes.clone(),
          settings.namespace_import,
          settings.ids.clone(),
        )
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
    ids.extend(members.iter().cloned());
    ids.push(left.into());

    let expr_span = expr.span;
    let range = DependencyRange::from(expr_span);
    let loc = parser.to_dependency_location(range);
    let mut dep = ESMImportSpecifierDependency::new(
      source,
      name,
      source_order,
      parser.in_short_hand,
      !parser.is_asi_position(expr.span_lo()),
      expr_span.into(),
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

    let dep_idx = parser.next_dependency_idx();
    parser.add_dependency(Box::new(dep));

    InnerGraphParserPlugin::on_usage(
      parser,
      InnerGraphUsageOperation::ESMImportSpecifier(dep_idx),
    );

    Some(true)
  }

  fn can_collect_destructuring_assignment_properties(
    &self,
    parser: &mut JavascriptParser<'p>,
    expr: &Expr,
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
    let range = DependencyRange::from(ident.span);
    let loc = parser.to_dependency_location(range);
    let dep = ESMImportSpecifierDependency::new(
      settings.source,
      settings.name,
      settings.source_order,
      parser.in_short_hand,
      !parser.is_asi_position(ident.span_lo()),
      DependencyRange::from(ident.span),
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
    let dep_idx = parser.next_dependency_idx();
    parser.add_dependency(Box::new(dep));

    InnerGraphParserPlugin::on_usage(
      parser,
      InnerGraphUsageOperation::ESMImportSpecifier(dep_idx),
    );

    Some(true)
  }

  fn call_member_chain(
    &self,
    parser: &mut JavascriptParser<'p>,
    call_expr: &CallExpr,
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
    let ns_access = settings.namespace_import && !ids.is_empty();
    let mut dep = ESMImportSpecifierDependency::new(
      settings.source,
      settings.name,
      settings.source_order,
      false,
      !parser.is_asi_position(call_expr.span_lo()),
      span.into(),
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
      parser.to_dependency_location(DependencyRange::from(call_expr.callee.span())),
    );
    dep.namespace_object_as_context = parser
      .javascript_options
      .strict_this_context_on_imports
      .unwrap_or(false)
      && !direct_import;
    let dep_idx = parser.next_dependency_idx();
    parser.add_dependency(Box::new(dep));

    InnerGraphParserPlugin::on_usage(
      parser,
      InnerGraphUsageOperation::ESMImportSpecifier(dep_idx),
    );

    parser.walk_expr_or_spread(&call_expr.args);
    Some(true)
  }

  fn member_chain(
    &self,
    parser: &mut JavascriptParser<'p>,
    member_expr: &MemberExpr,
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
    let ns_access = settings.namespace_import && !ids.is_empty();
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
      ids.into_vec(),
      false,
      false, // x.xx()
      ns_access,
      ESMImportSpecifierDependency::create_export_presence_mode(parser.javascript_options),
      referenced_properties_in_destructuring,
      settings.phase,
      settings.attributes,
      parser.to_dependency_location(DependencyRange::from(span)),
    );
    let dep_idx = parser.next_dependency_idx();
    parser.add_dependency(Box::new(dep));

    InnerGraphParserPlugin::on_usage(
      parser,
      InnerGraphUsageOperation::ESMImportSpecifier(dep_idx),
    );

    Some(true)
  }
}
