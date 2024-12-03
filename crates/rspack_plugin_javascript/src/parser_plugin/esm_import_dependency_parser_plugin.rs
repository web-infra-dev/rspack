use rspack_core::{ConstDependency, Dependency, DependencyType, ImportAttributes, SpanExt};
use swc_core::atoms::Atom;
use swc_core::common::{Span, Spanned};
use swc_core::ecma::ast::{
  AssignExpr, AssignOp, AssignTarget, AssignTargetPat, Callee, MemberExpr, OptChainBase,
};
use swc_core::ecma::ast::{Expr, Ident, ImportDecl};

use super::{InnerGraphPlugin, JavascriptParserPlugin};
use crate::dependency::{ESMImportSideEffectDependency, ESMImportSpecifierDependency};
use crate::utils::object_properties::get_attributes;
use crate::visitors::{collect_destructuring_assignment_properties, JavascriptParser, TagInfoData};

fn get_non_optional_part<'a>(members: &'a [Atom], members_optionals: &[bool]) -> &'a [Atom] {
  let mut i = 0;
  while i < members.len() && matches!(members_optionals.get(i), Some(false)) {
    i += 1;
  }
  if i != members.len() {
    &members[0..i]
  } else {
    members
  }
}

fn get_non_optional_member_chain_from_expr(mut expr: &Expr, mut count: i32) -> &Expr {
  while count != 0 {
    if let Expr::Member(member) = expr {
      expr = &member.obj;
      count -= 1;
    } else if let Expr::OptChain(opt_chain) = expr {
      expr = match &*opt_chain.base {
        OptChainBase::Member(member) => &*member.obj,
        OptChainBase::Call(call) if let Some(member) = call.callee.as_member() => &*member.obj,
        _ => unreachable!(),
      };
      count -= 1;
    } else {
      unreachable!()
    }
  }
  expr
}

fn get_non_optional_member_chain_from_member(member: &MemberExpr, mut count: i32) -> &Expr {
  count -= 1;
  get_non_optional_member_chain_from_expr(&member.obj, count)
}

pub struct ESMImportDependencyParserPlugin;

pub const ESM_SPECIFIER_TAG: &str = "_identifier__esm_specifier_tag__";

#[derive(Debug, Clone)]
pub struct ESMSpecifierData {
  pub name: Atom,
  pub source: Atom,
  pub ids: Vec<Atom>,
  pub source_order: i32,
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
    let dependency = ESMImportSideEffectDependency::new(
      source.into(),
      parser.last_esm_import_order,
      import_decl.span.into(),
      import_decl.src.span.into(),
      DependencyType::EsmImport,
      false,
      attributes,
      Some(parser.source_map.clone()),
    );
    parser.dependencies.push(Box::new(dependency));

    parser
      .presentational_dependencies
      .push(Box::new(ConstDependency::new(
        import_decl.span.real_lo(),
        import_decl.span.real_hi(),
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
    parser.tag_variable::<ESMSpecifierData>(
      name.to_string(),
      ESM_SPECIFIER_TAG,
      Some(ESMSpecifierData {
        name: name.clone(),
        source: source.clone(),
        ids: id.map(|id| vec![id.clone()]).unwrap_or_default(),
        source_order: parser.last_esm_import_order,
        attributes: statement.with.as_ref().map(|obj| get_attributes(obj)),
      }),
    );
    Some(true)
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
      parser.properties_in_destructuring.remove(&ident.sym),
      settings.attributes,
      Some(parser.source_map.clone()),
    );
    let dep_id = *dep.id();
    parser.dependencies.push(Box::new(dep));

    InnerGraphPlugin::on_usage(
      parser,
      Box::new(move |parser, used_by_exports| {
        if let Some(dep) = parser
          .dependencies
          .iter_mut()
          .find(|dep| dep.id() == &dep_id)
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
      None,
      settings.attributes,
      Some(parser.source_map.clone()),
    );
    let dep_id = *dep.id();
    parser.dependencies.push(Box::new(dep));

    InnerGraphPlugin::on_usage(
      parser,
      Box::new(move |parser, used_by_exports| {
        if let Some(dep) = parser
          .dependencies
          .iter_mut()
          .find(|dep| dep.id() == &dep_id)
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
      None,
      settings.attributes,
      Some(parser.source_map.clone()),
    );
    let dep_id = *dep.id();
    parser.dependencies.push(Box::new(dep));

    InnerGraphPlugin::on_usage(
      parser,
      Box::new(move |parser, used_by_exports| {
        if let Some(dep) = parser
          .dependencies
          .iter_mut()
          .find(|dep| dep.id() == &dep_id)
        {
          dep.set_used_by_exports(used_by_exports);
        }
      }),
    );

    Some(true)
  }

  // collect referenced properties in destructuring
  // import * as a from 'a';
  // const { value } = a;
  fn assign(&self, parser: &mut JavascriptParser, assign_expr: &AssignExpr) -> Option<bool> {
    if let AssignTarget::Pat(AssignTargetPat::Object(object_pat)) = &assign_expr.left
      && assign_expr.op == AssignOp::Assign
      && let box Expr::Ident(ident) = &assign_expr.right
      && let Some(settings) = parser.get_tag_data(&ident.sym, ESM_SPECIFIER_TAG)
      && let settings = ESMSpecifierData::downcast(settings)
      // import namespace
      && settings.ids.is_empty()
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
}
