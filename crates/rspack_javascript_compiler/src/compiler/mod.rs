use std::sync::Arc;

mod minify;
mod stringify;
mod transform;

use rspack_error::BatchErrors;
use rspack_sources::SourceMap;
use swc_core::{
  base::config::IsModule,
  common::{
    comments::Comments, input::SourceFileInput, FileName, Globals, SourceFile,
    SourceMap as SwcSourceMap, GLOBALS,
  },
  ecma::{
    ast::{EsVersion, Program as SwcProgram},
    parser::{self, lexer::Lexer, Parser, Syntax},
  },
};
use swc_node_comments::SwcComments;

use crate::{
  ast::Ast,
  error::{ecma_parse_error_deduped_to_rspack_error, DedupEcmaErrors},
};

pub struct JavaScriptCompiler {
  globals: Globals,
  cm: Arc<SwcSourceMap>,
}

impl JavaScriptCompiler {
  pub fn new() -> Self {
    // Initialize globals for swc
    let globals = Globals::default();
    let cm = Arc::new(SwcSourceMap::default());

    Self { globals, cm }
  }

  pub fn parse<S: Into<String>>(
    &self,
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
      .map(|program| Ast::new(program, self.cm.clone(), comments))
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

  fn run<R>(&self, op: impl FnOnce() -> R) -> R {
    GLOBALS.set(&self.globals, op)
  }
}

#[derive(Debug)]
pub struct TransformOutput {
  pub code: String,
  pub map: Option<SourceMap>,
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

  inner()

  // TODO: add stacker to avoid stack overflow
  // #[cfg(all(debug_assertions, not(target_family = "wasm")))]
  // {
  //   // Adjust stack to avoid stack overflow.
  //   stacker::maybe_grow(
  //     2 * 1024 * 1024, /* 2mb */
  //     4 * 1024 * 1024, /* 4mb */
  //     inner,
  //   )
  // }
  // #[cfg(any(not(debug_assertions), target_family = "wasm"))]
  // inner()
}
