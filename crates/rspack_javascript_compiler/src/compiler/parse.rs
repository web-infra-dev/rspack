use std::sync::Arc;

use rspack_error::BatchErrors;
use swc_core::{
  base::config::IsModule,
  common::{FileName, SourceFile, comments::Comments, input::SourceFileInput},
  ecma::{
    ast::{EsVersion, Program as SwcProgram},
    parser::{self, Parser, Syntax, lexer::Lexer, unstable::TokenAndSpan},
  },
};
use swc_node_comments::SwcComments;

use super::JavaScriptCompiler;
use crate::{
  ast::{self, Ast},
  error::{DedupEcmaErrors, ecma_parse_error_deduped_to_rspack_error},
};

impl JavaScriptCompiler {
  /// Parses JavaScript source code into an AST.
  ///
  /// This method takes the filename, source code, target ECMAScript version, syntax, module type, and optional comments.
  /// It returns a `Result` containing the parsed AST if successful, or a `BatchErrors` if an error occurs.
  ///
  /// # Parameters
  ///
  /// - `filename`: The name of the file being parsed.
  /// - `source`: The source code to parse.
  /// - `target`: The target ECMAScript version.
  /// - `syntax`: The syntax to use for parsing.
  /// - `is_module`: Indicates if the source code is a module.
  /// - `comments`: Optional comments to include in the parsing process.
  ///
  /// # Returns
  ///
  /// A `Result` containing the parsed AST if successful, or a `BatchErrors` if an error occurs.
  pub fn parse<S: Into<String>>(
    self,
    filename: FileName,
    source: S,
    target: EsVersion,
    syntax: Syntax,
    is_module: IsModule,
    comments: Option<SwcComments>,
  ) -> Result<Ast, BatchErrors> {
    let fm = self.cm.new_source_file(Arc::new(filename), source.into());
    let lexer = Lexer::new(
      syntax,
      target,
      SourceFileInput::from(&*fm),
      comments.as_ref().map(|c| c as &dyn Comments),
    );

    parse_with_lexer(lexer, is_module, false)
      .map(|(program, _)| {
        Ast::new(program, self.cm.clone(), comments)
          .with_context(ast::Context::new(self.cm, Some(self.globals)))
      })
      .map_err(|errs| {
        BatchErrors(
          errs
            .dedup_ecma_errors()
            .into_iter()
            .map(|err| ecma_parse_error_deduped_to_rspack_error(err, fm.src.to_string()))
            .collect::<Vec<_>>(),
        )
      })
  }

  pub fn parse_with_lexer(
    self,
    source: &str,
    lexer: Lexer,
    is_module: IsModule,
    comments: Option<SwcComments>,
    with_tokens: bool,
  ) -> Result<(Ast, Option<Vec<TokenAndSpan>>), BatchErrors> {
    parse_with_lexer(lexer, is_module, with_tokens)
      .map(|(program, tokens)| {
        (
          Ast::new(program, self.cm.clone(), comments)
            .with_context(ast::Context::new(self.cm.clone(), Some(self.globals))),
          tokens,
        )
      })
      .map_err(|errs| {
        BatchErrors(
          errs
            .dedup_ecma_errors()
            .into_iter()
            .map(|err| ecma_parse_error_deduped_to_rspack_error(err, source.to_string()))
            .collect::<Vec<_>>(),
        )
      })
  }

  /// Parses JavaScript code from a source file into an [SwcProgram].
  pub fn parse_js(
    &self,
    fm: &Arc<SourceFile>,
    target: EsVersion,
    syntax: Syntax,
    is_module: IsModule,
    comments: Option<&dyn Comments>,
  ) -> Result<SwcProgram, BatchErrors> {
    let lexer = Lexer::new(syntax, target, SourceFileInput::from(&**fm), comments);
    parse_with_lexer(lexer, is_module, false)
      .map(|(program, _)| program)
      .map_err(|errs| {
        BatchErrors(
          errs
            .dedup_ecma_errors()
            .into_iter()
            .map(|err| ecma_parse_error_deduped_to_rspack_error(err, fm.src.to_string()))
            .collect::<Vec<_>>(),
        )
      })
  }
}
use swc_core::ecma::parser::unstable::Capturing;
fn parse_with_lexer(
  lexer: Lexer,
  is_module: IsModule,
  with_tokens: bool,
) -> Result<(SwcProgram, Option<Vec<TokenAndSpan>>), Vec<parser::error::Error>> {
  let inner = || {
    // don't call capturing when with_tokens is false to avoid performance cost
    let (tokens, program_result, mut errors) = if with_tokens {
      let lexer = Capturing::new(lexer);
      let mut parser = Parser::new_from(lexer);
      let program_result = match is_module {
        IsModule::Bool(true) => parser.parse_module().map(SwcProgram::Module),
        IsModule::Bool(false) => parser.parse_script().map(SwcProgram::Script),
        IsModule::Unknown => parser.parse_program(),
        IsModule::CommonJS => parser.parse_commonjs().map(SwcProgram::Script),
      };
      (
        Some(parser.input_mut().iter_mut().take()),
        program_result,
        parser.take_errors(),
      )
    } else {
      let mut parser = Parser::new_from(lexer);
      let program_result = match is_module {
        IsModule::Bool(true) => parser.parse_module().map(SwcProgram::Module),
        IsModule::Bool(false) => parser.parse_script().map(SwcProgram::Script),
        IsModule::Unknown => parser.parse_program(),
        IsModule::CommonJS => parser.parse_commonjs().map(SwcProgram::Script),
      };
      (None, program_result, parser.take_errors())
    };

    // Using combinator will let rustc unhappy.
    match program_result {
      Ok(program) => {
        if !errors.is_empty() {
          return Err(errors);
        }
        Ok((program, tokens))
      }
      Err(err) => {
        errors.push(err);
        Err(errors)
      }
    }
  };

  // TODO: add stacker to avoid stack overflow
  #[cfg(all(debug_assertions, not(target_family = "wasm")))]
  {
    // Adjust stack to avoid stack overflow.
    stacker::maybe_grow(
      2 * 1024 * 1024, /* 2mb */
      4 * 1024 * 1024, /* 4mb */
      inner,
    )
  }
  #[cfg(any(not(debug_assertions), target_family = "wasm"))]
  inner()
}
