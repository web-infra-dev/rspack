use rspack_core::{BoxDependency, ConstDependency, DependencyLocation, DependencyType, SpanExt};
use swc_core::atoms::Atom;
use swc_core::common::Spanned;

use super::harmony_import_dependency_parser_plugin::{HarmonySpecifierData, HARMONY_SPECIFIER_TAG};
use super::{
  InnerGraphMapUsage, InnerGraphPlugin, JavascriptParserPlugin, DEFAULT_STAR_JS_WORD,
  JS_DEFAULT_KEYWORD,
};
use crate::dependency::{
  DeclarationId, DeclarationInfo, HarmonyExportExpressionDependency, HarmonyExportHeaderDependency,
  HarmonyExportImportedSpecifierDependency, HarmonyExportSpecifierDependency,
  HarmonyImportSideEffectDependency,
};
use crate::visitors::{
  ExportDefaultDeclaration, ExportDefaultExpression, ExportImport, ExportLocal, JavascriptParser,
  TagInfoData,
};

pub struct HarmonyExportDependencyParserPlugin;

impl JavascriptParserPlugin for HarmonyExportDependencyParserPlugin {
  fn export(&self, parser: &mut JavascriptParser, statement: ExportLocal) -> Option<bool> {
    let span = statement.span();
    let dep = HarmonyExportHeaderDependency::new(
      statement.declaration_span().map(|span| {
        DependencyLocation::new(
          span.real_lo(),
          span.real_hi(),
          Some(parser.source_map.clone()),
        )
      }),
      DependencyLocation::new(
        span.real_lo(),
        span.real_hi(),
        Some(parser.source_map.clone()),
      ),
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
    parser.last_harmony_import_order += 1;
    let span = statement.span();
    let clean_dep = ConstDependency::new(span.real_lo(), span.real_hi(), "".into(), None);
    parser.presentational_dependencies.push(Box::new(clean_dep));
    let side_effect_dep = HarmonyImportSideEffectDependency::new(
      source.clone(),
      parser.last_harmony_import_order,
      statement.span().into(),
      statement.source_span().into(),
      DependencyType::EsmExport,
      matches!(statement, ExportImport::All(_)),
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
      .harmony_named_exports
      .insert(export_name.clone());
    let dep = if let Some(settings) = parser.get_tag_data(local_id, HARMONY_SPECIFIER_TAG) {
      let settings = HarmonySpecifierData::downcast(settings);
      Box::new(HarmonyExportImportedSpecifierDependency::new(
        settings.source,
        settings.source_order,
        settings.ids,
        Some(export_name.clone()),
        false,
        None,
        statement.span().into(),
        HarmonyExportImportedSpecifierDependency::create_export_presence_mode(
          parser.javascript_options,
        ),
      )) as BoxDependency
    } else {
      Box::new(HarmonyExportSpecifierDependency::new(
        export_name.clone(),
        local_id.clone(),
      ))
    };
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
        .harmony_named_exports
        .insert(export_name.clone());
      None
    } else {
      Some(parser.build_info.all_star_exports.clone())
    };
    let dep = HarmonyExportImportedSpecifierDependency::new(
      source.clone(),
      parser.last_harmony_import_order,
      local_id.map(|id| vec![id.clone()]).unwrap_or_default(),
      export_name.cloned(),
      local_id.is_some(),
      star_exports,
      statement.span().into(),
      HarmonyExportImportedSpecifierDependency::create_export_presence_mode(
        parser.javascript_options,
      ),
    );
    if export_name.is_none() {
      parser.build_info.all_star_exports.push(dep.id);
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
    let dep = HarmonyExportExpressionDependency::new(
      DependencyLocation::new(
        expr_span.real_lo(),
        expr_span.real_hi(),
        Some(parser.source_map.clone()),
      ),
      DependencyLocation::new(
        statement_span.real_lo(),
        statement_span.real_hi(),
        Some(parser.source_map.clone()),
      ),
      match expr {
        ExportDefaultExpression::FnDecl(f) => {
          let start = f.span().real_lo();
          let end = if let Some(first_arg) = f.function.params.first() {
            first_arg.span().real_lo()
          } else {
            f.function.body.span().real_lo()
          };
          Some(DeclarationId::Func(DeclarationInfo {
            range: DependencyLocation::new(start, end, Some(parser.source_map.clone())),
            prefix: format!(
              "{}function{} ",
              if f.function.is_async { "async " } else { "" },
              if f.function.is_generator { "*" } else { "" },
            ),
            suffix: format!(
              r#"({}"#,
              if f.function.params.is_empty() {
                ") "
              } else {
                ""
              }
            ),
          }))
        }
        ExportDefaultExpression::ClassDecl(c) => c
          .ident
          .as_ref()
          .map(|ident| DeclarationId::Id(ident.sym.to_string())),
        ExportDefaultExpression::Expr(_) => None,
      },
    );
    parser.presentational_dependencies.push(Box::new(dep));
    InnerGraphPlugin::add_variable_usage(
      parser,
      expr.ident().unwrap_or_else(|| &DEFAULT_STAR_JS_WORD),
      InnerGraphMapUsage::Value(JS_DEFAULT_KEYWORD.clone()),
    );
    Some(true)
  }
}
