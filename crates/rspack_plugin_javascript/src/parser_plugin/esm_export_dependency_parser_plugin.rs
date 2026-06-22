use itertools::Itertools;
use rspack_core::{
  BoxDependency, ConstDependency, Dependency, DependencyRange, DependencyType, ImportPhase,
};
use rspack_util::SpanExt;
use swc_atoms::Atom;
use swc_experimental_ecma_ast::{CommentKind, Expr, GetSpan, Span};

use super::{
  DEFAULT_STAR_JS_WORD, JS_DEFAULT_KEYWORD, JavascriptParserPlugin,
  esm_import_dependency_parser_plugin::{ESM_SPECIFIER_TAG, ESMSpecifierData},
  inline_const::{ConstValueData, INLINABLE_CONST_TAG, to_evaluated_inlinable_value},
  inner_graph::state::InnerGraphMapUsage,
};
use crate::{
  ConstValue, InnerGraphParserPlugin,
  dependency::{
    DeclarationId, DeclarationInfo, ESMExportExpressionDependency, ESMExportHeaderDependency,
    ESMExportImportedSpecifierDependency, ESMExportSpecifierDependency,
    ESMImportSideEffectDependency,
  },
  parser_plugin::compatibility_plugin::{NESTED_IDENTIFIER_TAG, NestedRequireData},
  utils::object_properties::get_attributes,
  visitors::{
    ExportDefaultDeclaration, ExportDefaultExpression, ExportImport, ExportLocal, JavascriptParser,
    create_traceable_error,
  },
};

pub struct ESMExportDependencyParserPlugin;

fn create_default_exported_namespace_dependency(
  parser: &mut JavascriptParser,
  statement: ExportDefaultDeclaration,
  expr: ExportDefaultExpression,
) -> Option<ESMExportImportedSpecifierDependency> {
  let ExportDefaultExpression::Expr(Expr::Ident(ident)) = expr else {
    return None;
  };
  let settings = parser
    .get_tag_data::<ESMSpecifierData>(&Atom::from(ident.sym.as_str()), ESM_SPECIFIER_TAG)
    .filter(|settings| settings.namespace_import && settings.ids.is_empty())?
    .clone();
  let statement_span = statement.span();
  let mut dep = ESMExportImportedSpecifierDependency::new(
    settings.source,
    settings.source_order,
    vec![],
    Some(JS_DEFAULT_KEYWORD.clone()),
    None,
    statement_span.into(),
    ESMExportImportedSpecifierDependency::create_export_presence_mode(parser.javascript_options),
    settings.phase,
    settings.attributes,
    parser.to_dependency_location(DependencyRange::from(statement_span)),
  );
  if parser
    .factory_meta
    .and_then(|meta| meta.side_effect_free)
    .unwrap_or_default()
  {
    dep.set_lazy();
  }
  Some(dep)
}

#[rspack_macros::implemented_javascript_parser_hooks]
impl<'p, 'a> JavascriptParserPlugin<'p, 'a> for ESMExportDependencyParserPlugin {
  fn export(&self, parser: &mut JavascriptParser<'p>, statement: ExportLocal) -> Option<bool> {
    let range = DependencyRange::from(statement.span());
    let loc = parser.to_dependency_location(range);
    let dep = ESMExportHeaderDependency::new(
      statement.span().into(),
      statement.declaration_span().map(|span| span.into()),
      loc,
    );
    parser.add_presentational_dependency(Box::new(dep));
    Some(true)
  }

  fn export_import(
    &self,
    parser: &mut JavascriptParser<'p>,
    statement: ExportImport,
    source: &Atom,
  ) -> Option<bool> {
    parser.last_esm_import_order += 1;
    let clean_dep = ConstDependency::new(statement.span().into(), "".into());
    parser.add_presentational_dependency(Box::new(clean_dep));
    let range = DependencyRange::from(statement.span());
    let loc = parser.to_dependency_location(range);
    let mut side_effect_dep = ESMImportSideEffectDependency::new(
      source.clone(),
      parser.last_esm_import_order,
      statement.span().into(),
      DependencyType::EsmExportImport,
      ImportPhase::Evaluation,
      statement.get_with_obj().map(get_attributes),
      loc,
      statement.is_star_export(),
    );
    if parser
      .factory_meta
      .and_then(|meta| meta.side_effect_free)
      .unwrap_or_default()
    {
      side_effect_dep.set_lazy();
    }
    parser.add_dependency(Box::new(side_effect_dep));
    Some(true)
  }

  fn export_specifier(
    &self,
    parser: &mut JavascriptParser<'p>,
    statement: ExportLocal,
    local_id: &Atom,
    export_name: &Atom,
    export_name_span: Span,
  ) -> Option<bool> {
    InnerGraphParserPlugin::add_variable_usage(
      parser,
      local_id,
      InnerGraphMapUsage::Value(export_name.clone()),
    );
    if !parser
      .build_info
      .esm_named_exports
      .insert(export_name.clone())
    {
      parser.add_error(
        create_traceable_error(
          "JavaScript parse error".into(),
          format!("Duplicate export of '{export_name}'"),
          parser.source.to_string(),
          export_name_span.into(),
        )
        .into(),
      );
    }
    let dep = if let Some((source, source_order, ids, phase, attributes)) = parser
      .get_tag_data::<ESMSpecifierData>(local_id, ESM_SPECIFIER_TAG)
      .map(|settings| {
        (
          settings.source.clone(),
          settings.source_order,
          settings.ids.clone(),
          settings.phase,
          settings.attributes.clone(),
        )
      }) {
      let range = DependencyRange::from(statement.span());
      let loc = parser.to_dependency_location(range);
      let mut dep = ESMExportImportedSpecifierDependency::new(
        source,
        source_order,
        ids.into_vec(),
        Some(export_name.clone()),
        None,
        statement.span().into(),
        ESMExportImportedSpecifierDependency::create_export_presence_mode(
          parser.javascript_options,
        ),
        phase,
        attributes,
        loc,
      );
      if parser
        .factory_meta
        .and_then(|meta| meta.side_effect_free)
        .unwrap_or_default()
      {
        dep.set_lazy();
      }
      Box::new(dep) as BoxDependency
    } else {
      let const_value = parser
        .get_tag_data::<ConstValueData>(local_id, INLINABLE_CONST_TAG)
        .map(|data| data.value.clone());
      let enum_value = parser
        .build_info
        .collected_typescript_info
        .as_ref()
        .and_then(|info| info.exported_enums.get(local_id).cloned());
      let variable = parser
        .get_tag_data::<NestedRequireData>(local_id, NESTED_IDENTIFIER_TAG)
        .map(|data| data.name.clone());

      let range = DependencyRange::from(statement.span());
      let loc = parser.to_dependency_location(range);
      Box::new(ESMExportSpecifierDependency::new(
        export_name.clone(),
        if let Some(variable) = variable {
          variable.into()
        } else {
          local_id.clone()
        },
        const_value,
        enum_value,
        statement.span().into(),
        loc,
      ))
    };
    let is_asi_safe = !parser.is_asi_position(statement.span().start);
    if !is_asi_safe {
      parser.set_asi_position(statement.span().end);
    }
    parser.add_dependency(dep);
    Some(true)
  }

  fn export_import_specifier(
    &self,
    parser: &mut JavascriptParser<'p>,
    statement: ExportImport,
    source: &Atom,
    local_id: Option<&Atom>,
    export_name: Option<&Atom>,
    export_name_span: Option<Span>,
  ) -> Option<bool> {
    let star_exports = if let Some(export_name) = export_name {
      if !parser
        .build_info
        .esm_named_exports
        .insert(export_name.clone())
      {
        parser.add_error(
          create_traceable_error(
            "JavaScript parse error".into(),
            format!("Duplicate export of '{export_name}'"),
            parser.source.to_string(),
            export_name_span.expect("should exist").into(),
          )
          .into(),
        );
      }
      None
    } else {
      Some(parser.build_info.all_star_exports.clone())
    };
    let mut dep = ESMExportImportedSpecifierDependency::new(
      source.clone(),
      parser.last_esm_import_order,
      local_id.map(|id| vec![id.clone()]).unwrap_or_default(),
      export_name.cloned(),
      star_exports,
      statement.span().into(),
      ESMExportImportedSpecifierDependency::create_export_presence_mode(parser.javascript_options),
      ImportPhase::Evaluation,
      statement.get_with_obj().map(get_attributes),
      parser.to_dependency_location(DependencyRange::from(statement.span())),
    );
    if export_name.is_none() {
      parser.build_info.all_star_exports.push(dep.id);
    }
    let is_asi_safe = !parser.is_asi_position(statement.span().start);
    if !is_asi_safe {
      parser.set_asi_position(statement.span().end);
    }
    if parser
      .factory_meta
      .and_then(|meta| meta.side_effect_free)
      .unwrap_or_default()
    {
      dep.set_lazy();
    }
    parser.add_dependency(Box::new(dep));
    Some(true)
  }

  fn export_expression(
    &self,
    parser: &mut JavascriptParser<'p>,
    statement: ExportDefaultDeclaration,
    expr: ExportDefaultExpression,
  ) -> Option<bool> {
    let expr_span = expr.span();
    let statement_span = statement.span();
    if let Some(dep) = create_default_exported_namespace_dependency(parser, statement, expr) {
      parser.add_presentational_dependency(Box::new(ConstDependency::new(
        DependencyRange::new(statement_span.real_lo(), expr_span.real_lo()),
        "".into(),
      )));
      parser.add_dependency(Box::new(dep));
      return Some(true);
    }

    let comment = parser
      .ast
      .comments
      .leading
      .get(&expr_span.start)
      .map(|c| {
        c.iter()
          .dedup()
          .map(|c| match c.kind {
            CommentKind::Block => format!("/*{}*/", c.text),
            CommentKind::Line => format!("//{}\n", c.text),
          })
          .collect_vec()
          .join("")
      })
      .unwrap_or_default();
    let declaration = match expr {
      ExportDefaultExpression::FnDecl(f) => {
        let start = f.span().real_lo();
        let end = if let Some(first_arg) = f.function.params.first() {
          first_arg.span().real_lo()
        } else {
          f.function.body.span().real_lo()
        };
        Some(DeclarationId::Func(DeclarationInfo::new(
          DependencyRange::new(start, end),
          format!(
            "{}function{} ",
            if f.function.is_async { "async " } else { "" },
            if f.function.is_generator { "*" } else { "" },
          ),
          format!(
            r#"({}"#,
            if f.function.params.is_empty() {
              ") "
            } else {
              ""
            }
          ),
        )))
      }
      ExportDefaultExpression::ClassDecl(c) => c
        .ident
        .as_ref()
        .map(|ident| DeclarationId::Id(ident.sym.to_string())),
      ExportDefaultExpression::Expr(_) => None,
    };
    let const_value = match expr {
      ExportDefaultExpression::Expr(Expr::Ident(ident)) => parser
        .get_tag_data::<ConstValueData>(&Atom::from(ident.sym.as_str()), INLINABLE_CONST_TAG)
        .map(|data| data.value.clone()),
      ExportDefaultExpression::Expr(expr) => {
        to_evaluated_inlinable_value(&parser.evaluate_expression(expr)).map(ConstValue::Inlinable)
      }
      _ => None,
    };
    let dep = ESMExportExpressionDependency::new(
      expr_span.into(),
      statement_span.into(),
      comment,
      declaration,
      const_value,
      parser.to_dependency_location(DependencyRange::from(expr_span)),
    );
    parser.add_dependency(Box::new(dep));
    let name = expr.ident().map_or_else(
      || DEFAULT_STAR_JS_WORD.clone(),
      |ident| Atom::from(ident.as_str()),
    );
    InnerGraphParserPlugin::add_variable_usage(
      parser,
      &name,
      InnerGraphMapUsage::Value(JS_DEFAULT_KEYWORD.clone()),
    );
    Some(true)
  }
}
