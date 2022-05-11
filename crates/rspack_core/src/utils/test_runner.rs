use std::io::stderr;

use ast::Module;
use swc::{config::IsModule, try_with_handler, Compiler, HandlerOpts, TransformOutput};
use swc_common::{self, sync::Lrc, FileName, SourceMap};
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax, TsConfig};
pub fn compile(code: String, mut ast: Option<Module>) -> (Module, TransformOutput, Compiler) {
  let filename = "a.js";
  let cm: Lrc<SourceMap> = Default::default();
  let compiler = Compiler::new(cm.clone());
  if ast.is_none() {
    let fm = cm.new_source_file(FileName::Custom(filename.into()), code.into());
    let syntax = Syntax::Typescript(TsConfig {
      tsx: true,
      ..Default::default()
    });
    let program = swc::try_with_handler(cm, Default::default(), |handler| {
      compiler.parse_js(
        fm,
        handler,
        ast::EsVersion::Es2022,
        syntax,
        IsModule::Bool(true),
        None,
      )
    })
    .unwrap();
    ast = program.module();
  }
  let ast = ast.unwrap();
  let code = compiler
    .print(
      &ast,
      None,
      None,
      false,
      ast::EsVersion::Es2020,
      swc::config::SourceMapsConfig::Bool(false),
      &Default::default(),
      None,
      false,
      None,
    )
    .unwrap();
  return (ast, code, compiler);
}
