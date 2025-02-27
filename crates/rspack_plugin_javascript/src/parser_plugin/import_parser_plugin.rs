use itertools::Itertools;
use rspack_core::{
  AsyncDependenciesBlock, ContextDependency, DependencyRange, DynamicImportMode, GroupOptions,
  ImportAttributes, SharedSourceMap,
};
use rspack_core::{ChunkGroupOptions, DynamicImportFetchPriority};
use rspack_core::{ContextNameSpaceObject, ContextOptions, DependencyCategory, SpanExt};
use rspack_error::miette::Severity;
use swc_core::common::Spanned;
use swc_core::ecma::ast::CallExpr;
use swc_core::ecma::atoms::Atom;

use super::JavascriptParserPlugin;
use crate::dependency::{ImportContextDependency, ImportDependency, ImportEagerDependency};
use crate::utils::object_properties::{get_attributes, get_value_by_obj_prop};
use crate::visitors::{
  context_reg_exp, create_context_dependency, create_traceable_error, parse_order_string,
  ContextModuleScanResult, JavascriptParser,
};
use crate::webpack_comment::try_extract_webpack_magic_comment;

pub struct ImportParserPlugin;

impl JavascriptParserPlugin for ImportParserPlugin {
  fn import_call(&self, parser: &mut JavascriptParser, node: &CallExpr) -> Option<bool> {
    let dyn_imported = node.args.first()?;
    if dyn_imported.spread.is_some() {
      return None;
    }
    let dynamic_import_mode = parser.javascript_options.dynamic_import_mode;
    let dynamic_import_preload = parser
      .javascript_options
      .dynamic_import_preload
      .expect("should have dynamic_import_preload")
      .get_order();
    let dynamic_import_prefetch = parser
      .javascript_options
      .dynamic_import_prefetch
      .expect("should have dynamic_import_prefetch")
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
      .unwrap_or(dynamic_import_mode.expect("should have dynamic_import_mode"));
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

    let attributes = get_attributes_from_call_expr(node);
    let param = parser.evaluate_expression(dyn_imported.expr.as_ref());

    if param.is_string() {
      if matches!(mode, DynamicImportMode::Eager) {
        let dep = ImportEagerDependency::new(
          param.string().as_str().into(),
          node.span.into(),
          exports,
          attributes,
        );
        parser.dependencies.push(Box::new(dep));
        return Some(true);
      }
      let dep = Box::new(ImportDependency::new(
        param.string().as_str().into(),
        node.span.into(),
        exports,
        attributes,
      ));
      let source_map: SharedSourceMap = parser.source_map.clone();
      let mut block = AsyncDependenciesBlock::new(
        *parser.module_identifier,
        Into::<DependencyRange>::into(node.span).to_loc(Some(&source_map)),
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
      if matches!(parser.javascript_options.import_dynamic, Some(false)) {
        return None;
      }

      let ContextModuleScanResult {
        context,
        reg,
        query,
        fragment,
        replaces,
        critical,
      } = create_context_dependency(&param, parser);
      parser.walk_expression(&dyn_imported.expr);

      let reg_exp = context_reg_exp(&reg, "", Some(dyn_imported.span().into()), parser);
      let mut dep = ImportContextDependency::new(
        ContextOptions {
          mode: mode.into(),
          recursive: true,
          reg_exp,
          include,
          exclude,
          category: DependencyCategory::Esm,
          request: format!("{}{}{}", context.clone(), query, fragment),
          context,
          namespace_object: if parser.build_meta.strict_esm_module {
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
          attributes,
        },
        node.span().into(),
        dyn_imported.span().into(),
        parser.in_try,
      );
      *dep.critical_mut() = critical;
      parser.dependencies.push(Box::new(dep));
      Some(true)
    }
  }
}

fn get_attributes_from_call_expr(node: &CallExpr) -> Option<ImportAttributes> {
  node
    .args
    .get(1)
    .and_then(|arg| arg.expr.as_object())
    .and_then(|obj| get_value_by_obj_prop(obj, "with"))
    .and_then(|expr| expr.as_object())
    .map(get_attributes)
}
