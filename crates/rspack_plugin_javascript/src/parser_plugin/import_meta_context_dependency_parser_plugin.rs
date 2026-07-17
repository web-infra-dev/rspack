use concat_string::concat_string;
use rspack_core::{
  ContextMode, ContextModulePattern, ContextNameSpaceObject, ContextOptions, DependencyCategory,
  ReferencedSpecifier, compile_context_module_glob_request, get_context, normalize_path_separators,
  normalize_path_separators_for_path,
};
use rspack_macros::AstObject;
use rspack_paths::Utf8Path;
use rspack_regex::RspackRegex;
use rspack_util::{SpanExt, identifier::relative_path_to_request, node_path::NodePath};
use sugar_path::SugarPath;
use swc_atoms::Atom;
use swc_experimental_ecma_ast::{CallExpr, Expr};

use super::JavascriptParserPlugin;
use crate::{
  dependency::ImportMetaContextDependency,
  utils::{
    ast_object::{AstRegex, FromAstExpr},
    eval::{self, BasicEvaluatedExpression},
  },
  visitors::{
    JavascriptParser, clean_regexp_in_context_module, default_context_reg_exp, expr_name,
    static_string_from_expr,
  },
};

/// Options of `import.meta.webpackContext(request, options)`, mirroring the
/// TypeScript declaration in `packages/rspack/module.d.ts`.
#[derive(Debug, Default, AstObject)]
#[ast_object(rename_all = "camelCase")]
struct ImportMetaWebpackContextOptions {
  reg_exp: Option<AstRegex>,
  include: Option<AstRegex>,
  exclude: Option<AstRegex>,
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
  fn into_query_string(self) -> String {
    match self {
      Self::String(query) => normalize_import_meta_glob_query(query),
      Self::Record(entries) => {
        let mut serializer = form_urlencoded::Serializer::new(String::new());
        for (key, value) in entries {
          let value = match value {
            ImportMetaGlobQueryValue::String(value) => value,
            ImportMetaGlobQueryValue::Number(value) => value.to_string(),
            ImportMetaGlobQueryValue::Bool(value) => value.to_string(),
          };
          serializer.append_pair(&key, &value);
        }
        normalize_import_meta_glob_query(serializer.finish())
      }
    }
  }
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
  let options = node
    .args
    .get(1)
    .and_then(|arg| arg.expr.as_object())
    .map(ImportMetaWebpackContextOptions::from_ast_object)
    .unwrap_or_default();
  let regexp_span = options.reg_exp.as_ref().map(|regex| regex.span.into());
  let regexp = options
    .reg_exp
    .map_or_else(default_context_reg_exp, |regex| {
      RspackRegex::with_flags(regex.exp.as_str(), regex.flags.as_str()).expect("reg failed")
    });
  let include = options.include.map(|regex| {
    RspackRegex::with_flags(regex.exp.as_str(), regex.flags.as_str()).expect("reg failed")
  });
  let exclude = options.exclude.map(|regex| {
    RspackRegex::with_flags(regex.exp.as_str(), regex.flags.as_str()).expect("reg failed")
  });
  let mode = options
    .mode
    .map_or(ContextMode::Sync, |mode| mode.as_str().into());
  let recursive = options.recursive.is_none_or(|recursive| recursive);
  let span = node.span;
  let context_options = ContextOptions {
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
  let options = node
    .args
    .get(1)
    .and_then(|arg| arg.expr.as_object())
    .map(ImportMetaGlobOptions::from_ast_object)
    .unwrap_or_default();
  let mode = if options.eager {
    ContextMode::Sync
  } else {
    ContextMode::Lazy
  };
  let glob_import = options.import;
  let glob_query = options
    .query
    .map_or_else(String::new, ImportMetaGlobQuery::into_query_string);
  let base = options.base;
  let glob_exhaustive = options.exhaustive;
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
