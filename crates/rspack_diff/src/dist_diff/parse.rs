use oxc_allocator::Allocator;
use oxc_ast::{ast::Program, AstKind, Visit};
use oxc_parser::Parser;
use oxc_span::SourceType;
use swc_core::ecma::{parser::parse_file_as_script, visit as swc_visit};
pub struct DistParseBuilder<'a> {
  source_text: &'a str,
  filename: &'a str,
}
impl swc_visit::Visit for DistParseBuilder<'_> {}
impl<'a> Visit<'a> for DistParseBuilder<'a> {
  fn visit_program(&mut self, program: &'a Program<'a>) {
    dbg!("visit programe");
    for stmt in &program.body {
      self.visit_statement(stmt);
    }
  }
  fn visit_expression_statement(&mut self, stmt: &'a oxc_ast::ast::ExpressionStatement<'a>) {
    let kind = AstKind::ExpressionStatement(stmt);
    dbg!(kind);
  }
}
impl<'a> DistParseBuilder<'a> {
  pub fn new(source_text: &'a str, filename: &'a str) -> Self {
    DistParseBuilder {
      source_text,
      filename,
    }
  }
  fn parse_by_swc(&self) {
    let cm: swc_core::common::SourceMap = Default::default();
    let fm = cm.new_source_file(
      swc_core::common::FileName::Custom(self.filename.to_owned()),
      self.source_text.to_owned(),
    );
    let mut errors = vec![];
    let result = parse_file_as_script(
      &fm,
      Default::default(),
      Default::default(),
      Default::default(),
      &mut errors,
    );
  }
  fn parse_by_oxc(&self) {
    let allocator = Allocator::default();
    let source_type = SourceType::default();
    let parser = Parser::new(&allocator, &self.source_text, source_type).parse();

    if parser.errors.is_empty() {
      let mut dist_parser = DistParseBuilder::new(&self.source_text, self.filename);
      let program = allocator.alloc(parser.program);
      dist_parser.visit_program(program);
    } else {
      for error in parser.errors {
        let error = error.with_source_code(self.source_text.to_owned());
        println!("{error:?}");
      }
    }
  }
}

#[test]
fn test_module() {
  let path = std::path::Path::new(
    "/Users/yangjian/github/rspack/crates/rspack_diff/fixtures/basic/rspack-dist/main.js",
  );
  let source_text = std::fs::read_to_string(path).unwrap();
  let builder = DistParseBuilder::new(&source_text);
  builder.parse_by_oxc()
}
