use crate::utils::{ecma_parse_error_to_rspack_error, get_swc_compiler, syntax_by_module_type};
use rspack_core::{ast::javascript::Ast, ModuleType, PATH_START_BYTE_POS_MAP};
use rspack_error::Error;
use std::sync::Arc;
use swc::config::IsModule;
use swc_common::comments::Comments;
use swc_common::{FileName, SourceFile};
use swc_ecma_ast::{EsVersion, Program};
use swc_ecma_parser::{parse_file_as_module, parse_file_as_program, parse_file_as_script, Syntax};

/// Why this helper function design like this?
/// 1. `swc_ecma_parser` could return ast with some errors which are recoverable
/// or warning (though swc defined them as errors), but the parser at here should
/// be non-error-tolerant.
///
/// 2. We can't convert to [rspack_error::Error] at this point, because there is
/// no `path` and `source`
pub fn parse_js(
  fm: Arc<SourceFile>,
  target: EsVersion,
  syntax: Syntax,
  is_module: IsModule,
  comments: Option<&dyn Comments>,
) -> Result<Program, Vec<swc_ecma_parser::error::Error>> {
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
    Ok(program) => {
      if !errors.is_empty() {
        return Err(errors);
      }
      Ok(program)
    }
    Err(err) => {
      errors.push(err);
      Err(errors)
    }
  }
}

pub fn parse(
  source_code: String,
  syntax: Syntax,
  filename: &str,
  module_type: &ModuleType,
) -> Result<Ast, Error> {
  let compiler = get_swc_compiler();
  let fm = compiler
    .cm
    .new_source_file(FileName::Custom(filename.to_string()), source_code);
  PATH_START_BYTE_POS_MAP.insert(filename.to_string(), fm.start_pos.0);

  match parse_js(
    fm,
    swc_ecma_ast::EsVersion::Es2022,
    syntax,
    // TODO: Is this correct to think the code is module by default?
    IsModule::Bool(true),
    None,
  ) {
    Ok(program) => Ok(Ast::new(program)),
    Err(errs) => Err(Error::BatchErrors(
      errs
        .into_iter()
        .map(|err| ecma_parse_error_to_rspack_error(err, filename, module_type))
        .collect::<Vec<_>>(),
    )),
  }
}

pub fn parse_js_code(js_code: String, module_type: &ModuleType) -> Result<Program, Error> {
  let syntax = syntax_by_module_type("", module_type, false);
  let compiler = get_swc_compiler();
  let fm = compiler
    .cm
    .new_source_file(FileName::Custom("".to_string()), js_code);

  match parse_js(
    fm,
    swc_ecma_ast::EsVersion::Es2022,
    syntax,
    // TODO: Is this correct to think the code is module by default?
    IsModule::Bool(true),
    None,
  ) {
    Ok(program) => Ok(program),
    Err(errs) => Err(Error::BatchErrors(
      errs
        .into_iter()
        .map(|err| ecma_parse_error_to_rspack_error(err, "", module_type))
        .collect::<Vec<_>>(),
    )),
  }
}
