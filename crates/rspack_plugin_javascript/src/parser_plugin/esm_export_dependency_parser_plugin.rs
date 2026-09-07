use std::sync::Arc;

use itertools::Itertools;
use rspack_core::{
  BoxDependency, ConstDependency, Dependency, DependencyRange, DependencyType, ImportPhase,
};
use rspack_util::SpanExt;
use swc_next_ecma_ast::{CommentKind, ExprData, GetSpan, Span};

use super::{
  DEFAULT_STAR_JS_WORD, JS_DEFAULT_KEYWORD, JavascriptParserPlugin,
  esm_import_dependency_parser_plugin::{ESM_SPECIFIER_TAG, ESMSpecifierData},
  inline_const::{ConstValueData, INLINABLE_CONST_TAG, to_evaluated_inlinable_value},
  inner_graph::state::InnerGraphMapUsage,
};
use crate::{
  Atom, ConstValue, InnerGraphParserPlugin,
  dependency::{
    DeclarationId, DeclarationInfo, ESMExportExpressionDependency, ESMExportHeaderDependency,
    ESMExportImportedSpecifierDependency, ESMExportSpecifierDependency,
    ESMImportSideEffectDependency,
  },
  parser_plugin::compatibility_plugin::CompatibilityPlugin,
  utils::object_properties::get_import_attributes,
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
  let ast = parser.ast.ast;
  let ExportDefaultExpression::Expr(expression) = expr else {
    return None;
  };
  let ExprData::IdentifierReference(identifier) = ast.expr_data(expression) else {
    return None;
  };
  let settings = parser
    .get_tag_data::<ESMSpecifierData>(
      &Atom::from(ast.get_utf8(identifier.name(ast))),
      ESM_SPECIFIER_TAG,
    )
    .filter(|settings| settings.namespace_import && settings.ids.is_empty())?
    .clone();
  let statement_span = statement.span(ast);
  let dep = ESMExportImportedSpecifierDependency::new(
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
    let ast = parser.ast.ast;
    let statement_span = statement.span(ast);
    let range = DependencyRange::from(statement_span);
    let loc = parser.to_dependency_location(range);
    let dep = ESMExportHeaderDependency::new(
      statement_span.into(),
      statement.declaration_span(ast).map(|span| span.into()),
      loc,
    );
    parser.add_presentational_dependency(Arc::new(dep));
    Some(true)
  }

  fn export_import(
    &self,
    parser: &mut JavascriptParser<'p>,
    statement: ExportImport,
    source: &Atom,
  ) -> Option<bool> {
    let ast = parser.ast.ast;
    let statement_span = statement.span(ast);
    parser.last_esm_import_order += 1;
    let clean_dep = ConstDependency::new(statement_span.into(), "".into());
    parser.add_presentational_dependency(Arc::new(clean_dep));
    let range = DependencyRange::from(statement_span);
    let loc = parser.to_dependency_location(range);
    let side_effect_dep = ESMImportSideEffectDependency::new(
      source.clone(),
      parser.last_esm_import_order,
      statement_span.into(),
      DependencyType::EsmExportImport,
      ImportPhase::Evaluation,
      get_import_attributes(ast, statement.attributes(ast)),
      loc,
      statement.is_star_export(ast),
    );
    if parser
      .factory_meta
      .and_then(|meta| meta.side_effect_free)
      .unwrap_or_default()
    {
      side_effect_dep.set_lazy();
    }
    parser.add_dependency(BoxDependency::new(side_effect_dep));
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
    let ast = parser.ast.ast;
    let statement_span = statement.span(ast);
    if parser.javascript_options.is_create_require_enabled() {
      parser
        .created_require_references
        .record_exported_local(local_id.clone());
    }
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
      let range = DependencyRange::from(statement_span);
      let loc = parser.to_dependency_location(range);
      let dep = ESMExportImportedSpecifierDependency::new(
        source,
        source_order,
        ids.into_vec(),
        Some(export_name.clone()),
        None,
        statement_span.into(),
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
      BoxDependency::new(dep)
    } else {
      let const_value = parser
        .get_tag_data::<ConstValueData>(local_id, INLINABLE_CONST_TAG)
        .map(|data| data.value.clone());
      let enum_value = parser
        .build_info
        .collected_typescript_info
        .as_ref()
        .and_then(|info| info.exported_enums.get(local_id).cloned());
      let variable = CompatibilityPlugin::update_nested_binding_declaration(parser, local_id);

      let range = DependencyRange::from(statement_span);
      let loc = parser.to_dependency_location(range);
      BoxDependency::new(ESMExportSpecifierDependency::new(
        export_name.clone(),
        if let Some(variable) = variable {
          variable
        } else {
          local_id.clone()
        },
        const_value,
        enum_value,
        statement_span.into(),
        loc,
      ))
    };
    let is_asi_safe = !parser.is_asi_position(statement_span.start);
    if !is_asi_safe {
      parser.set_asi_position(statement_span.end);
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
    let ast = parser.ast.ast;
    let statement_span = statement.span(ast);
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
    let dep = ESMExportImportedSpecifierDependency::new(
      source.clone(),
      parser.last_esm_import_order,
      local_id.map(|id| vec![id.clone()]).unwrap_or_default(),
      export_name.cloned(),
      star_exports,
      statement_span.into(),
      ESMExportImportedSpecifierDependency::create_export_presence_mode(parser.javascript_options),
      ImportPhase::Evaluation,
      get_import_attributes(ast, statement.attributes(ast)),
      parser.to_dependency_location(DependencyRange::from(statement_span)),
    );
    if export_name.is_none() {
      parser.build_info.all_star_exports.push(dep.id);
    }
    let is_asi_safe = !parser.is_asi_position(statement_span.start);
    if !is_asi_safe {
      parser.set_asi_position(statement_span.end);
    }
    if parser
      .factory_meta
      .and_then(|meta| meta.side_effect_free)
      .unwrap_or_default()
    {
      dep.set_lazy();
    }
    parser.add_dependency(BoxDependency::new(dep));
    Some(true)
  }

  fn export_expression(
    &self,
    parser: &mut JavascriptParser<'p>,
    statement: ExportDefaultDeclaration,
    expr: ExportDefaultExpression,
  ) -> Option<bool> {
    let ast = parser.ast.ast;
    let expr_span = expr.span(ast);
    let statement_span = statement.span(ast);
    if let Some(dep) = create_default_exported_namespace_dependency(parser, statement, expr) {
      parser.add_presentational_dependency(Arc::new(ConstDependency::new(
        DependencyRange::new(statement_span.real_lo(), expr_span.real_lo()),
        "".into(),
      )));
      parser.add_dependency(BoxDependency::new(dep));
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
        let start = f.span(ast).real_lo();
        let params = f.params(ast);
        let end = if let Some(first_arg) = params.items(ast).get_node(ast, 0) {
          first_arg.span(ast).real_lo()
        } else if let Some(rest) = params.rest(ast) {
          rest.span(ast).real_lo()
        } else {
          f.body(ast).span(ast).real_lo()
        };
        Some(DeclarationId::Func(DeclarationInfo::new(
          DependencyRange::new(start, end),
          format!(
            "{}function{} ",
            if f.r#async(ast) { "async " } else { "" },
            if f.generator(ast) { "*" } else { "" },
          ),
          format!(
            r#"({}"#,
            if params.items(ast).is_empty() && params.rest(ast).is_none() {
              ") "
            } else {
              ""
            }
          ),
        )))
      }
      ExportDefaultExpression::ClassDecl(c) => c
        .id(ast)
        .map(|identifier| DeclarationId::Id(ast.get_utf8(identifier.name(ast)).to_string())),
      ExportDefaultExpression::Expr(_) | ExportDefaultExpression::Other(_) => None,
    };
    let const_value = match expr {
      ExportDefaultExpression::Expr(expression) => {
        if let ExprData::IdentifierReference(identifier) = ast.expr_data(expression) {
          parser
            .get_tag_data::<ConstValueData>(
              &Atom::from(ast.get_utf8(identifier.name(ast))),
              INLINABLE_CONST_TAG,
            )
            .map(|data| data.value.clone())
        } else {
          to_evaluated_inlinable_value(&parser.evaluate_expression(expression))
            .map(ConstValue::Inlinable)
        }
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
    parser.add_dependency(BoxDependency::new(dep));
    let name = expr.ident(ast).map_or_else(
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
