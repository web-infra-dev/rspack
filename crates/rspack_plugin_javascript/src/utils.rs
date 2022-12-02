use dashmap::DashMap;
use hashbrown::hash_map::DefaultHashBuilder;
use once_cell::sync::Lazy;
use pathdiff::diff_paths;
use rspack_core::rspack_sources::{
  BoxSource, CachedSource, ConcatSource, MapOptions, RawSource, Source, SourceExt,
};
use rspack_core::{runtime_globals, Compilation, ErrorSpan, ModuleType};
use rspack_error::{DiagnosticKind, Error};
use serde_json::json;
use std::path::Path;
use std::sync::Arc;
use swc::Compiler as SwcCompiler;
use swc_atoms::js_word;
use swc_common::{FilePathMapping, SourceMap, Span, Spanned, SyntaxContext, DUMMY_SP};
use swc_ecma_ast::{CallExpr, Callee, Expr, ExprOrSpread, Ident, Lit, Str};
use swc_ecma_parser::Syntax;
use swc_ecma_parser::{EsConfig, TsConfig};

static SWC_COMPILER: Lazy<Arc<SwcCompiler>> = Lazy::new(|| {
  let cm = Arc::new(SourceMap::new(FilePathMapping::empty()));

  Arc::new(SwcCompiler::new(cm))
});

pub fn get_swc_compiler() -> Arc<SwcCompiler> {
  SWC_COMPILER.clone()
}

fn syntax_by_ext(filename: &str, enable_decorators: bool) -> Syntax {
  let ext = Path::new(filename)
    .extension()
    .and_then(|ext| ext.to_str())
    .unwrap_or("js");
  match ext == "ts" || ext == "tsx" {
    true => Syntax::Typescript(TsConfig {
      decorators: enable_decorators,
      tsx: ext == "tsx",
      dts: filename.ends_with(".d.ts") || filename.ends_with(".d.tsx"),
      ..Default::default()
    }),
    false => Syntax::Es(EsConfig {
      import_assertions: true,
      jsx: ext == "jsx",
      export_default_from: true,
      decorators_before_export: true,
      decorators: enable_decorators,
      fn_bind: true,
      allow_super_outside_method: true,
      ..Default::default()
    }),
  }
}

pub fn syntax_by_module_type(
  filename: &str,
  module_type: &ModuleType,
  enable_decorators: bool,
) -> Syntax {
  match module_type {
    ModuleType::Js | ModuleType::Jsx => Syntax::Es(EsConfig {
      import_assertions: true,
      jsx: matches!(module_type, ModuleType::Jsx),
      export_default_from: true,
      decorators_before_export: true,
      decorators: enable_decorators,
      fn_bind: true,
      allow_super_outside_method: true,
      ..Default::default()
    }),
    ModuleType::Ts | ModuleType::Tsx => Syntax::Typescript(TsConfig {
      decorators: enable_decorators,
      tsx: matches!(module_type, ModuleType::Tsx),
      dts: filename.ends_with(".d.ts") || filename.ends_with(".d.tsx"),
      ..Default::default()
    }),
    _ => syntax_by_ext(filename, enable_decorators),
  }
}

pub fn set_require_literal_args(e: &mut CallExpr, arg_value: &str) {
  match e.args.first().expect("this should never happen") {
    ExprOrSpread { spread: None, expr } => match &**expr {
      Expr::Lit(Lit::Str(str)) => str.clone(),
      _ => panic!("should never be here"),
    },
    _ => panic!("should never be here"),
  };

  e.args = vec![ExprOrSpread {
    spread: None,
    expr: Box::new(Expr::Lit(Lit::Str(Str {
      span: DUMMY_SP,
      value: arg_value.into(),
      raw: None,
    }))),
  }];
}

pub fn get_callexpr_literal_args(e: &CallExpr) -> String {
  match e.args.first().expect("this should never happen") {
    ExprOrSpread { spread: None, expr } => match &**expr {
      Expr::Lit(Lit::Str(str)) => str.value.to_string(),
      _ => String::new(),
    },
    _ => String::new(),
  }
}

pub fn is_require_literal_expr(e: &CallExpr, unresolved_ctxt: &SyntaxContext) -> bool {
  if e.args.len() == 1 {
    let res = !get_callexpr_literal_args(e).is_empty();

    res
      && match &e.callee {
        Callee::Expr(callee) => {
          matches!(
            &**callee,
            Expr::Ident(Ident {
              sym: js_word!("require"),
              span: Span { ctxt, .. },
              ..
            }) if ctxt == unresolved_ctxt
          )
        }
        _ => false,
      }
  } else {
    false
  }
}

pub fn is_dynamic_import_literal_expr(e: &CallExpr) -> bool {
  if e.args.len() == 1 {
    let res = !get_callexpr_literal_args(e).is_empty();

    res && matches!(&e.callee, Callee::Import(_))
  } else {
    false
  }
}

pub fn wrap_module_function(source: BoxSource, module_id: &str) -> BoxSource {
  /***
   * generate wrapper module:
   * {module_id}: function(module, exports, __rspack_require__, __rspack_dynamic_require__) {
   * "use strict";
   * {source}
   * },
   */
  CachedSource::new(ConcatSource::new([
    RawSource::from("\"").boxed(),
    RawSource::from(module_id.to_string()).boxed(),
    RawSource::from("\": ").boxed(),
    RawSource::from(format!(
      "function (module, exports, {}) {{\n",
      runtime_globals::REQUIRE
    ))
    .boxed(),
    RawSource::from("\"use strict\";\n").boxed(),
    source,
    RawSource::from("},\n").boxed(),
  ]))
  .boxed()
}

pub fn ecma_parse_error_to_rspack_error(
  error: swc_ecma_parser::error::Error,
  path: &str,
  module_type: &ModuleType,
) -> Error {
  let (file_type, diagnostic_kind) = match module_type {
    ModuleType::Js => ("JavaScript", DiagnosticKind::JavaScript),
    ModuleType::Jsx => ("JSX", DiagnosticKind::Jsx),
    ModuleType::Tsx => ("TSX", DiagnosticKind::Tsx),
    ModuleType::Ts => ("Typescript", DiagnosticKind::Typescript),
    _ => unreachable!(),
  };
  let message = error.kind().msg().to_string();
  let span: ErrorSpan = error.span().into();
  let traceable_error = rspack_error::TraceableError::from_path(
    path.to_string(),
    span.start as usize,
    span.end as usize,
    format!("{} parsing error", file_type),
    message,
  )
  .with_kind(diagnostic_kind);
  rspack_error::Error::TraceableError(traceable_error)
  //Use this `Error` convertion could avoid eagerly clone source file.
}

pub fn wrap_eval_source_map(
  module_source: BoxSource,
  cache: &DashMap<BoxSource, BoxSource, DefaultHashBuilder>,
  compilation: &Compilation,
) -> rspack_error::Result<BoxSource> {
  if let Some(cached) = cache.get(&module_source) {
    return Ok(cached.clone());
  }
  if let Some(mut map) = module_source.map(&MapOptions::new(compilation.options.devtool.cheap())) {
    for source in map.sources_mut() {
      let uri = if source.starts_with('<') && source.ends_with('>') {
        &source[1..source.len() - 1] // remove '<' and '>' for swc FileName::Custom
      } else {
        &source[..]
      };
      *source = if let Some(relative_path) = diff_paths(uri, &*compilation.options.context) {
        relative_path.to_string_lossy().to_string()
      } else {
        uri.to_owned()
      };
    }
    if compilation.options.devtool.no_sources() {
      for content in map.sources_content_mut() {
        *content = String::default();
      }
    }
    let mut map_buffer = Vec::new();
    map
      .to_writer(&mut map_buffer)
      .map_err(|e| rspack_error::Error::InternalError(e.to_string()))?;
    let base64 = base64::encode(&map_buffer);
    let footer =
      format!("\n//# sourceMappingURL=data:application/json;charset=utf-8;base64,{base64}");
    let content = module_source.source().to_string();
    let result = RawSource::from(format!("eval({});", json!(content + &footer))).boxed();
    cache.insert(module_source, result.clone());
    Ok(result)
  } else {
    Ok(module_source)
  }
}
