use rspack_core::{ConstDependency, Dependency, DependencyType, ImportAttributes};
use swc_core::{
  atoms::Atom,
  common::{Span, Spanned},
  ecma::ast::{Callee, Expr, Ident, ImportDecl, MemberExpr, OptChainBase},
};

use super::{InnerGraphPlugin, JavascriptParserPlugin};
use crate::{
  dependency::{ESMImportSideEffectDependency, ESMImportSpecifierDependency},
  utils::object_properties::get_attributes,
  visitors::{JavascriptParser, TagInfoData},
};

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
        OptChainBase::Call(call) if call.callee.as_member().is_some() => {
          let member = call
            .callee
            .as_member()
            .expect("`call.callee` is `MemberExpr` in `if_guard`");
          &*member.obj
        }
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
      attributes,
      Some(parser.source_map.clone()),
    );
    parser.dependencies.push(Box::new(dependency));

    parser
      .presentational_dependencies
      .push(Box::new(ConstDependency::new(
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
    let referenced_properties_in_destructuring =
      parser.destructuring_assignment_properties_for(&ident.span());
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
      // we don't need to pass destructuring properties here, since this is a call expr,
      // pass destructuring properties here won't help for tree shaking.
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
    let referenced_properties_in_destructuring =
      parser.destructuring_assignment_properties_for(&member_expr.span());
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
}
