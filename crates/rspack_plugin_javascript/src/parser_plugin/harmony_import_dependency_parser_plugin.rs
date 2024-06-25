use rspack_core::{ConstDependency, Dependency, DependencyType, SpanExt};
use swc_core::atoms::Atom;
use swc_core::common::{Span, Spanned};
use swc_core::ecma::ast::{
  AssignExpr, AssignOp, AssignTarget, AssignTargetPat, Callee, ImportSpecifier, MemberExpr,
  ModuleExportName, OptChainBase,
};
use swc_core::ecma::ast::{Expr, Ident, ImportDecl};

use super::{InnerGraphPlugin, JavascriptParserPlugin};
use crate::dependency::{
  HarmonyImportSideEffectDependency, HarmonyImportSpecifierDependency, Specifier,
};
use crate::visitors::ImporterReferenceInfo;
use crate::visitors::{collect_destructuring_assignment_properties, JavascriptParser, TagInfoData};

pub(super) fn handle_harmony_import_side_effects_dep(
  parser: &mut JavascriptParser,
  request: Atom,
  span: Span,
  source_span: Span,
  dep_type: DependencyType,
  exports_all: bool,
) {
  let dependency = HarmonyImportSideEffectDependency::new(
    request,
    parser.last_harmony_import_order,
    Some(span.into()),
    Some(source_span.into()),
    dep_type,
    exports_all,
  );
  parser.dependencies.push(Box::new(dependency));
}

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

pub struct HarmonyImportDependencyParserPlugin;

pub const HARMONY_SPECIFIER_TAG: &str = "_identifier__harmony_specifier_tag__";

#[derive(Debug, Clone)]
pub struct HarmonySpecifierData {
  pub name: Atom,
  pub source: Atom,
  pub ids: Vec<Atom>,
  pub source_order: i32,
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
            Some("default".into()),
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
      DependencyType::EsmImport,
      false,
    );

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
    _statement: &ImportDecl,
    source: &Atom,
    id: Option<&Atom>,
    name: &Atom,
  ) -> Option<bool> {
    parser.tag_variable::<HarmonySpecifierData>(
      name.to_string(),
      HARMONY_SPECIFIER_TAG,
      Some(HarmonySpecifierData {
        name: name.clone(),
        source: source.clone(),
        ids: id.map(|id| vec![id.clone()]).unwrap_or_default(),
        source_order: parser.last_harmony_import_order,
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
    if for_name != HARMONY_SPECIFIER_TAG {
      return None;
    }
    let tag_info = parser
      .definitions_db
      .expect_get_tag_info(&parser.current_tag_info?);
    let settings = HarmonySpecifierData::downcast(tag_info.data.clone()?);

    let dep = HarmonyImportSpecifierDependency::new(
      settings.source,
      settings.name,
      settings.source_order,
      parser.in_short_hand,
      !parser.is_asi_position(ident.span_lo()),
      ident.span.real_lo(),
      ident.span.real_hi(),
      settings.ids,
      parser.in_tagged_template_tag,
      true,
      HarmonyImportSpecifierDependency::create_export_presence_mode(parser.javascript_options),
      parser.properties_in_destructuring.remove(&ident.sym),
      ident.span,
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
    if for_name != HARMONY_SPECIFIER_TAG {
      return None;
    }
    let tag_info = parser
      .definitions_db
      .expect_get_tag_info(&parser.current_tag_info?);
    let settings = HarmonySpecifierData::downcast(tag_info.data.clone()?);

    let non_optional_members = get_non_optional_part(members, members_optionals);
    let (start, end) = if members.len() > non_optional_members.len() {
      let expr = get_non_optional_member_chain_from_expr(
        callee,
        (members.len() - non_optional_members.len()) as i32,
      );
      (expr.span().real_lo(), expr.span().real_hi())
    } else {
      (callee.span().real_lo(), callee.span().real_hi())
    };
    let mut ids = settings.ids;
    ids.extend(non_optional_members.iter().cloned());
    let direct_import = members.is_empty();
    let dep = HarmonyImportSpecifierDependency::new(
      settings.source,
      settings.name,
      settings.source_order,
      false,
      !parser.is_asi_position(call_expr.span_lo()),
      start,
      end,
      ids,
      true,
      direct_import,
      HarmonyImportSpecifierDependency::create_export_presence_mode(parser.javascript_options),
      None,
      callee.span(),
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
    if for_name != HARMONY_SPECIFIER_TAG {
      return None;
    }
    let tag_info = parser
      .definitions_db
      .expect_get_tag_info(&parser.current_tag_info?);
    let settings = HarmonySpecifierData::downcast(tag_info.data.clone()?);

    let non_optional_members = get_non_optional_part(members, members_optionals);
    let (start, end) = if members.len() > non_optional_members.len() {
      let expr = get_non_optional_member_chain_from_member(
        member_expr,
        (members.len() - non_optional_members.len()) as i32,
      );
      (expr.span().real_lo(), expr.span().real_hi())
    } else {
      (member_expr.span.real_lo(), member_expr.span.real_hi())
    };
    let mut ids = settings.ids;
    ids.extend(non_optional_members.iter().cloned());
    let dep = HarmonyImportSpecifierDependency::new(
      settings.source,
      settings.name,
      settings.source_order,
      false,
      !parser.is_asi_position(member_expr.span_lo()),
      start,
      end,
      ids,
      false,
      false, // x.xx()
      HarmonyImportSpecifierDependency::create_export_presence_mode(parser.javascript_options),
      None,
      member_expr.span,
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
}
