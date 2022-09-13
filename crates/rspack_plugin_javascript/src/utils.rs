use once_cell::sync::Lazy;
use rspack_core::{ErrorSpan, ModuleType, PATH_START_BYTE_POS_MAP};
use rspack_error::{
  errors_to_diagnostics, DiagnosticKind, Error, IntoTWithDiagnosticArray, TWithDiagnosticArray,
};
use std::path::Path;
use std::sync::Arc;
use swc::{config::IsModule, Compiler as SwcCompiler};
use swc_atoms::js_word;
use swc_common::comments::Comments;
use swc_common::{FileName, FilePathMapping, Mark, SourceFile, SourceMap, Span, Spanned, DUMMY_SP};
use swc_ecma_ast::{CallExpr, Callee, EsVersion, Expr, ExprOrSpread, Id, Ident, Lit, Program, Str};
use swc_ecma_parser::{parse_file_as_module, parse_file_as_program, parse_file_as_script, Syntax};
use swc_ecma_parser::{EsConfig, TsConfig};

static SWC_COMPILER: Lazy<Arc<SwcCompiler>> = Lazy::new(|| {
  let cm = Arc::new(SourceMap::new(FilePathMapping::empty()));

  Arc::new(SwcCompiler::new(cm))
});

pub fn get_swc_compiler() -> Arc<SwcCompiler> {
  SWC_COMPILER.clone()
}

/// Why this helper function design like this?
/// 1. `swc_ecma_parser` could return ast with some errors which are recoverable or warning(though swc defined them as errors), we
/// need to return those error for better dx, some warning behavior may lead some unexpected result.
/// 2. We can't convert to [rspack_error::Error] at this point, because there is no `path` and `source`
pub fn parse_js(
  compiler: &SwcCompiler,
  fm: Arc<SourceFile>,
  target: EsVersion,
  syntax: Syntax,
  is_module: IsModule,
  comments: Option<&dyn Comments>,
) -> Result<(Program, Vec<swc_ecma_parser::error::Error>), Vec<swc_ecma_parser::error::Error>> {
  compiler.run(|| {
    let mut errors = vec![];
    let program_result = match is_module {
      IsModule::Bool(true) => {
        parse_file_as_module(&fm, syntax, target, comments, &mut errors).map(Program::Module)
      }
      IsModule::Bool(false) => {
        parse_file_as_script(&fm, syntax, target, comments, &mut errors).map(Program::Script)
      }
      IsModule::Unknown => parse_file_as_program(&fm, syntax, target, comments, &mut errors),
    };

    // Using combinator will let rustc unhappy.
    match program_result {
      Ok(program) => Ok((program, errors)),
      Err(err) => {
        errors.push(err);
        Err(errors)
      }
    }
  })
}
pub fn parse_file(
  source_code: String,
  filename: &str,
  module_type: &ModuleType,
) -> Result<TWithDiagnosticArray<Program>, Error> {
  let syntax = syntax_by_module_type(filename, module_type);
  let compiler = get_swc_compiler();
  let fm = compiler
    .cm
    .new_source_file(FileName::Custom(filename.to_string()), source_code);
  PATH_START_BYTE_POS_MAP.insert(filename.to_string(), fm.start_pos.0);

  match parse_js(
    &compiler,
    fm,
    swc_ecma_ast::EsVersion::Es2022,
    syntax,
    // TODO: Is this correct to think the code is module by default?
    IsModule::Bool(true),
    None,
  ) {
    Ok((program, errs)) => {
      let errors = errs
        .into_iter()
        .map(|err| ecma_parse_error_to_rspack_error(err, filename, module_type))
        .collect::<Vec<_>>();
      Ok(program.with_diagnostic(errors_to_diagnostics(errors)))
    }
    Err(errs) => Err(Error::BatchErrors(
      errs
        .into_iter()
        .map(|err| ecma_parse_error_to_rspack_error(err, filename, module_type))
        .collect::<Vec<_>>(),
    )),
  }
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

pub fn wrap_module_function(source: String, module_id: &str) -> String {
  format!(
    r#""{}":{},"#,
    module_id,
    source.trim_end().trim_end_matches(';')
  )
}

pub fn get_wrap_chunk_before(namespace: &str, register: &str, chunk_id: &str) -> String {
  format!(
    r#"self["{}"].{}([
    "{}"
  ], {{"#,
    namespace, register, chunk_id
  )
}

pub fn get_wrap_chunk_after() -> String {
  String::from("});")
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
