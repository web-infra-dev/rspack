use itertools::Itertools;
use rspack_core::{
  AsyncDependenciesBlock, DependencyLocation, DynamicImportMode, ErrorSpan, GroupOptions,
};
use rspack_core::{ChunkGroupOptions, DynamicImportFetchPriority};
use rspack_core::{ContextNameSpaceObject, ContextOptions, DependencyCategory, SpanExt};
use rspack_error::miette::Severity;
use swc_core::common::Spanned;
use swc_core::ecma::ast::{CallExpr, Callee};
use swc_core::ecma::atoms::Atom;

use super::JavascriptParserPlugin;
use crate::dependency::{ImportContextDependency, ImportDependency, ImportEagerDependency};
use crate::visitors::{
  context_reg_exp, create_context_dependency, create_traceable_error, parse_order_string,
  ContextModuleScanResult, JavascriptParser,
};
use crate::webpack_comment::try_extract_webpack_magic_comment;

pub struct ImportParserPlugin;

impl JavascriptParserPlugin for ImportParserPlugin {
  fn import_call(&self, parser: &mut JavascriptParser, node: &CallExpr) -> Option<bool> {
    let Callee::Import(import_call) = &node.callee else {
      unreachable!()
    };
    let Some(dyn_imported) = node.args.first() else {
      return None;
    };
    if dyn_imported.spread.is_some() {
      return None;
    }
    let dynamic_import_mode = parser.javascript_options.dynamic_import_mode;
    let dynamic_import_preload = parser.javascript_options.dynamic_import_preload.get_order();
    let dynamic_import_prefetch = parser
      .javascript_options
      .dynamic_import_prefetch
      .get_order();
    let dynamic_import_fetch_priority = parser.javascript_options.dynamic_import_fetch_priority;

    let magic_comment_options = try_extract_webpack_magic_comment(
      parser.source_file,
      &parser.comments,
      node.span,
      dyn_imported.span(),
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
      .map(|x| DynamicImportMode::from(x.as_str()))
      .unwrap_or(dynamic_import_mode);
    let chunk_name = magic_comment_options
      .get_webpack_chunk_name()
      .map(|x| x.to_owned());
    let chunk_prefetch = magic_comment_options
      .get_webpack_prefetch()
      .and_then(|x| parse_order_string(x.as_str()))
      .or(dynamic_import_prefetch);
    let chunk_preload = magic_comment_options
      .get_webpack_preload()
      .and_then(|x| parse_order_string(x.as_str()))
      .or(dynamic_import_preload);
    let fetch_priority = magic_comment_options
      .get_fetch_priority()
      .map(|x| DynamicImportFetchPriority::from(x.as_str()))
      .or(dynamic_import_fetch_priority);
    let include = magic_comment_options.get_webpack_include();
    let exclude = magic_comment_options.get_webpack_exclude();
    let mut exports = magic_comment_options.get_webpack_exports().map(|x| {
      x.iter()
        .map(|name| Atom::from(name.to_owned()))
        .collect::<Vec<_>>()
    });

    if let Some(referenced_properties_in_destructuring) =
      parser.destructuring_assignment_properties_for(&node.span())
    {
      if exports.is_some() {
        parser.warning_diagnostics.push(Box::new(
          create_traceable_error(
            "Magic comments parse failed".into(),
            "`webpackExports` could not be used with destructuring assignment.".into(),
            parser.source_file,
            node.span().into(),
          )
          .with_severity(Severity::Warning)
          .with_hide_stack(Some(true)),
        ));
      }
      exports = Some(
        referenced_properties_in_destructuring
          .iter()
          .cloned()
          .map(Atom::from)
          .collect_vec(),
      );
    }

    let param = parser.evaluate_expression(dyn_imported.expr.as_ref());

    if param.is_string() {
      let span = ErrorSpan::from(node.span);
      if matches!(mode, DynamicImportMode::Eager) {
        let dep = ImportEagerDependency::new(
          node.span.real_lo(),
          node.span.real_hi(),
          param.string().as_str().into(),
          Some(span),
          exports,
        );
        parser.dependencies.push(Box::new(dep));
        return Some(true);
      }
      let dep = Box::new(ImportDependency::new(
        node.span.real_lo(),
        node.span.real_hi(),
        param.string().as_str().into(),
        Some(span),
        exports,
      ));
      let mut block = AsyncDependenciesBlock::new(
        *parser.module_identifier,
        Some(DependencyLocation::new(
          span.start,
          span.end,
          Some(parser.source_map.clone()),
        )),
        None,
        vec![dep],
        Some(param.string().clone()),
      );
      block.set_group_options(GroupOptions::ChunkGroup(ChunkGroupOptions::new(
        chunk_name,
        chunk_preload,
        chunk_prefetch,
        fetch_priority,
      )));
      parser.blocks.push(Box::new(block));
      Some(true)
    } else {
      let ContextModuleScanResult {
        context,
        reg,
        query,
        fragment,
        replaces,
      } = create_context_dependency(&param, &dyn_imported.expr, parser);
      let reg_exp = context_reg_exp(&reg, "", Some(dyn_imported.span().into()), parser);
      parser
        .dependencies
        .push(Box::new(ImportContextDependency::new(
          import_call.span.real_lo(),
          import_call.span.real_hi(),
          node.span.real_hi(),
          ContextOptions {
            mode: mode.into(),
            recursive: true,
            reg_exp,
            include,
            exclude,
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
              chunk_preload,
              chunk_prefetch,
              fetch_priority,
            ))),
            replaces,
            start: node.span().real_lo(),
            end: node.span().real_hi(),
            referenced_exports: exports,
          },
          Some(node.span.into()),
          parser.in_try,
        )));
      Some(true)
    }
  }
}
