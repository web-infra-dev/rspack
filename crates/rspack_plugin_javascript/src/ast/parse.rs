use std::sync::Arc;

use rspack_ast::javascript::Ast;
use rspack_core::ModuleType;
use rspack_error::TraceableError;
use swc_core::common::comments::Comments;
use swc_core::common::input::SourceFileInput;
use swc_core::common::{SourceFile, SourceMap};
use swc_core::ecma::ast::{EsVersion, Program};
use swc_core::ecma::parser::lexer::Lexer;
use swc_core::ecma::parser::{self, Parser, Syntax};
use swc_node_comments::SwcComments;

use crate::utils::{ecma_parse_error_deduped_to_rspack_error, DedupEcmaErrors};
use crate::IsModule;

fn module_type_to_is_module(value: &ModuleType) -> IsModule {
  // parser options align with webpack
  match value {
    ModuleType::JsEsm => IsModule::Bool(true),
    ModuleType::JsDynamic => IsModule::Bool(false),
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
  let lexer = Lexer::new(syntax, target, SourceFileInput::from(&*fm), comments);
  parse_with_lexer(lexer, is_module)
}

fn parse_with_lexer(
  lexer: Lexer,
  is_module: IsModule,
) -> Result<Program, Vec<parser::error::Error>> {
  let mut parser = Parser::new_from(lexer);
  let program_result = match is_module {
    IsModule::Bool(true) => parser.parse_module().map(Program::Module),
    IsModule::Bool(false) => parser.parse_script().map(Program::Script),
    IsModule::Unknown => parser.parse_program(),
  };
  let mut errors = parser.take_errors();
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
  lexer: Lexer,
  fm: &SourceFile,
  cm: Arc<SourceMap>,
  comments: Option<SwcComments>,
  module_type: &ModuleType,
) -> Result<Ast, Vec<TraceableError>> {
  match parse_with_lexer(lexer, module_type_to_is_module(module_type)) {
    Ok(program) => Ok(Ast::new(program, cm, comments)),
    Err(errs) => Err(
      errs
        .dedup_ecma_errors()
        .into_iter()
        .map(|err| ecma_parse_error_deduped_to_rspack_error(err, fm, module_type))
        .collect::<Vec<_>>(),
    ),
  }
}
