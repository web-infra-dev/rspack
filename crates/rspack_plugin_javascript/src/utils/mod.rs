mod r#const;
pub mod eval;
mod get_prop_from_obj;
pub mod mangle_exports;

use std::path::Path;

use rspack_core::{ErrorSpan, ModuleType};
use rspack_error::{DiagnosticKind, TraceableError};
use rustc_hash::FxHashSet as HashSet;
use swc_core::common::{SourceFile, Span, Spanned};
use swc_core::ecma::parser::Syntax;
use swc_core::ecma::parser::{EsConfig, TsConfig};

pub use self::get_prop_from_obj::*;
pub use self::r#const::*;

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
    export_default_from: false,
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

#[derive(PartialEq, Eq, Hash)]
pub struct EcmaError(String, Span);
pub struct EcmaErrorsDeduped(Vec<EcmaError>);

impl IntoIterator for EcmaErrorsDeduped {
  type Item = EcmaError;
  type IntoIter = std::vec::IntoIter<Self::Item>;
  fn into_iter(self) -> Self::IntoIter {
    self.0.into_iter()
  }
}

impl From<Vec<swc_core::ecma::parser::error::Error>> for EcmaErrorsDeduped {
  fn from(value: Vec<swc_core::ecma::parser::error::Error>) -> Self {
    Self(
      value
        .into_iter()
        .map(|v| EcmaError(v.kind().msg().to_string(), v.span()))
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>(),
    )
  }
}

/// Dedup ecma errors against [swc_core::ecma::parser::error::Error]
/// Returns a wrapper of an iterator that contains deduped errors.
impl DedupEcmaErrors for Vec<swc_core::ecma::parser::error::Error> {
  fn dedup_ecma_errors(self) -> EcmaErrorsDeduped {
    EcmaErrorsDeduped::from(self)
  }
}

pub trait DedupEcmaErrors {
  fn dedup_ecma_errors(self) -> EcmaErrorsDeduped;
}

pub fn ecma_parse_error_deduped_to_rspack_error(
  EcmaError(message, span): EcmaError,
  fm: &SourceFile,
  module_type: &ModuleType,
) -> TraceableError {
  let (file_type, diagnostic_kind) = match module_type {
    ModuleType::Js | ModuleType::JsDynamic | ModuleType::JsEsm => {
      ("JavaScript", DiagnosticKind::JavaScript)
    }
    ModuleType::Jsx | ModuleType::JsxDynamic | ModuleType::JsxEsm => ("JSX", DiagnosticKind::Jsx),
    ModuleType::Tsx => ("TSX", DiagnosticKind::Tsx),
    ModuleType::Ts => ("Typescript", DiagnosticKind::Typescript),
    _ => unreachable!(),
  };
  let span: ErrorSpan = span.into();
  rspack_error::TraceableError::from_source_file(
    fm,
    span.start as usize,
    span.end as usize,
    format!("{file_type} parsing error"),
    message,
  )
  .with_kind(diagnostic_kind)
}

pub fn is_diff_mode() -> bool {
  let is_diff_mode = std::env::var("RSPACK_DIFF").ok().unwrap_or_default();
  is_diff_mode == "true"
}
