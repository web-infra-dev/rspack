use rspack_core::{
  context_reg_exp, AsyncDependenciesBlock, DependencyLocation, DynamicImportMode, ErrorSpan,
  GroupOptions,
};
use rspack_core::{ChunkGroupOptions, ContextMode};
use rspack_core::{ContextNameSpaceObject, ContextOptions, DependencyCategory, SpanExt};
use swc_core::common::Spanned;
use swc_core::ecma::ast::{CallExpr, Callee, Expr, Lit};
use swc_core::ecma::atoms::Atom;

use super::JavascriptParserPlugin;
use crate::dependency::{ImportContextDependency, ImportDependency, ImportEagerDependency};
use crate::visitors::{parse_order_string, scanner_context_module, ContextModuleScanResult};
use crate::webpack_comment::try_extract_webpack_magic_comment;

pub struct ImportParserPlugin;

impl JavascriptParserPlugin for ImportParserPlugin {
  fn import_call(
    &self,
    parser: &mut crate::visitors::JavascriptParser,
    node: &CallExpr,
  ) -> Option<bool> {
    let Callee::Import(import_call) = &node.callee else {
      unreachable!()
    };
    let Some(dyn_imported) = node.args.first() else {
      return None;
    };
    if dyn_imported.spread.is_some() {
      return None;
    }
    let dynamic_import_mode = parser
      .javascript_options
      .map(|o| o.dynamic_import_mode)
      .unwrap_or_default();

    let dynamic_import_preload = parser
      .javascript_options
      .map(|o| o.dynamic_import_preload)
      .and_then(|o| o.get_order());

    let dynamic_import_prefetch = parser
      .javascript_options
      .map(|o| o.dynamic_import_prefetch)
      .and_then(|o| o.get_order());

    match dyn_imported.expr.as_ref() {
      Expr::Lit(Lit::Str(imported)) => {
        let magic_comment_options = try_extract_webpack_magic_comment(
          parser.source_file,
          &parser.comments,
          node.span,
          imported.span,
          &mut parser.warning_diagnostics,
        );
        if magic_comment_options
          .get_webpack_ignore()
          .unwrap_or_default()
        {
          return None;
        }
        let mode = magic_comment_options
          .get_webpack_mode()
          .map(|x| DynamicImportMode::from(x.as_str()));
        let chunk_name = magic_comment_options
          .get_webpack_chunk_name()
          .map(|x| x.to_owned());
        let chunk_prefetch = magic_comment_options
          .get_webpack_prefetch()
          .and_then(|x| parse_order_string(x.as_str()));
        let chunk_preload = magic_comment_options
          .get_webpack_preload()
          .and_then(|x| parse_order_string(x.as_str()));
        let span = ErrorSpan::from(node.span);
        if matches!(
          mode.unwrap_or(dynamic_import_mode),
          DynamicImportMode::Eager
        ) {
          let dep = ImportEagerDependency::new(
            node.span.real_lo(),
            node.span.real_hi(),
            imported.value.clone(),
            Some(span),
            // TODO scan dynamic import referenced exports
            None,
          );
          parser.dependencies.push(Box::new(dep));
          return Some(true);
        }
        let dep = Box::new(ImportDependency::new(
          node.span.real_lo(),
          node.span.real_hi(),
          imported.value.clone(),
          Some(span),
          // TODO scan dynamic import referenced exports
          None,
        ));
        let mut block = AsyncDependenciesBlock::new(
          *parser.module_identifier,
          Some(DependencyLocation::new(span.start, span.end)),
          None,
          vec![dep],
        );
        block.set_group_options(GroupOptions::ChunkGroup(ChunkGroupOptions::new(
          chunk_name,
          chunk_preload.or(dynamic_import_preload),
          chunk_prefetch.or(dynamic_import_prefetch),
        )));
        parser.blocks.push(block);
        Some(true)
      }
      Expr::Tpl(tpl) if tpl.quasis.len() == 1 => {
        let magic_comment_options = try_extract_webpack_magic_comment(
          parser.source_file,
          &parser.comments,
          node.span,
          tpl.span,
          &mut parser.warning_diagnostics,
        );
        let mode = magic_comment_options
          .get_webpack_mode()
          .map(|x| DynamicImportMode::from(x.as_str()));
        let chunk_name = magic_comment_options
          .get_webpack_chunk_name()
          .map(|x| x.to_owned());
        let chunk_prefetch = magic_comment_options
          .get_webpack_prefetch()
          .and_then(|x| parse_order_string(x.as_str()));
        let chunk_preload = magic_comment_options
          .get_webpack_preload()
          .and_then(|x| parse_order_string(x.as_str()));
        let request = Atom::from(
          tpl
            .quasis
            .first()
            .expect("should have one quasis")
            .raw
            .to_string(),
        );
        let span = ErrorSpan::from(node.span);
        if matches!(
          mode.unwrap_or(dynamic_import_mode),
          DynamicImportMode::Eager
        ) {
          let dep = ImportEagerDependency::new(
            node.span.real_lo(),
            node.span.real_hi(),
            request,
            Some(span),
            // TODO scan dynamic import referenced exports
            None,
          );
          parser.dependencies.push(Box::new(dep));
          return Some(true);
        }
        let dep = Box::new(ImportDependency::new(
          node.span.real_lo(),
          node.span.real_hi(),
          request,
          Some(span),
          None,
        ));
        let mut block = AsyncDependenciesBlock::new(
          *parser.module_identifier,
          Some(DependencyLocation::new(span.start, span.end)),
          None,
          vec![dep],
        );
        block.set_group_options(GroupOptions::ChunkGroup(ChunkGroupOptions::new(
          chunk_name,
          chunk_preload.or(dynamic_import_preload),
          chunk_prefetch.or(dynamic_import_prefetch),
        )));
        parser.blocks.push(block);
        // FIXME: align `parser.walk_expression` to webpack, which put into `context_dependency_helper`
        parser.walk_template_expression(tpl);
        Some(true)
      }
      _ => {
        let Some(ContextModuleScanResult {
          context,
          reg,
          query,
          fragment,
          replaces,
        }) = scanner_context_module(dyn_imported.expr.as_ref())
        else {
          return None;
        };
        let magic_comment_options = try_extract_webpack_magic_comment(
          parser.source_file,
          &parser.comments,
          node.span,
          dyn_imported.span(),
          &mut parser.warning_diagnostics,
        );
        let _mode = magic_comment_options
          .get_webpack_mode()
          .map(|x| DynamicImportMode::from(x.as_str()));
        let chunk_name = magic_comment_options
          .get_webpack_chunk_name()
          .map(|x| x.to_owned());
        let chunk_prefetch = magic_comment_options
          .get_webpack_prefetch()
          .and_then(|x| parse_order_string(x.as_str()));
        let chunk_preload = magic_comment_options
          .get_webpack_preload()
          .and_then(|x| parse_order_string(x.as_str()));
        parser
          .dependencies
          .push(Box::new(ImportContextDependency::new(
            import_call.span.real_lo(),
            import_call.span.real_hi(),
            node.span.real_hi(),
            ContextOptions {
              mode: ContextMode::Lazy,
              recursive: true,
              reg_exp: context_reg_exp(&reg, ""),
              include: None,
              exclude: None,
              category: DependencyCategory::Esm,
              request: format!("{}{}{}", context.clone(), query, fragment),
              context,
              namespace_object: if parser.build_meta.strict_harmony_module {
                ContextNameSpaceObject::Strict
              } else {
                ContextNameSpaceObject::Bool(true)
              },
              group_options: Some(GroupOptions::ChunkGroup(ChunkGroupOptions::new(
                chunk_name,
                chunk_preload.or(dynamic_import_preload),
                chunk_prefetch.or(dynamic_import_prefetch),
              ))),
              start: node.span().real_lo(),
              end: node.span().real_hi(),
            },
            replaces,
            Some(node.span.into()),
          )));
        // FIXME: align `parser.walk_expression` to webpack, which put into `context_dependency_helper`
        parser.walk_expression(&dyn_imported.expr);
        Some(true)
      }
    }
  }
}
