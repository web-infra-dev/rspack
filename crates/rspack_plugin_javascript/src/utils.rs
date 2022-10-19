use crate::{RSPACK_DYNAMIC_IMPORT, RSPACK_REQUIRE};
use dashmap::DashMap;
use hashbrown::hash_map::DefaultHashBuilder;
use once_cell::sync::Lazy;
use pathdiff::diff_paths;
use rspack_core::rspack_sources::{
  BoxSource, ConcatSource, MapOptions, RawSource, Source, SourceExt,
};
use rspack_core::{Compilation, ErrorSpan, ModuleType, TargetPlatform};
use rspack_error::{DiagnosticKind, Error};
use serde_json::json;
use std::path::Path;
use std::sync::Arc;
use swc::Compiler as SwcCompiler;
use swc_atoms::js_word;
use swc_common::{FilePathMapping, Mark, SourceMap, Span, Spanned, DUMMY_SP};
use swc_ecma_ast::{CallExpr, Callee, Expr, ExprOrSpread, Id, Ident, Lit, Str};
use swc_ecma_parser::Syntax;
use swc_ecma_parser::{EsConfig, TsConfig};

static SWC_COMPILER: Lazy<Arc<SwcCompiler>> = Lazy::new(|| {
  let cm = Arc::new(SourceMap::new(FilePathMapping::empty()));

  Arc::new(SwcCompiler::new(cm))
});

pub fn get_swc_compiler() -> Arc<SwcCompiler> {
  SWC_COMPILER.clone()
}

pub fn syntax_by_ext(ext: &str) -> Syntax {
  match ext == "ts" || ext == "tsx" {
    true => Syntax::Typescript(TsConfig {
      decorators: false,
      tsx: ext == "tsx",
      ..Default::default()
    }),
    false => Syntax::Es(EsConfig {
      private_in_object: true,
      import_assertions: true,
      jsx: ext == "jsx",
      export_default_from: true,
      decorators_before_export: true,
      decorators: true,
      fn_bind: true,
      allow_super_outside_method: true,
      ..Default::default()
    }),
  }
}

pub fn syntax_by_module_type(filename: &str, module_type: &ModuleType) -> Syntax {
  match module_type {
    ModuleType::Js | ModuleType::Jsx => Syntax::Es(EsConfig {
      private_in_object: true,
      import_assertions: true,
      jsx: matches!(module_type, ModuleType::Jsx),
      export_default_from: true,
      decorators_before_export: true,
      decorators: true,
      fn_bind: true,
      allow_super_outside_method: true,
      ..Default::default()
    }),
    ModuleType::Ts | ModuleType::Tsx => Syntax::Typescript(TsConfig {
      decorators: false,
      tsx: matches!(module_type, ModuleType::Tsx),
      ..Default::default()
    }),
    _ => {
      let ext = Path::new(filename)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("js");
      syntax_by_ext(ext)
    }
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

pub fn is_require_literal_expr(e: &CallExpr, _unresolved_mark: Mark, _require_id: &Id) -> bool {
  if e.args.len() == 1 {
    let res = !get_callexpr_literal_args(e).is_empty();

    res
      && match &e.callee {
        Callee::Expr(callee) => {
          matches!(
            &**callee,
            Expr::Ident(Ident {
              sym: js_word!("require"),
              span: Span { .. },
              ..
            })
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
  ConcatSource::new([
    RawSource::from("\"").boxed(),
    RawSource::from(module_id.to_string()).boxed(),
    RawSource::from("\": ").boxed(),
    RawSource::from(format!(
      "function (module, exports, {}, {}) {{\n",
      RSPACK_REQUIRE, RSPACK_DYNAMIC_IMPORT
    ))
    .boxed(),
    RawSource::from("\"use strict\";\n").boxed(),
    source,
    RawSource::from("},\n").boxed(),
  ])
  .boxed()
}

pub fn get_wrap_chunk_before(
  namespace: &str,
  register: &str,
  chunk_id: &str,
  platform: &TargetPlatform,
) -> BoxSource {
  match platform {
    TargetPlatform::Node(_) => RawSource::from(format!(
      r#"exports.ids = ["{}"];
      exports.modules = {{"#,
      chunk_id
    ))
    .boxed(),
    _ => RawSource::from(format!(
      "self[\"{}\"].{}([\"{}\"], {{\n",
      namespace, register, chunk_id
    ))
    .boxed(),
  }
}

pub fn get_wrap_chunk_after(platform: &TargetPlatform) -> BoxSource {
  match platform {
    TargetPlatform::Node(_) => RawSource::from("};").boxed(),
    _ => RawSource::from("});").boxed(),
  }
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
      *source = if let Some(relative_path) = diff_paths(uri, &compilation.options.context) {
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
