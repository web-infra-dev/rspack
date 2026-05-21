use rspack_core::{
  ContextMode, ContextModuleGlobPattern, ContextModulePattern, ContextNameSpaceObject,
  ContextOptions, DependencyCategory, get_context, normalize_path_separators,
};
use rspack_paths::{Utf8Path, Utf8PathBuf};
use rspack_util::{SpanExt, node_path::NodePath};
use swc_core::{
  common::Spanned,
  ecma::ast::{CallExpr, Expr},
};

use super::JavascriptParserPlugin;
use crate::{
  dependency::ImportMetaGlobDependency,
  utils::{
    eval::{self, BasicEvaluatedExpression},
    object_properties::get_bool_by_obj_prop,
  },
  visitors::{JavascriptParser, expr_name, static_string_from_expr},
};

fn static_glob_patterns_from_expr(expr: &Expr) -> Option<Vec<String>> {
  if let Some(pattern) = static_string_from_expr(expr) {
    return Some(vec![pattern]);
  }

  let array = expr.as_array()?;
  array
    .elems
    .iter()
    .map(|elem| {
      let elem = elem.as_ref()?;
      if elem.spread.is_some() {
        return None;
      }
      static_string_from_expr(&elem.expr)
    })
    .collect()
}

fn create_glob_pattern(
  pattern: String,
  context: &str,
  compiler_context: &str,
) -> ContextModuleGlobPattern {
  let (pattern, negative) = if let Some(pattern) = pattern.strip_prefix('!') {
    (pattern.to_string(), true)
  } else {
    (pattern, false)
  };
  let (base, pattern_to_join) = if let Some(pattern) = pattern.strip_prefix('/') {
    (compiler_context, pattern)
  } else {
    (context, pattern.as_str())
  };
  let absolute_pattern = Utf8Path::new(base)
    .node_join_posix(pattern_to_join)
    .node_normalize_posix()
    .to_string();
  let absolute_pattern = normalize_path_separators(&absolute_pattern);

  ContextModuleGlobPattern::new(pattern, absolute_pattern, negative)
}

fn common_glob_base_dir(patterns: &[ContextModuleGlobPattern], fallback: &str) -> String {
  let mut positive_patterns = patterns.iter().filter(|pattern| !pattern.negative);
  let Some(first) = positive_patterns.next() else {
    return fallback.to_string();
  };

  let mut common_base = Utf8PathBuf::from(first.absolute_base.as_ref());
  for pattern in positive_patterns {
    let base = Utf8Path::new(pattern.absolute_base.as_ref());
    while !base.starts_with(&common_base) {
      let Some(parent) = common_base.parent() else {
        return fallback.to_string();
      };
      common_base = parent.to_path_buf();
    }
  }

  let common_base = common_base.as_str();
  if common_base.ends_with('/') {
    common_base.to_string()
  } else {
    format!("{common_base}/")
  }
}

fn glob_patterns_are_recursive(
  patterns: &[ContextModuleGlobPattern],
  common_base_dir: &str,
) -> bool {
  patterns
    .iter()
    .filter(|pattern| !pattern.negative)
    .any(|pattern| {
      pattern.absolute_pattern.contains("**")
        || pattern
          .absolute_pattern
          .strip_prefix(common_base_dir)
          .unwrap_or(pattern.absolute_pattern.as_ref())
          .contains('/')
    })
}

fn create_import_meta_glob_dependency(
  node: &CallExpr,
  parser: &mut JavascriptParser,
) -> Option<ImportMetaGlobDependency> {
  assert!(node.callee.is_expr());
  let dyn_imported = node.args.first()?;
  if dyn_imported.spread.is_some() {
    return None;
  }
  let glob_patterns = static_glob_patterns_from_expr(&dyn_imported.expr)?;
  let context = get_context(parser.resource_data);
  let glob_patterns = glob_patterns
    .into_iter()
    .map(|pattern| {
      create_glob_pattern(
        pattern,
        context.as_str(),
        parser.compiler_options.context.as_str(),
      )
    })
    .collect::<Vec<_>>();
  let base_dir = common_glob_base_dir(&glob_patterns, context.as_str());
  let recursive = glob_patterns_are_recursive(&glob_patterns, &base_dir);

  let mode = node
    .args
    .get(1)
    .and_then(|arg| arg.expr.as_object())
    .map_or(ContextMode::Lazy, |obj| {
      if get_bool_by_obj_prop(obj, "eager").is_some_and(|b| b.value) {
        ContextMode::Sync
      } else {
        ContextMode::Lazy
      }
    });
  let namespace_object = if parser.build_meta.strict_esm_module {
    ContextNameSpaceObject::Strict
  } else {
    ContextNameSpaceObject::Bool(true)
  };

  let context_options = ContextOptions {
    pattern: ContextModulePattern::Glob(glob_patterns),
    recursive,
    category: DependencyCategory::Esm,
    request: base_dir.clone(),
    context: base_dir,
    namespace_object,
    mode,
    start: node.span().real_lo(),
    end: node.span().real_hi(),
    ..Default::default()
  };
  Some(ImportMetaGlobDependency::new(
    context_options,
    node.span.into(),
    parser.in_try,
  ))
}

pub struct ImportMetaGlobDependencyParserPlugin;

#[rspack_macros::implemented_javascript_parser_hooks]
impl JavascriptParserPlugin for ImportMetaGlobDependencyParserPlugin {
  fn evaluate_identifier(
    &self,
    _parser: &mut JavascriptParser,
    for_name: &str,
    start: u32,
    end: u32,
  ) -> Option<BasicEvaluatedExpression<'static>> {
    if for_name == expr_name::IMPORT_META_GLOB {
      Some(eval::evaluate_to_identifier(
        expr_name::IMPORT_META_GLOB.into(),
        expr_name::IMPORT_META.into(),
        Some(true),
        start,
        end,
      ))
    } else {
      None
    }
  }

  fn call(
    &self,
    parser: &mut JavascriptParser,
    expr: &swc_core::ecma::ast::CallExpr,
    for_name: &str,
  ) -> Option<bool> {
    if for_name != expr_name::IMPORT_META_GLOB || expr.args.is_empty() || expr.args.len() > 2 {
      None
    } else if let Some(dep) = create_import_meta_glob_dependency(expr, parser) {
      parser.add_dependency(Box::new(dep));
      Some(true)
    } else {
      None
    }
  }
}
