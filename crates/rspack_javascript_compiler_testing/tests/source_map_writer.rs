use std::sync::Arc;

use swc_core::{
  common::{BytePos, FileName, LineCol, SourceMap, Span, comments::SingleThreadedComments},
  ecma::{
    ast::EsVersion,
    codegen::{
      Config, Emitter, Node,
      text_writer::{JsWriter, WriteJs, omit_trailing_semi},
    },
    parser::{EsSyntax, Parser, StringInput, Syntax},
  },
};

#[path = "../../rspack_javascript_compiler/src/compiler/stringify/source_map_writer.rs"]
mod source_map_writer;

use source_map_writer::SourceMapWriter;

#[derive(Debug)]
enum Operation {
  Indent,
  Dedent,
  Mapping(u32),
  Keyword(Option<Span>, &'static str),
  Operator(Option<Span>, &'static str),
  Symbol(Span, &'static str),
  Literal(Span, &'static str),
  StringLiteral(Span, &'static str),
  Text(&'static str),
  Comment(&'static str),
  Parameter(&'static str),
  Property(&'static str),
  Line,
  Space,
  Semicolon(Option<Span>),
  Punctuation(Option<Span>, &'static str, bool),
  CommitSemicolon,
}

fn apply(writer: &mut dyn WriteJs, operation: &Operation) {
  match operation {
    Operation::Indent => writer.increase_indent(),
    Operation::Dedent => writer.decrease_indent(),
    Operation::Mapping(position) => writer.add_srcmap(BytePos(*position)),
    Operation::Keyword(span, text) => writer.write_keyword(*span, text),
    Operation::Operator(span, text) => writer.write_operator(*span, text),
    Operation::Symbol(span, text) => writer.write_symbol(*span, text),
    Operation::Literal(span, text) => writer.write_lit(*span, text),
    Operation::StringLiteral(span, text) => writer.write_str_lit(*span, text),
    Operation::Text(text) => writer.write_str(text),
    Operation::Comment(text) => writer.write_comment(text),
    Operation::Parameter(text) => writer.write_param(text),
    Operation::Property(text) => writer.write_property(text),
    Operation::Line => writer.write_line(),
    Operation::Space => writer.write_space(),
    Operation::Semicolon(span) => writer.write_semi(*span),
    Operation::Punctuation(span, text, commit) => writer.write_punct(*span, text, *commit),
    Operation::CommitSemicolon => writer.commit_pending_semi(),
  }
  .expect("write to a vector");
}

fn compare_operations(operations: &[Operation], preamble: &str, minify: bool) {
  let mut expected_code = vec![];
  let mut actual_code = vec![];
  let mut expected_mappings = vec![];
  let mut actual_mappings = vec![];

  {
    let mut expected = JsWriter::new(
      Arc::default(),
      "\n",
      &mut expected_code,
      Some(&mut expected_mappings),
    );
    let mut actual = SourceMapWriter::new(&mut actual_code, |position, location| {
      actual_mappings.push((position, location));
    });
    expected.preamble(preamble).expect("write preamble");
    actual.preamble(preamble).expect("write preamble");
    let mut expected: Box<dyn WriteJs> = Box::new(expected);
    let mut actual: Box<dyn WriteJs> = Box::new(actual);
    if minify {
      expected = Box::new(omit_trailing_semi(expected));
      actual = Box::new(omit_trailing_semi(actual));
    }
    for operation in operations {
      apply(&mut *expected, operation);
      apply(&mut *actual, operation);
    }
  }

  assert_eq!(actual_code, expected_code, "output: {operations:?}");
  assert_eq!(
    actual_mappings, expected_mappings,
    "mappings: {operations:?}"
  );
}

#[test]
fn deduplicates_non_adjacent_sources_only_at_the_same_generated_position() {
  let mut code = vec![];
  let mut mappings = vec![];
  {
    let mut writer = SourceMapWriter::new(&mut code, |position, location| {
      mappings.push((position, location));
    });
    writer.write_str("x").expect("write to a vector");
    for position in [10, 20, 10, 20] {
      writer
        .add_srcmap(BytePos(position))
        .expect("write to a vector");
    }
    writer.write_str("y").expect("write to a vector");
    writer.add_srcmap(BytePos(10)).expect("write to a vector");
    writer.write_line().expect("write to a vector");
    writer.add_srcmap(BytePos(10)).expect("write to a vector");
    writer.write_str("z").expect("write to a vector");
  }
  assert_eq!(code, b"xy\nz");
  assert_eq!(
    mappings,
    [
      (BytePos(10), LineCol { line: 0, col: 1 }),
      (BytePos(20), LineCol { line: 0, col: 1 }),
      (BytePos(10), LineCol { line: 0, col: 2 }),
      (BytePos(10), LineCol { line: 1, col: 0 }),
    ]
  );
}

#[test]
fn preserves_pending_mappings_indentation_literals_and_semicolons() {
  use Operation::*;
  let span = Span::new(BytePos(10), BytePos(20));
  let operations = [
    Mapping(10),
    Mapping(20),
    Line,
    Indent,
    Mapping(10),
    Literal(span, ""),
    Mapping(20),
    Literal(span, "😀雪\r\nsecond"),
    Mapping(10),
    Mapping(20),
    Mapping(10),
    Semicolon(Some(span)),
    Punctuation(Some(span), "}", false),
    Line,
    Mapping(20),
    StringLiteral(span, "\"雪\""),
    Semicolon(Some(span)),
    Mapping(10),
    Punctuation(None, "[", true),
    Symbol(span, "value"),
    Punctuation(None, "]", false),
    Semicolon(None),
    CommitSemicolon,
    Line,
    Dedent,
    Mapping(0),
    Symbol(span, "end"),
    Mapping(u32::MAX),
    Mapping(u32::MAX),
  ];
  for preamble in ["", "// 😀\r\n", "/* 雪 */", "\r", "\n", "\r\n"] {
    for minify in [false, true] {
      compare_operations(&operations, preamble, minify);
    }
  }
}

#[test]
fn matches_swc_for_randomized_writer_operations() {
  use Operation::*;
  let texts = [
    "",
    "x",
    "😀",
    "雪",
    "\n",
    "\r",
    "\r\n",
    "x\ny",
    "x\ry",
    "x\r\ny",
    "😀\r\n雪\nx",
    "\u{2028}",
    "\u{2029}",
    "</script>",
  ];
  for seed in 1..=256u64 {
    let mut random = seed;
    let mut next = || {
      random ^= random << 13;
      random ^= random >> 7;
      random ^= random << 17;
      random
    };
    let mut indent = 0;
    let mut operations = Vec::with_capacity(512);
    for _ in 0..512 {
      let value = next();
      let source = (value % 7 + 1) as u32;
      let span = Span::new(BytePos(source), BytePos(source + 1));
      let optional_span = (value & 1 == 0).then_some(span);
      let text = texts[(next() % texts.len() as u64) as usize];
      let operation = match value % 20 {
        0 if indent < 8 => {
          indent += 1;
          Indent
        }
        1 if indent > 0 => {
          indent -= 1;
          Dedent
        }
        2 | 3 => Mapping(source),
        4 => Keyword(optional_span, text),
        5 => Operator(optional_span, text),
        6 => Symbol(span, text),
        7 => Literal(span, text),
        8 => StringLiteral(span, text),
        9 => Text(text),
        10 => Comment(text),
        11 => Parameter(text),
        12 => Property(text),
        13 => Line,
        14 => Space,
        15 => Semicolon(optional_span),
        16 => Punctuation(optional_span, text, value & 8 == 0),
        17 => CommitSemicolon,
        18 => Mapping(u32::MAX),
        _ => Mapping(0),
      };
      operations.push(operation);
    }
    for minify in [false, true] {
      compare_operations(&operations, "/* 😀 */\r\n", minify);
    }
  }
}

#[test]
fn matches_swc_for_emitted_programs() {
  let sources = [
    "const café = '😀雪'; let suffix = café.length; console.log(suffix);",
    "function f(a = 1) { if (a) { return a + 1; } else { return 0; } }",
    "/* 😀\r\n雪 */\r\nconst text = `a\r\nb\r😀\n雪`;\r\nconsole.log(text);",
    "class A { #x = 0; static field = 1; m() { return this.#x++; } }",
    "async function f(a) { for await (const x of a) { await x?.f?.(); } }",
    "const x = () => ({ key: /a+/g, ['雪']: '\\uD800' }); x();",
    "function f() { debugger; }; [1].forEach(f); (function() {})();",
    "export const x = <div title='😀'>text<span>{1 + 2}</span></div>;",
    "export * from './a.js'; export { default as x } from './b.js';",
  ];
  for source in sources {
    for minify in [false, true] {
      for ascii_only in [false, true] {
        for inline_script in [false, true] {
          let cfg = Config::default()
            .with_target(EsVersion::EsNext)
            .with_minify(minify)
            .with_ascii_only(ascii_only)
            .with_inline_script(inline_script);
          assert_eq!(
            emit_program(source, cfg, true),
            emit_program(source, cfg, false),
            "{source}",
          );
        }
      }
    }
  }
}

fn emit_program(source: &str, cfg: Config, optimized: bool) -> (Vec<u8>, Vec<(BytePos, LineCol)>) {
  // Emission consumes comments, so each writer needs an independent parse.
  let cm = Arc::<SourceMap>::default();
  let file = cm.new_source_file(Arc::new(FileName::Anon), source.to_owned());
  let comments = SingleThreadedComments::default();
  let mut parser = Parser::new(
    Syntax::Es(EsSyntax {
      jsx: true,
      ..Default::default()
    }),
    StringInput::from(&*file),
    Some(&comments),
  );
  let program = parser.parse_program().expect("valid fixture");
  assert!(parser.take_errors().is_empty());

  let mut code = vec![];
  let mut mappings = vec![];
  {
    let mut writer: Box<dyn WriteJs> = if optimized {
      let mut writer = SourceMapWriter::new(&mut code, |position, location| {
        mappings.push((position, location));
      });
      writer.preamble("/*! 😀 */\n").expect("write to a vector");
      Box::new(writer)
    } else {
      let mut writer = JsWriter::new(cm.clone(), "\n", &mut code, Some(&mut mappings));
      writer.preamble("/*! 😀 */\n").expect("write to a vector");
      Box::new(writer)
    };
    if cfg.minify {
      writer = Box::new(omit_trailing_semi(writer));
    }
    program
      .emit_with(&mut Emitter {
        cfg,
        cm,
        comments: Some(&comments),
        wr: writer,
      })
      .expect("write to a vector");
  }
  (code, mappings)
}
