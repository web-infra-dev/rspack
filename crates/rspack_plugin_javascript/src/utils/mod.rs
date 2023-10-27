mod get_prop_from_obj;

use std::path::Path;

pub use get_prop_from_obj::*;
use rspack_core::{ErrorSpan, ModuleType};
use rspack_error::{DiagnosticKind, Error};
use swc_core::common::{SourceFile, Span, Spanned, SyntaxContext, DUMMY_SP};
use swc_core::ecma::ast::{CallExpr, Callee, Expr, ExprOrSpread, Ident, Lit, Str};
use swc_core::ecma::atoms::JsWord;
use swc_core::ecma::parser::Syntax;
use swc_core::ecma::parser::{EsConfig, TsConfig};

fn syntax_by_ext(
  filename: &Path,
  enable_decorators: bool,
  should_transform_by_default: bool,
) -> Syntax {
  // swc_core::base::Compiler::process_js_with_custom_pass()
  let ext = filename
    .extension()
    .and_then(|ext| ext.to_str())
    .unwrap_or("js");
  match ext == "ts" || ext == "tsx" {
    true => {
      let filename = filename.to_string_lossy();
      Syntax::Typescript(TsConfig {
        decorators: enable_decorators,
        tsx: ext == "tsx",
        dts: filename.ends_with(".d.ts") || filename.ends_with(".d.tsx"),
        ..Default::default()
      })
    }
    false => Syntax::Es(EsConfig {
      jsx: ext == "jsx",
      export_default_from: true,
      decorators_before_export: true,
      decorators: should_transform_by_default && enable_decorators,
      fn_bind: true,
      allow_super_outside_method: true,
      ..Default::default()
    }),
  }
}

pub fn syntax_by_module_type(
  filename: &Path,
  module_type: &ModuleType,
  enable_decorators: bool,
  should_transform_by_default: bool,
) -> Syntax {
  let js_syntax = Syntax::Es(EsConfig {
    jsx: should_transform_by_default && matches!(module_type, ModuleType::Jsx),
    export_default_from: true,
    decorators_before_export: true,
    // If `disableTransformByDefault` is on, then we treat everything passed in as a web standard stuff,
    // which means everything that is not a web standard would results in a parsing error.
    // So as the decorator.
    decorators: should_transform_by_default && enable_decorators,
    fn_bind: true,
    allow_super_outside_method: true,
    ..Default::default()
  });

  // Legacy behavior: `ts`, `tsx`, etc.
  if should_transform_by_default {
    return match module_type {
      ModuleType::Js | ModuleType::Jsx => js_syntax,
      ModuleType::Ts | ModuleType::Tsx => {
        let filename = filename.to_string_lossy();
        Syntax::Typescript(TsConfig {
          // `disableTransformByDefault` will not affect TypeScript-like modules,
          // as we are following standard of TypeScript compiler.
          // This is not a web standard by all means.
          decorators: enable_decorators,
          tsx: matches!(module_type, ModuleType::Tsx),
          dts: filename.ends_with(".d.ts") || filename.ends_with(".d.tsx"),
          ..Default::default()
        })
      }
      _ => syntax_by_ext(filename, enable_decorators, should_transform_by_default),
    };
  }

  js_syntax
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
              sym,
              span: Span { ctxt, .. },
              ..
            }) if sym == "require" && ctxt == unresolved_ctxt
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

pub fn ecma_parse_error_to_rspack_error(
  error: swc_core::ecma::parser::error::Error,
  fm: &SourceFile,
  module_type: &ModuleType,
) -> Error {
  let (file_type, diagnostic_kind) = match module_type {
    ModuleType::Js | ModuleType::JsDynamic | ModuleType::JsEsm => {
      ("JavaScript", DiagnosticKind::JavaScript)
    }
    ModuleType::Jsx | ModuleType::JsxDynamic | ModuleType::JsxEsm => ("JSX", DiagnosticKind::Jsx),
    ModuleType::Tsx => ("TSX", DiagnosticKind::Tsx),
    ModuleType::Ts => ("Typescript", DiagnosticKind::Typescript),
    _ => unreachable!(),
  };
  let message = error.kind().msg().to_string();
  let span: ErrorSpan = error.span().into();
  let traceable_error = rspack_error::TraceableError::from_source_file(
    fm,
    span.start as usize,
    span.end as usize,
    format!("{file_type} parsing error"),
    message,
  )
  .with_kind(diagnostic_kind);
  Error::TraceableError(traceable_error)
}

pub fn join_jsword(arr: &[JsWord], separator: &str) -> String {
  let mut ret = String::new();
  if let Some(item) = arr.first() {
    ret.push_str(item);
  }
  for item in arr.iter().skip(1) {
    ret.push_str(separator);
    ret.push_str(item);
  }
  ret
}

pub fn is_diff_mode() -> bool {
  let is_diff_mode = std::env::var("RSPACK_DIFF").ok().unwrap_or_default();
  is_diff_mode == "true"
}
