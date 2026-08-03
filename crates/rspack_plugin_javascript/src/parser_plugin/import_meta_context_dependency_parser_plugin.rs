use concat_string::concat_string;
use rspack_core::{
  ContextMode, ContextModulePattern, ContextNameSpaceObject, ContextOptions, DependencyCategory,
  ReferencedSpecifier, compile_context_module_glob_request, get_context, normalize_path_separators,
  normalize_path_separators_for_path,
};
use rspack_paths::Utf8Path;
use rspack_regex::RspackRegex;
use rspack_util::{SpanExt, identifier::relative_path_to_request, node_path::NodePath};
use sugar_path::SugarPath;
use swc_atoms::Atom;
use swc_experimental_ecma_ast::{CallExpr, Expr, GetSpan, Lit, PropName};

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
  let request = static_string_from_expr(&dyn_imported.expr)?;
  let context_options = if let Some(obj) = node.args.get(1).and_then(|arg| arg.expr.as_object()) {
    let regexp = get_regex_by_obj_prop(obj, "regExp");
    let regexp_span = regexp.map(|r| r.span().into());
    let regexp = regexp.map_or_else(default_context_reg_exp, |regexp| {
      RspackRegex::with_flags(regexp.exp.as_str(), regexp.flags.as_str()).expect("reg failed")
    });
    let include = get_regex_by_obj_prop(obj, "include").map(|regexp| {
      RspackRegex::with_flags(regexp.exp.as_str(), regexp.flags.as_str()).expect("reg failed")
    });
    let exclude = get_regex_by_obj_prop(obj, "exclude").map(|regexp| {
      RspackRegex::with_flags(regexp.exp.as_str(), regexp.flags.as_str()).expect("reg failed")
    });
    let mode = get_literal_str_by_obj_prop(obj, "mode").map_or(ContextMode::Sync, |s| {
      s.value.to_string_lossy().as_ref().into()
    });
    let recursive = get_bool_by_obj_prop(obj, "recursive").is_none_or(|bool| bool.value);
    let span = node.span;
    ContextOptions {
      pattern: clean_regexp_in_context_module(regexp, regexp_span, parser).into(),
      include,
      exclude,
      recursive,
      category: DependencyCategory::Esm,
      request,
      context: get_context(parser.resource_data).to_string(),
      compiler_context: parser.compiler_options.context.clone(),
      mode,
      start: span.real_lo(),
      end: span.real_hi(),
      ..Default::default()
    }
  } else {
    let span = node.span;
    ContextOptions {
      recursive: true,
      mode: ContextMode::Sync,
      pattern: clean_regexp_in_context_module(default_context_reg_exp(), None, parser).into(),
      category: DependencyCategory::Esm,
      request,
      context: get_context(parser.resource_data).to_string(),
      compiler_context: parser.compiler_options.context.clone(),
      start: span.real_lo(),
      end: span.real_hi(),
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
  let compiled = compile_context_module_glob_request(
    &concat_string!(".", glob_query),
    &glob_patterns,
    context.as_str(),
    parser.compiler_options.context.as_str(),
    true,
  );

  let referenced_specifiers = glob_import
    .as_ref()
    .filter(|import| import.as_str() != "*")
    .map(|import| vec![ReferencedSpecifier::new(vec![Atom::from(import.as_str())])]);
  let namespace_object = if parser.build_meta.strict_esm_module() {
    ContextNameSpaceObject::Strict
  } else {
    ContextNameSpaceObject::Bool(true)
  };

  let span = node.span;
  let context_options = ContextOptions {
    pattern: ContextModulePattern::Glob(glob_patterns),
    recursive: compiled.recursive,
    category: DependencyCategory::Esm,
    request: compiled.request,
    context,
    compiler_context: parser.compiler_options.context.clone(),
    namespace_object,
    mode,
    start: span.real_lo(),
    end: span.real_hi(),
    referenced_specifiers,
    glob_import,
    glob_exhaustive,
    ..Default::default()
  };
  Some(ImportMetaContextDependency::new_glob(
    context_options,
    span.into(),
    parser.in_try,
  ))
}

pub struct ImportMetaContextDependencyParserPlugin {
  pub webpack_context: bool,
  pub glob: bool,
}

#[rspack_macros::implemented_javascript_parser_hooks]
impl<'p, 'a> JavascriptParserPlugin<'p, 'a> for ImportMetaContextDependencyParserPlugin {
  fn evaluate_identifier(
    &self,
    _parser: &mut JavascriptParser<'p>,
    for_name: &str,
    _member_expr_info: Option<&crate::visitors::ExpressionExpressionInfo>,
    start: u32,
    end: u32,
  ) -> Option<BasicEvaluatedExpression<'p>> {
    let name = match for_name {
      expr_name::IMPORT_META_CONTEXT if self.webpack_context => expr_name::IMPORT_META_CONTEXT,
      expr_name::IMPORT_META_GLOB if self.glob => expr_name::IMPORT_META_GLOB,
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
    parser: &mut JavascriptParser<'p>,
    expr: &CallExpr,
    for_name: &str,
  ) -> Option<bool> {
    if expr.args.is_empty() || expr.args.len() > 2 {
      return None;
    }

    let dep = match for_name {
      expr_name::IMPORT_META_CONTEXT if self.webpack_context => {
        create_import_meta_context_dependency(expr, parser)
      }
      expr_name::IMPORT_META_GLOB if self.glob => create_import_meta_glob_dependency(expr, parser),
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
