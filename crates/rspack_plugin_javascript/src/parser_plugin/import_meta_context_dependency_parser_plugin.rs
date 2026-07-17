use concat_string::concat_string;
use rspack_core::{
  ContextMode, ContextModulePattern, ContextNameSpaceObject, ContextOptions, DependencyCategory,
  ReferencedSpecifier, escape_glob_pattern, extract_glob_base_dir, get_context,
  normalize_path_separators, normalize_path_separators_for_path, unescape_glob_path,
};
use rspack_error::{Error, Severity};
use rspack_macros::AstObject;
use rspack_paths::{Utf8Path, Utf8PathBuf};
use rspack_regex::RspackRegex;
use rspack_util::{SpanExt, identifier::relative_path_to_request, node_path::NodePath};
use sugar_path::SugarPath;
use swc_atoms::Atom;
use swc_experimental_ecma_ast::{CallExpr, Expr, GetSpan, ObjectLit};

use super::JavascriptParserPlugin;
use crate::{
  dependency::ImportMetaContextDependency,
  utils::{
    eval::{self, BasicEvaluatedExpression},
    object_properties::{FromAstExpr, get_from_object},
  },
  visitors::{
    JavascriptParser, clean_regexp_in_context_module, create_traceable_error,
    default_context_reg_exp, expr_name, static_string_from_expr,
  },
};

/// Options of `import.meta.webpackContext(request, options)`, mirroring the
/// TypeScript declaration in `packages/rspack/module.d.ts`.
#[derive(Debug, Default, AstObject)]
#[ast_object(rename_all = "camelCase")]
struct ImportMetaWebpackContextOptions {
  reg_exp: Option<RspackRegex>,
  include: Option<RspackRegex>,
  exclude: Option<RspackRegex>,
  mode: Option<String>,
  /// Absent or unrecognized means `true`.
  recursive: Option<bool>,
}

/// Options of `import.meta.glob(pattern, options)`, mirroring
/// `Rspack.ImportMetaGlobOptions` in `packages/rspack/module.d.ts`.
#[derive(Debug, Default, AstObject)]
#[ast_object(rename_all = "camelCase")]
struct ImportMetaGlobOptions {
  eager: bool,
  import: Option<String>,
  query: Option<ImportMetaGlobQuery>,
  base: Option<String>,
  exhaustive: bool,
}

impl From<&ImportMetaWebpackContextOptions> for ContextOptions {
  fn from(options: &ImportMetaWebpackContextOptions) -> Self {
    Self {
      pattern: options.reg_exp.clone().into(),
      include: options.include.clone(),
      exclude: options.exclude.clone(),
      mode: options
        .mode
        .as_deref()
        .map_or(ContextMode::Sync, ContextMode::from),
      recursive: options.recursive.unwrap_or(true),
      ..Default::default()
    }
  }
}

impl From<&ImportMetaGlobOptions> for ContextOptions {
  fn from(options: &ImportMetaGlobOptions) -> Self {
    Self {
      mode: if options.eager {
        ContextMode::Sync
      } else {
        ContextMode::Lazy
      },
      glob_import: options.import.clone(),
      glob_exhaustive: options.exhaustive,
      ..Default::default()
    }
  }
}

#[derive(Debug)]
enum ImportMetaGlobQuery {
  String(String),
  Record(Vec<(String, ImportMetaGlobQueryValue)>),
}

impl<'a> FromAstExpr<'a> for ImportMetaGlobQuery {
  fn from_ast_expr(expr: &Expr<'a>) -> Option<Self> {
    if let Some(query) = String::from_ast_expr(expr) {
      return Some(Self::String(query));
    }
    Vec::<(String, ImportMetaGlobQueryValue)>::from_ast_expr(expr).map(Self::Record)
  }
}

#[derive(Debug)]
enum ImportMetaGlobQueryValue {
  String(String),
  Number(f64),
  Bool(bool),
}

impl FromAstExpr<'_> for ImportMetaGlobQueryValue {
  fn from_ast_expr(expr: &Expr<'_>) -> Option<Self> {
    if let Some(value) = String::from_ast_expr(expr) {
      return Some(Self::String(value));
    }
    if let Some(value) = f64::from_ast_expr(expr) {
      return Some(Self::Number(value));
    }
    bool::from_ast_expr(expr).map(Self::Bool)
  }
}

impl ImportMetaGlobQuery {
  fn to_query_string(&self) -> String {
    match self {
      Self::String(query) => normalize_import_meta_glob_query(query.clone()),
      Self::Record(entries) => {
        let mut serializer = form_urlencoded::Serializer::new(String::new());
        for (key, value) in entries {
          let value = match value {
            ImportMetaGlobQueryValue::String(value) => value.clone(),
            ImportMetaGlobQueryValue::Number(value) => value.to_string(),
            ImportMetaGlobQueryValue::Bool(value) => value.to_string(),
          };
          serializer.append_pair(key, &value);
        }
        normalize_import_meta_glob_query(serializer.finish())
      }
    }
  }
}

fn parse_import_meta_glob_case_sensitive(
  glob_options: Option<&ObjectLit>,
  parser: &mut JavascriptParser,
) -> bool {
  let Some(value) = glob_options.and_then(|object| get_from_object(object, &["caseSensitive"]))
  else {
    return true;
  };

  let evaluated = parser.evaluate_expression(value);
  if evaluated.is_bool() {
    return evaluated.bool();
  }

  let mut error: Error = create_traceable_error(
    "Invalid import.meta.glob option".into(),
    "import.meta.glob() 'caseSensitive' option must be a constant boolean (true or false), defaulting to true".into(),
    parser.source.to_string(),
    value.span().into(),
  );
  error.severity = Severity::Warning;
  error.hide_stack = Some(true);
  parser.add_warning(error.into());
  true
}

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
  let base = normalize_path_separators_for_path(base);
  normalize_path_separators_for_path(
    Utf8Path::new(&base)
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

fn common_glob_base_dir<'a>(
  mut base_dirs: impl Iterator<Item = &'a str>,
  fallback: &str,
) -> String {
  let Some(first) = base_dirs.next() else {
    return fallback.to_string();
  };

  let mut common_base = Utf8PathBuf::from(first);
  for base in base_dirs {
    let base = Utf8Path::new(base);
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

fn case_insensitive_glob_pattern_root(
  pattern: &str,
  context: &str,
  compiler_context: &str,
) -> String {
  let pattern = pattern.strip_prefix('!').unwrap_or(pattern);
  let pattern = normalize_path_separators(pattern);
  let (base, pattern_to_join) =
    import_meta_glob_path_parts(context, compiler_context, pattern.as_str());
  let normalized_pattern_to_join = Utf8Path::new(pattern_to_join)
    .node_normalize_posix()
    .to_string();
  let stable_prefix = normalized_pattern_to_join
    .split('/')
    .take_while(|segment| segment.is_empty() || *segment == "." || *segment == "..")
    .collect::<Vec<_>>()
    .join("/");
  join_import_meta_glob_fs_path(base, &stable_prefix)
}

fn case_insensitive_glob_base_dir(
  patterns: &[String],
  context: &str,
  compiler_context: &str,
) -> String {
  let roots = patterns
    .iter()
    .filter(|pattern| !pattern.starts_with('!'))
    .map(|pattern| case_insensitive_glob_pattern_root(pattern, context, compiler_context))
    .collect::<Vec<_>>();
  common_glob_base_dir(roots.iter().map(String::as_str), context)
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
  let request = static_string_from_expr(&dyn_imported.expr)?;
  let raw_options = node.args.get(1).and_then(|arg| arg.expr.as_object());
  let options = raw_options
    .map(ImportMetaWebpackContextOptions::from_ast_object)
    .unwrap_or_default();
  let regexp_span = options.reg_exp.as_ref().and_then(|_| {
    raw_options
      .and_then(|options| get_from_object(options, &["regExp"]))
      .map(|regexp| regexp.span().into())
  });
  let regexp = options
    .reg_exp
    .clone()
    .unwrap_or_else(default_context_reg_exp);
  let span = node.span;
  let mut context_options = ContextOptions::new(
    &options,
    DependencyCategory::Esm,
    request,
    parser.compiler_options.context.to_string(),
    get_context(parser.resource_data).to_string(),
    span.real_lo(),
    span.real_hi(),
  );
  context_options.pattern = clean_regexp_in_context_module(regexp, regexp_span, parser).into();
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
  let options = glob_options
    .map(ImportMetaGlobOptions::from_ast_object)
    .unwrap_or_default();
  let glob_query = options
    .query
    .as_ref()
    .map_or_else(String::new, ImportMetaGlobQuery::to_query_string);
  let base = options.base.as_deref();
  let glob_case_sensitive = parse_import_meta_glob_case_sensitive(glob_options, parser);
  let resolve_context = resolve_import_meta_glob_context(
    importer_context.as_str(),
    parser.compiler_options.context.as_str(),
    base,
  );
  let glob_patterns = normalize_import_meta_glob_patterns(
    raw_glob_patterns,
    resolve_context.as_str(),
    parser.compiler_options.context.as_str(),
    base.is_some(),
  );
  let resolved_glob_patterns = glob_patterns
    .iter()
    .map(|pattern| {
      resolve_glob_pattern(
        pattern,
        resolve_context.as_str(),
        parser.compiler_options.context.as_str(),
      )
    })
    .collect::<Vec<_>>();
  let base_dir = if glob_case_sensitive {
    common_glob_base_dir(
      resolved_glob_patterns
        .iter()
        .filter(|pattern| !pattern.negative)
        .map(|pattern| pattern.absolute_base.as_str()),
      resolve_context.as_str(),
    )
  } else {
    case_insensitive_glob_base_dir(
      &glob_patterns,
      resolve_context.as_str(),
      parser.compiler_options.context.as_str(),
    )
  };
  let recursive = glob_patterns_are_recursive(&resolved_glob_patterns, &base_dir);

  let referenced_specifiers = options
    .import
    .as_deref()
    .filter(|import| *import != "*")
    .map(|import| vec![ReferencedSpecifier::new(vec![Atom::from(import)])]);
  let namespace_object = if parser.build_meta.strict_esm_module() {
    ContextNameSpaceObject::Strict
  } else {
    ContextNameSpaceObject::Bool(true)
  };

  let span = node.span;
  let context_options = ContextOptions {
    pattern: ContextModulePattern::Glob(glob_patterns),
    recursive,
    namespace_object,
    referenced_specifiers,
    glob_case_sensitive,
    ..ContextOptions::new(
      &options,
      DependencyCategory::Esm,
      concat_string!(base_dir, glob_query),
      parser.compiler_options.context.to_string(),
      resolve_context,
      span.real_lo(),
      span.real_hi(),
    )
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn converts_webpack_context_options() {
    let options = ImportMetaWebpackContextOptions {
      reg_exp: Some(RspackRegex::with_flags("^\\./", "i").expect("valid regexp")),
      include: Some(RspackRegex::new("include").expect("valid regexp")),
      exclude: Some(RspackRegex::new("exclude").expect("valid regexp")),
      mode: Some("lazy".to_string()),
      recursive: Some(false),
    };
    let context_options = ContextOptions::new(
      &options,
      DependencyCategory::Esm,
      "./request".to_string(),
      "/project".to_string(),
      "/project/src".to_string(),
      1,
      2,
    );

    assert_eq!(context_options.mode, ContextMode::Lazy);
    assert!(!context_options.recursive);
    assert_eq!(context_options.pattern.reg_exp(), options.reg_exp.as_ref());
    assert_eq!(context_options.include, options.include);
    assert_eq!(context_options.exclude, options.exclude);
    assert_eq!(context_options.category, DependencyCategory::Esm);
    assert_eq!(context_options.request, "./request");
    assert_eq!(context_options.context, "/project");
    assert_eq!(context_options.resolve_context, "/project/src");
    assert_eq!((context_options.start, context_options.end), (1, 2));
  }

  #[test]
  fn converts_glob_options() {
    let options = ImportMetaGlobOptions {
      eager: false,
      import: Some("default".to_string()),
      exhaustive: true,
      ..Default::default()
    };
    let context_options = ContextOptions::from(&options);

    assert_eq!(context_options.mode, ContextMode::Lazy);
    assert_eq!(context_options.glob_import.as_deref(), Some("default"));
    assert!(context_options.glob_exhaustive);
  }
  #[test]
  fn import_meta_glob_fs_path_normalizes_windows_base_before_parent_join() {
    assert_eq!(
      join_import_meta_glob_fs_path(r"D:\repo\context\nested", ".."),
      "D:/repo/context"
    );
  }
}
