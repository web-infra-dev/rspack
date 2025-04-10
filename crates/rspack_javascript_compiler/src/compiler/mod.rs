use std::{
  alloc::Global,
  path::{Path, PathBuf},
  sync::{Arc, LazyLock},
};

use anyhow::{Context, Error};
use rspack_error::{BatchErrors, DiagnosticKind, TraceableError};
use rspack_util::{itoa, swc::minify_file_comments};
use swc_core::{
  base::config::{Config, ConfigFile, IsModule, Options as SwcOptions, Rc},
  common::{
    comments::Comments, FileName, FilePathMapping, Globals, Mark, SourceFile, SourceMap, Span,
    GLOBALS,
  },
  ecma::{
    ast::{EsVersion, Program as SwcProgram},
    parser::{self, lexer::Lexer, Parser, Syntax},
    transforms::base::helpers::{self, Helpers},
  },
};
use swc_ecma_minifier::{
  self,
  option::{MinifyOptions, TopLevelOptions},
};

use crate::{
  ast::Ast,
  error::{ecma_parse_error_deduped_to_rspack_error, DedupEcmaErrors, ErrorSpan},
};

pub struct JavaScriptCompiler {
  globals: Globals,
  cm: Arc<SourceMap>,
}

impl JavaScriptCompiler {
  pub fn new() -> Self {
    // Initialize globals for swc
    let globals = Globals::default();

    Self {
      globals,
      cm: Default::default(),
    }
  }

  pub fn parse<S: Into<String>>(
    &self,
    filename: FileName,
    source: S,
    target: EsVersion,
    syntax: Syntax,
    is_module: IsModule,
    comments: Option<&dyn Comments>,
  ) -> Result<Ast, BatchErrors> {
    let fm = self.cm.new_source_file(&filename, source.into());
    let lexer = Lexer::new(syntax, target, SourceFileInput::from(&*fm), comments);

    parse_with_lexer(lexer, is_module)
      .map(|program| {
        let ast = Ast::new(program, self.cm.clone(), comments, Some(&self.globals));
        ast
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

  pub fn transform(&self) -> Ast {
    todo!("implement transform")
  }

  pub fn minify(&self, ast: &mut Ast, options: MinifyOptions) -> Result<Ast, BatchErrors> {
    let context = ast.get_context();
    let unresolved_mark = &context.unresolved_mark;
    let top_level_mark = &context.top_level_mark;

    // TODO: implement minify fork from [crates/rspack_plugin_swc_js_minimizer/src/minify.rs]
    // let program = helpers::HELPERS.set(&Helpers::new(false), || {});
  }

  pub fn print(&self, ast: Ast) -> Result<TransformOutput, Error> {
    todo!("implement print")
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
