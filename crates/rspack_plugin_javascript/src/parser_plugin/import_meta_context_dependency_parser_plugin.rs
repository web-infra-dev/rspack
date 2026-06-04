use concat_string::concat_string;
use rspack_core::{
  ContextMode, ContextModulePattern, ContextNameSpaceObject, ContextOptions, DependencyCategory,
  ReferencedSpecifier, escape_glob_pattern, extract_glob_base_dir, get_context,
  normalize_path_separators, normalize_path_separators_for_path, unescape_glob_path,
};
use rspack_paths::{Utf8Path, Utf8PathBuf};
use rspack_regex::RspackRegex;
use rspack_util::{SpanExt, identifier::relative_path_to_request, node_path::NodePath};
use sugar_path::SugarPath;
use swc_core::{
  atoms::Atom,
  common::Spanned,
  ecma::ast::{CallExpr, Expr, Lit, PropName},
};

use super::JavascriptParserPlugin;
use crate::{
  dependency::ImportMetaContextDependency,
  utils::{
    eval::{self, BasicEvaluatedExpression},
    object_properties::{
      get_bool_by_obj_prop, get_literal_str_by_obj_prop, get_regex_by_obj_prop,
      get_value_by_obj_prop,
    },
  },
  visitors::{
    JavascriptParser, clean_regexp_in_context_module, default_context_reg_exp, expr_name,
    static_string_from_expr,
  },
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

fn normalize_import_meta_glob_query(query: String) -> String {
  if query.is_empty() || query.starts_with('?') {
    query
  } else {
    concat_string!("?", query)
  }
}

fn static_import_meta_glob_query_from_expr(expr: &Expr) -> Option<String> {
  if let Some(query) = static_string_from_expr(expr) {
    return Some(normalize_import_meta_glob_query(query));
  }

  let query = expr.as_object()?;
  let mut serializer = form_urlencoded::Serializer::new(String::new());
  for prop in &query.props {
    let kv = prop.as_prop().and_then(|prop| prop.as_key_value())?;
    let key = static_import_meta_glob_query_key_from_prop_name(&kv.key)?;
    let value = if let Some(value) = static_string_from_expr(&kv.value) {
      value
    } else {
      match kv.value.as_lit()? {
        Lit::Bool(bool) => bool.value.to_string(),
        Lit::Num(num) => num.value.to_string(),
        _ => return None,
      }
    };
    serializer.append_pair(&key, &value);
  }

  Some(normalize_import_meta_glob_query(serializer.finish()))
}

fn static_import_meta_glob_query_key_from_prop_name(prop_name: &PropName) -> Option<String> {
  match prop_name {
    PropName::Ident(ident) => Some(ident.sym.to_string()),
    PropName::Str(str) => Some(str.value.to_string_lossy().into_owned()),
    PropName::Num(num) => Some(num.value.to_string()),
    PropName::Computed(computed) => static_import_meta_glob_query_key_from_expr(&computed.expr),
    _ => None,
  }
}

fn static_import_meta_glob_query_key_from_expr(expr: &Expr) -> Option<String> {
  if let Some(key) = static_string_from_expr(expr) {
    return Some(key);
  }

  match expr.as_lit()? {
    Lit::Num(num) => Some(num.value.to_string()),
    Lit::Bool(bool) => Some(bool.value.to_string()),
    Lit::Null(_) => Some("null".to_string()),
    _ => None,
  }
}

fn import_meta_glob_path_parts<'a>(
  context: &'a str,
  compiler_context: &'a str,
  path: &'a str,
) -> (&'a str, &'a str) {
  if let Some(path) = path.strip_prefix('/') {
    (compiler_context, path)
  } else {
    (context, path)
  }
}

fn join_import_meta_glob_path(base: &str, path: &str) -> String {
  normalize_path_separators(
    Utf8Path::new(base)
      .node_join_posix(path)
      .node_normalize_posix()
      .as_ref(),
  )
}

fn join_import_meta_glob_fs_path(base: &str, path: &str) -> String {
  normalize_path_separators_for_path(
    Utf8Path::new(base)
      .node_join_posix(path)
      .node_normalize_posix()
      .as_ref(),
  )
}

struct ResolvedContextModuleGlobPattern {
  absolute_pattern: String,
  absolute_base: String,
  negative: bool,
}

fn resolve_glob_pattern(
  pattern: &str,
  context: &str,
  compiler_context: &str,
) -> ResolvedContextModuleGlobPattern {
  let (pattern, negative) = if let Some(pattern) = pattern.strip_prefix('!') {
    (pattern, true)
  } else {
    (pattern, false)
  };
  let pattern = normalize_path_separators(pattern);
  let (base, pattern_to_join) =
    import_meta_glob_path_parts(context, compiler_context, pattern.as_str());
  let base = normalize_path_separators_for_path(base);
  let escaped_base = escape_glob_pattern(&base);
  let absolute_pattern = join_import_meta_glob_path(&escaped_base, pattern_to_join);
  let absolute_base = unescape_glob_path(extract_glob_base_dir(&absolute_pattern));

  ResolvedContextModuleGlobPattern {
    absolute_pattern,
    absolute_base,
    negative,
  }
}

fn common_glob_base_dir(patterns: &[ResolvedContextModuleGlobPattern], fallback: &str) -> String {
  let mut positive_patterns = patterns.iter().filter(|pattern| !pattern.negative);
  let Some(first) = positive_patterns.next() else {
    return fallback.to_string();
  };

  let mut common_base = Utf8PathBuf::from(first.absolute_base.as_str());
  for pattern in positive_patterns {
    let base = Utf8Path::new(pattern.absolute_base.as_str());
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
    concat_string!(common_base, "/")
  }
}

fn resolve_import_meta_glob_context(
  context: &str,
  compiler_context: &str,
  base: Option<&str>,
) -> String {
  let Some(base) = base else {
    return context.to_string();
  };

  let base = normalize_path_separators_for_path(base);
  let (base_context, path_to_join) =
    import_meta_glob_path_parts(context, compiler_context, base.as_str());
  join_import_meta_glob_fs_path(base_context, path_to_join)
}

fn absolute_path_to_glob_request(context: &str, absolute_path: &str) -> String {
  let relative_path = absolute_path.as_path().relative(context);
  let relative_path = relative_path.to_string_lossy();
  let relative_path = normalize_path_separators_for_path(&relative_path);
  relative_path_to_request(&relative_path).into_owned()
}

fn normalize_base_glob_pattern(
  pattern: String,
  base_context: &str,
  compiler_context: &str,
) -> String {
  let (negative, pattern) = if let Some(pattern) = pattern.strip_prefix('!') {
    (true, pattern)
  } else {
    (false, pattern.as_str())
  };

  let pattern = normalize_path_separators(pattern);
  let Some(pattern) = pattern.strip_prefix('/') else {
    return if negative {
      concat_string!("!", pattern)
    } else {
      pattern
    };
  };

  let compiler_context = normalize_path_separators_for_path(compiler_context);
  let absolute_pattern = join_import_meta_glob_path(&compiler_context, pattern);
  let relative_pattern = absolute_path_to_glob_request(base_context, &absolute_pattern);

  if negative {
    concat_string!("!", relative_pattern)
  } else {
    relative_pattern
  }
}

fn normalize_import_meta_glob_patterns(
  patterns: Vec<String>,
  base_context: &str,
  compiler_context: &str,
  has_custom_base: bool,
) -> Vec<String> {
  if has_custom_base {
    patterns
      .into_iter()
      .map(|pattern| normalize_base_glob_pattern(pattern, base_context, compiler_context))
      .collect()
  } else {
    patterns
  }
}

fn glob_patterns_are_recursive(
  patterns: &[ResolvedContextModuleGlobPattern],
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
          .unwrap_or(pattern.absolute_pattern.as_str())
          .contains('/')
    })
}

fn create_import_meta_context_dependency(
  node: &CallExpr,
  parser: &mut JavascriptParser,
) -> Option<ImportMetaContextDependency> {
  assert!(node.callee.is_expr());
  let dyn_imported = node.args.first()?;
  if dyn_imported.spread.is_some() {
    return None;
  }
  // TODO: should've used expression evaluation to handle cases like `abc${"efg"}`, etc.
  let context = static_string_from_expr(&dyn_imported.expr)?;
  let context_options = if let Some(obj) = node.args.get(1).and_then(|arg| arg.expr.as_object()) {
    let regexp = get_regex_by_obj_prop(obj, "regExp");
    let regexp_span = regexp.map(|r| r.span().into());
    let regexp = regexp.map_or_else(default_context_reg_exp, |regexp| {
      RspackRegex::try_from(regexp).expect("reg failed")
    });
    let include = get_regex_by_obj_prop(obj, "include")
      .map(|regexp| RspackRegex::try_from(regexp).expect("reg failed"));
    let exclude = get_regex_by_obj_prop(obj, "exclude")
      .map(|regexp| RspackRegex::try_from(regexp).expect("reg failed"));
    let mode = get_literal_str_by_obj_prop(obj, "mode").map_or(ContextMode::Sync, |s| {
      s.value.to_string_lossy().as_ref().into()
    });
    let recursive = get_bool_by_obj_prop(obj, "recursive").is_none_or(|bool| bool.value);
    ContextOptions {
      pattern: clean_regexp_in_context_module(regexp, regexp_span, parser).into(),
      include,
      exclude,
      recursive,
      category: DependencyCategory::Esm,
      request: context.clone(),
      context,
      mode,
      start: node.span().real_lo(),
      end: node.span().real_hi(),
      ..Default::default()
    }
  } else {
    ContextOptions {
      recursive: true,
      mode: ContextMode::Sync,
      pattern: clean_regexp_in_context_module(default_context_reg_exp(), None, parser).into(),
      category: DependencyCategory::Esm,
      request: context.clone(),
      context,
      start: node.span().real_lo(),
      end: node.span().real_hi(),
      ..Default::default()
    }
  };
  Some(ImportMetaContextDependency::new(
    context_options,
    node.span.into(),
    parser.in_try,
  ))
}

fn create_import_meta_glob_dependency(
  node: &CallExpr,
  parser: &mut JavascriptParser,
) -> Option<ImportMetaContextDependency> {
  assert!(node.callee.is_expr());
  let dyn_imported = node.args.first()?;
  if dyn_imported.spread.is_some() {
    return None;
  }
  let raw_glob_patterns = static_glob_patterns_from_expr(&dyn_imported.expr)?;
  let importer_context = get_context(parser.resource_data);
  let glob_options = node.args.get(1).and_then(|arg| arg.expr.as_object());
  let mode = glob_options.map_or(ContextMode::Lazy, |obj| {
    if get_bool_by_obj_prop(obj, "eager").is_some_and(|b| b.value) {
      ContextMode::Sync
    } else {
      ContextMode::Lazy
    }
  });
  let glob_import = glob_options
    .and_then(|obj| get_literal_str_by_obj_prop(obj, "import"))
    .map(|s| s.value.to_string_lossy().into_owned());
  let glob_query = glob_options
    .and_then(|obj| get_value_by_obj_prop(obj, "query"))
    .and_then(static_import_meta_glob_query_from_expr)
    .unwrap_or_default();
  let base = glob_options
    .and_then(|obj| get_value_by_obj_prop(obj, "base"))
    .and_then(static_string_from_expr);
  let glob_exhaustive = glob_options
    .is_some_and(|obj| get_bool_by_obj_prop(obj, "exhaustive").is_some_and(|b| b.value));
  let context = resolve_import_meta_glob_context(
    importer_context.as_str(),
    parser.compiler_options.context.as_str(),
    base.as_deref(),
  );
  let glob_patterns = normalize_import_meta_glob_patterns(
    raw_glob_patterns,
    context.as_str(),
    parser.compiler_options.context.as_str(),
    base.is_some(),
  );
  let resolved_glob_patterns = glob_patterns
    .iter()
    .map(|pattern| {
      resolve_glob_pattern(
        pattern,
        context.as_str(),
        parser.compiler_options.context.as_str(),
      )
    })
    .collect::<Vec<_>>();
  let base_dir = common_glob_base_dir(&resolved_glob_patterns, context.as_str());
  let recursive = glob_patterns_are_recursive(&resolved_glob_patterns, &base_dir);

  let referenced_specifiers = glob_import
    .as_ref()
    .filter(|import| import.as_str() != "*")
    .map(|import| vec![ReferencedSpecifier::new(vec![Atom::from(import.as_str())])]);
  let namespace_object = if parser.build_meta.strict_esm_module {
    ContextNameSpaceObject::Strict
  } else {
    ContextNameSpaceObject::Bool(true)
  };

  let context_options = ContextOptions {
    pattern: ContextModulePattern::Glob(glob_patterns),
    recursive,
    category: DependencyCategory::Esm,
    request: concat_string!(base_dir, glob_query),
    context,
    namespace_object,
    mode,
    start: node.span().real_lo(),
    end: node.span().real_hi(),
    referenced_specifiers,
    glob_import,
    glob_exhaustive,
    ..Default::default()
  };
  Some(ImportMetaContextDependency::new_glob(
    context_options,
    node.span.into(),
    parser.in_try,
  ))
}

pub struct ImportMetaContextDependencyParserPlugin;

#[rspack_macros::implemented_javascript_parser_hooks]
impl JavascriptParserPlugin for ImportMetaContextDependencyParserPlugin {
  fn evaluate_identifier(
    &self,
    _parser: &mut JavascriptParser,
    for_name: &str,
    start: u32,
    end: u32,
  ) -> Option<BasicEvaluatedExpression<'static>> {
    let name = match for_name {
      expr_name::IMPORT_META_CONTEXT => expr_name::IMPORT_META_CONTEXT,
      expr_name::IMPORT_META_GLOB => expr_name::IMPORT_META_GLOB,
      _ => return None,
    };

    Some(eval::evaluate_to_identifier(
      name.into(),
      expr_name::IMPORT_META.into(),
      Some(true),
      start,
      end,
    ))
  }

  fn call(
    &self,
    parser: &mut JavascriptParser,
    expr: &swc_core::ecma::ast::CallExpr,
    for_name: &str,
  ) -> Option<bool> {
    if expr.args.is_empty() || expr.args.len() > 2 {
      return None;
    }

    let dep = match for_name {
      expr_name::IMPORT_META_CONTEXT => create_import_meta_context_dependency(expr, parser),
      expr_name::IMPORT_META_GLOB => create_import_meta_glob_dependency(expr, parser),
      _ => None,
    };

    if let Some(dep) = dep {
      parser.add_dependency(Box::new(dep));
      Some(true)
    } else {
      None
    }
  }
}
