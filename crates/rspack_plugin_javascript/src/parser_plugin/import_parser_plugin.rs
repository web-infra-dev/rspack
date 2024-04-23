use rspack_core::{
  context_reg_exp, AsyncDependenciesBlock, DependencyLocation, DynamicImportMode, ErrorSpan,
  GroupOptions,
};
use rspack_core::{ChunkGroupOptions, ContextMode};
use rspack_core::{ContextNameSpaceObject, ContextOptions, DependencyCategory, SpanExt};
use swc_core::common::Spanned;
use swc_core::ecma::ast::{CallExpr, Callee};

use super::JavascriptParserPlugin;
use crate::dependency::{ImportContextDependency, ImportDependency, ImportEagerDependency};
use crate::visitors::{create_context_dependency, parse_order_string, ContextModuleScanResult};
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

    let magic_comment_options = try_extract_webpack_magic_comment(
      parser.source_file,
      &parser.comments,
      node.span,
      dyn_imported.expr.span(),
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

    let param = parser.evaluate_expression(dyn_imported.expr.as_ref());

    if param.is_string() {
      let span = ErrorSpan::from(node.span);
      if matches!(
        mode.unwrap_or(dynamic_import_mode),
        DynamicImportMode::Eager
      ) {
        let dep = ImportEagerDependency::new(
          node.span.real_lo(),
          node.span.real_hi(),
          param.string().as_str().into(),
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
        param.string().as_str().into(),
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
    } else {
      let ContextModuleScanResult {
        context,
        reg,
        query,
        fragment,
        replaces,
      } = create_context_dependency(&param);
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
