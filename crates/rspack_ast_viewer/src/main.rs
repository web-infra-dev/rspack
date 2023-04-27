use std::{io::Read, sync::Arc};

use anyhow::{Context, Result};
use clap::Parser;
use swc_core::{
  self,
  common::{errors::HANDLER, FileName, Globals, GLOBALS},
  ecma::{
    ast::*,
    parser::{parse_file_as_module, Syntax, TsConfig},
  },
};
use swc_error_reporters::handler::try_with_handler;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
  /// Module type: "js" | "css" (default: "js")
  #[arg(short, long)]
  r#type: Option<ModuleType>,

  /// Whether to keep span (default: false)
  #[arg(short, long)]
  keep_span: Option<bool>,
}

#[derive(Debug, Copy, Clone)]
enum ModuleType {
  JavaScript,
}

impl<'s> From<&'s str> for ModuleType {
  fn from(value: &'s str) -> Self {
    match &*value.to_ascii_lowercase() {
      "js" => Self::JavaScript,
      "css" => todo!("CSS module type is not supported yet"),
      _ => panic!("Unknown module type: {value}"),
    }
  }
}

fn handle_javascript(input: String, keep_span: bool) -> Result<()> {
  let cm: Arc<swc_core::common::SourceMap> = Default::default();
  let fm = cm.new_source_file(FileName::Anon, input);
  let mut errors = Default::default();

  let ast = try_with_handler(cm, Default::default(), |handler| {
    GLOBALS
      .set(&Globals::default(), || {
        HANDLER.set(handler, || {
          parse_file_as_module(
            &fm,
            Syntax::Typescript(TsConfig {
              tsx: true,
              decorators: true,
              ..Default::default()
            }),
            EsVersion::Es2022,
            None,
            &mut errors,
          )
          .map(Program::Module)
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
  let args = Args::parse();
  let module_type = args.r#type.unwrap_or(ModuleType::JavaScript);
  let keep_span = args.keep_span.unwrap_or(false);

  let mut input = String::new();
  std::io::stdin()
    .read_to_string(&mut input)
    .context("Failed to read from stdin")?;

  match module_type {
    ModuleType::JavaScript => handle_javascript(input, keep_span),
  }
}
