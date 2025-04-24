use std::sync::Arc;

use rspack_error::BatchErrors;
use swc_core::{
  base::config::IsModule,
  common::{comments::Comments, input::SourceFileInput, FileName, SourceFile},
  ecma::{
    ast::{EsVersion, Program as SwcProgram},
    parser::{self, lexer::Lexer, Parser, Syntax},
  },
};
use swc_node_comments::SwcComments;

use super::JavaScriptCompiler;
use crate::{
  ast::{self, Ast},
  error::{ecma_parse_error_deduped_to_rspack_error, DedupEcmaErrors},
};

impl JavaScriptCompiler {
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

    parse_with_lexer(lexer, is_module)
      .map(|program| {
        Ast::new(program, self.cm.clone(), comments)
          .with_context(ast::Context::new(self.cm, Some(self.globals)))
      })
      .map_err(|errs| {
        BatchErrors(
          errs
            .dedup_ecma_errors()
            .into_iter()
            .map(|err| {
              rspack_error::miette::Error::new(ecma_parse_error_deduped_to_rspack_error(err, &fm))
            })
            .collect::<Vec<_>>(),
        )
      })
  }

  pub fn parse_with_lexer(
    self,
    fm: &SourceFile,
    lexer: Lexer,
    is_module: IsModule,
    comments: Option<SwcComments>,
  ) -> Result<Ast, BatchErrors> {
    parse_with_lexer(lexer, is_module)
      .map(|program| {
        Ast::new(program, self.cm.clone(), comments)
          .with_context(ast::Context::new(self.cm.clone(), Some(self.globals)))
      })
      .map_err(|errs| {
        BatchErrors(
          errs
            .dedup_ecma_errors()
            .into_iter()
            .map(|err| {
              rspack_error::miette::Error::new(ecma_parse_error_deduped_to_rspack_error(err, fm))
            })
            .collect::<Vec<_>>(),
        )
      })
  }

  pub fn parse_js(
    &self,
    fm: Arc<SourceFile>,
    target: EsVersion,
    syntax: Syntax,
    is_module: IsModule,
    comments: Option<&dyn Comments>,
  ) -> Result<SwcProgram, BatchErrors> {
    let lexer = Lexer::new(syntax, target, SourceFileInput::from(&*fm), comments);
    parse_with_lexer(lexer, is_module).map_err(|errs| {
      BatchErrors(
        errs
          .dedup_ecma_errors()
          .into_iter()
          .map(|err| {
            rspack_error::miette::Error::new(ecma_parse_error_deduped_to_rspack_error(err, &fm))
          })
          .collect::<Vec<_>>(),
      )
    })
  }
}

fn parse_with_lexer(
  lexer: Lexer,
  is_module: IsModule,
) -> Result<SwcProgram, Vec<parser::error::Error>> {
  let inner = || {
    let mut parser = Parser::new_from(lexer);
    let program_result = match is_module {
      IsModule::Bool(true) => parser.parse_module().map(SwcProgram::Module),
      IsModule::Bool(false) => parser.parse_script().map(SwcProgram::Script),
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
