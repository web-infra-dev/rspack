use ast::Module;
use swc::{Compiler, TransformOutput};
use swc_common::{self, sync::Lrc, FileName, SourceMap};
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax};

pub fn compile(code: String, mut ast: Option<Module>) -> (Module, TransformOutput) {
  let filename = "a.js";
  let cm: Lrc<SourceMap> = Default::default();
  let compiler = Compiler::new(cm.clone());
  if ast.is_none() {
    let fm = cm.new_source_file(FileName::Custom(filename.into()), code.into());
    let lexer = Lexer::new(
      Syntax::Typescript(Default::default()),
      Default::default(),
      StringInput::from(&*fm),
      None,
    );
    let mut parser = Parser::new_from(lexer);

    for e in parser.take_errors() {
      println!("parse failed, {}", e.kind().msg());
    }

    ast = parser.parse_module().ok();
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
  return (ast, code);
}
