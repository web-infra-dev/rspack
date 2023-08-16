use std::path::Path;

use oxc_allocator::Allocator;
use oxc_ast::{ast::Program, Visit};
use oxc_parser::Parser;
use oxc_span::SourceType;
use serde_json::{Deserializer, Serializer};
pub struct DistParseBuilder<'a> {
  source_text: &'a str,
}
impl<'a> Visit<'a> for DistParseBuilder<'a> {
  fn visit_program(&mut self, program: &'a Program<'a>) {
    dbg!("visit programe");
    for stmt in &program.body {
      self.visit_statement(stmt);
    }
  }
  fn visit_expression_statement(&mut self, stmt: &'a oxc_ast::ast::ExpressionStatement<'a>) {
    dbg!("visit expression");
    dbg!(stmt);
  }
}
impl<'a> DistParseBuilder<'a> {
  pub fn new(source_text: &'a str) -> Self {
    DistParseBuilder { source_text }
  }
}

fn parse_bundle(source_text: String) {
  let allocator = Allocator::default();
  let source_type = SourceType::default();
  let parser = Parser::new(&allocator, &source_text, source_type).parse();

  if parser.errors.is_empty() {
    let mut dist_parser = DistParseBuilder::new(&source_text);
    let program = allocator.alloc(parser.program);
    dist_parser.visit_program(program);
  } else {
    for error in parser.errors {
      let error = error.with_source_code(source_text.clone());
      println!("{error:?}");
    }
  }
}

#[test]
fn test_module() {
  let path = Path::new(
    "/Users/yangjian/github/rspack/crates/rspack_diff/fixtures/basic/rspack-dist/main.js",
  );
  let source_text = std::fs::read_to_string(path).unwrap();
  parse_bundle(source_text);
}
