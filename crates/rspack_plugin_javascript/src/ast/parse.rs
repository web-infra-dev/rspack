use std::sync::Arc;

use rspack_ast::javascript::Ast;
use rspack_core::ModuleType;
use rspack_error::Error;
use swc_core::common::comments::Comments;
use swc_core::common::{FileName, SourceFile};
use swc_core::ecma::ast::{self, EsVersion, Program};
use swc_core::ecma::parser::{
  self, parse_file_as_module, parse_file_as_program, parse_file_as_script, Syntax,
};
use swc_node_comments::SwcComments;

use crate::utils::ecma_parse_error_to_rspack_error;
use crate::IsModule;

fn module_type_to_is_module(value: &ModuleType) -> IsModule {
  // parser options align with webpack
  match value {
    ModuleType::JsEsm | ModuleType::JsxEsm => IsModule::Bool(true),
    ModuleType::JsDynamic | ModuleType::JsxDynamic => IsModule::Bool(false),
    _ => IsModule::Unknown,
  }
}

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
) -> Result<Program, Vec<parser::error::Error>> {
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
  let source_code = if syntax.dts() {
    // dts build result must be empty
    "".to_string()
  } else {
    source_code
  };

  let cm: Arc<swc_core::common::SourceMap> = Default::default();
  let fm = cm.new_source_file(FileName::Custom(filename.to_string()), source_code);
  let comments = SwcComments::default();

  match parse_js(
    fm.clone(),
    ast::EsVersion::EsNext,
    syntax,
    module_type_to_is_module(module_type),
    Some(&comments),
  ) {
    Ok(program) => Ok(Ast::new(program, cm, Some(comments))),
    Err(errs) => Err(Error::BatchErrors(
      errs
        .into_iter()
        .map(|err| ecma_parse_error_to_rspack_error(err, &fm, module_type))
        .collect::<Vec<_>>(),
    )),
  }
}
