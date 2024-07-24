use std::{io::Read, sync::Arc};

use anyhow::{Context, Result};
use argh::FromArgs;
use swc_core::{
  self,
  common::{errors::HANDLER, FileName, Globals, Mark, GLOBALS},
  ecma::{
    ast::*,
    parser::{parse_file_as_module, Syntax, TsSyntax},
    transforms::base::resolver,
    visit::VisitMutWith,
  },
};
use swc_error_reporters::handler::try_with_handler;

#[derive(FromArgs)]
/// rspack ast viewer
struct Args {
  /// module type: "js" | "css" (default: "js")
  #[argh(option, short = 't')]
  ty: Option<String>,

  /// whether to keep span (default: false)
  #[argh(switch, short = 'k')]
  keep_span: bool,
}

enum ModuleType {
  JavaScript,
}

fn from_str_fn(ty: &str) -> ModuleType {
  match ty.to_ascii_lowercase().as_str() {
    "js" => ModuleType::JavaScript,
    "css" => todo!("CSS module type is not supported yet"),
    _ => panic!("Unknown module type: {ty}"),
  }
}

fn handle_javascript(input: String, keep_span: bool) -> Result<()> {
  let cm: Arc<swc_core::common::SourceMap> = Default::default();
  let fm = cm.new_source_file(FileName::Anon, input);
  let mut errors = Default::default();

  let ast = try_with_handler(cm, Default::default(), |handler| {
    GLOBALS
      .set(&Globals::default(), || {
        let unresolved_mark = Mark::new();
        let top_level_mark = Mark::new();

        HANDLER.set(handler, || {
          parse_file_as_module(
            &fm,
            Syntax::Typescript(TsSyntax {
              tsx: true,
              decorators: true,
              ..Default::default()
            }),
            EsVersion::EsNext,
            None,
            &mut errors,
          )
          .map(Program::Module)
          .map(|mut program| {
            program.visit_mut_with(&mut resolver(unresolved_mark, top_level_mark, false));
            program
          })
        })
      })
      .map_err(|err| anyhow::anyhow!("{err:?}"))
  })?;

  if !errors.is_empty() {
    anyhow::bail!("{errors:?}")
  }

  let output = {
    let output = format!("{ast:#?}");

    if keep_span {
      output
    } else {
      let reg = regex::Regex::new(r#"\s*span: Span \{[^}]*?\},"#)?;
      reg.replace_all(&output, "").into_owned()
    }
  };

  println!("{output}");

  Ok(())
}

fn main() -> Result<()> {
  let args: Args = argh::from_env();
  let module_type = args
    .ty
    .as_ref()
    .map(|ty| from_str_fn(ty))
    .unwrap_or(ModuleType::JavaScript);
  let keep_span = args.keep_span;

  let mut input = String::new();
  std::io::stdin()
    .read_to_string(&mut input)
    .context("Failed to read from stdin")?;

  match module_type {
    ModuleType::JavaScript => handle_javascript(input, keep_span),
  }
}
