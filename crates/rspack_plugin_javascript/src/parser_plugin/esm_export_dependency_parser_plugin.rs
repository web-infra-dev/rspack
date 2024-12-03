use itertools::Itertools;
use rspack_core::{BoxDependency, ConstDependency, DependencyRange, DependencyType, SpanExt};
use swc_core::atoms::Atom;
use swc_core::common::comments::CommentKind;
use swc_core::common::Spanned;

use super::esm_import_dependency_parser_plugin::{ESMSpecifierData, ESM_SPECIFIER_TAG};
use super::{
  InnerGraphMapUsage, InnerGraphPlugin, JavascriptParserPlugin, DEFAULT_STAR_JS_WORD,
  JS_DEFAULT_KEYWORD,
};
use crate::dependency::{
  DeclarationId, DeclarationInfo, ESMExportExpressionDependency, ESMExportHeaderDependency,
  ESMExportImportedSpecifierDependency, ESMExportSpecifierDependency,
  ESMImportSideEffectDependency,
};
use crate::utils::object_properties::get_attributes;
use crate::visitors::{
  ExportDefaultDeclaration, ExportDefaultExpression, ExportImport, ExportLocal, JavascriptParser,
  TagInfoData,
};

pub struct ESMExportDependencyParserPlugin;

impl JavascriptParserPlugin for ESMExportDependencyParserPlugin {
  fn export(&self, parser: &mut JavascriptParser, statement: ExportLocal) -> Option<bool> {
    let dep = ESMExportHeaderDependency::new(
      statement.span().into(),
      statement.declaration_span().map(|span| span.into()),
      Some(parser.source_map.clone()),
    );
    parser.presentational_dependencies.push(Box::new(dep));
    Some(true)
  }

  fn export_import(
    &self,
    parser: &mut JavascriptParser,
    statement: ExportImport,
    source: &Atom,
  ) -> Option<bool> {
    parser.last_esm_import_order += 1;
    let span = statement.span();
    let clean_dep = ConstDependency::new(span.real_lo(), span.real_hi(), "".into(), None);
    parser.presentational_dependencies.push(Box::new(clean_dep));
    let side_effect_dep = ESMImportSideEffectDependency::new(
      source.clone(),
      parser.last_esm_import_order,
      span.into(),
      statement.source_span().into(),
      DependencyType::EsmExport,
      matches!(statement, ExportImport::All(_)),
      statement.get_with_obj().map(get_attributes),
      Some(parser.source_map.clone()),
    );
    parser.dependencies.push(Box::new(side_effect_dep));
    Some(true)
  }

  fn export_specifier(
    &self,
    parser: &mut JavascriptParser,
    statement: ExportLocal,
    local_id: &Atom,
    export_name: &Atom,
  ) -> Option<bool> {
    InnerGraphPlugin::add_variable_usage(
      parser,
      local_id,
      InnerGraphMapUsage::Value(export_name.clone()),
    );
    parser
      .build_info
      .esm_named_exports
      .insert(export_name.clone());
    let dep = if let Some(settings) = parser.get_tag_data(local_id, ESM_SPECIFIER_TAG) {
      let settings = ESMSpecifierData::downcast(settings);
      Box::new(ESMExportImportedSpecifierDependency::new(
        settings.source,
        settings.source_order,
        settings.ids,
        Some(export_name.clone()),
        false,
        None,
        statement.span().into(),
        ESMExportImportedSpecifierDependency::create_export_presence_mode(
          parser.javascript_options,
        ),
        settings.attributes,
        Some(parser.source_map.clone()),
      )) as BoxDependency
    } else {
      Box::new(ESMExportSpecifierDependency::new(
        export_name.clone(),
        local_id.clone(),
        statement.span().into(),
        Some(parser.source_map.clone()),
      ))
    };
    let is_asi_safe = !parser.is_asi_position(statement.span_lo());
    if !is_asi_safe {
      parser.set_asi_position(statement.span_hi());
    }
    parser.dependencies.push(dep);
    Some(true)
  }

  fn export_import_specifier(
    &self,
    parser: &mut JavascriptParser,
    statement: ExportImport,
    source: &Atom,
    local_id: Option<&Atom>,
    export_name: Option<&Atom>,
  ) -> Option<bool> {
    let star_exports = if let Some(export_name) = export_name {
      parser
        .build_info
        .esm_named_exports
        .insert(export_name.clone());
      None
    } else {
      Some(parser.build_info.all_star_exports.clone())
    };
    let dep = ESMExportImportedSpecifierDependency::new(
      source.clone(),
      parser.last_esm_import_order,
      local_id.map(|id| vec![id.clone()]).unwrap_or_default(),
      export_name.cloned(),
      local_id.is_some(),
      star_exports,
      statement.span().into(),
      ESMExportImportedSpecifierDependency::create_export_presence_mode(parser.javascript_options),
      None,
      Some(parser.source_map.clone()),
    );
    if export_name.is_none() {
      parser.build_info.all_star_exports.push(dep.id);
    }
    let is_asi_safe = !parser.is_asi_position(statement.span_lo());
    if !is_asi_safe {
      parser.set_asi_position(statement.span_hi());
    }
    parser.dependencies.push(Box::new(dep));
    Some(true)
  }

  fn export_expression(
    &self,
    parser: &mut JavascriptParser,
    statement: ExportDefaultDeclaration,
    expr: ExportDefaultExpression,
  ) -> Option<bool> {
    let expr_span = expr.span();
    let statement_span = statement.span();

    let dep: ESMExportExpressionDependency = ESMExportExpressionDependency::new(
      expr_span.into(),
      statement_span.into(),
      parser
        .comments
        .and_then(|c| c.get_leading(expr_span.lo))
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
        .unwrap_or_default(),
      match expr {
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
      },
      Some(parser.source_map.clone()),
    );
    parser.dependencies.push(Box::new(dep));
    InnerGraphPlugin::add_variable_usage(
      parser,
      expr.ident().unwrap_or_else(|| &DEFAULT_STAR_JS_WORD),
      InnerGraphMapUsage::Value(JS_DEFAULT_KEYWORD.clone()),
    );
    Some(true)
  }
}
