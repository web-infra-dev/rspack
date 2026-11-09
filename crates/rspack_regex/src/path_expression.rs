use std::fmt::Write;

use regex_syntax::{
  ast::{
    AssertionKind, Ast, ClassPerlKind, ClassSet, ClassSetItem, HexLiteralKind, Literal,
    LiteralKind, RepetitionKind, SpecialLiteralKind, parse::Parser,
  },
  escape_into,
};

use crate::algo::Algo;

pub(super) fn translate(algo: &Algo, source: &str, flags: &str) -> Option<String> {
  if let Algo::EndWith { pats, ignore_case } = algo {
    let mut expression = String::from(r"(?s:.*)");
    expression.push_str(if *ignore_case { "(?i-u:" } else { "(?:" });
    for (index, suffix) in pats.iter().enumerate() {
      if index != 0 {
        expression.push('|');
      }
      escape_into(suffix, &mut expression);
    }
    expression.push_str(r")\z");
    return (!pats.is_empty()).then_some(expression);
  }
  if !matches!(algo, Algo::Regress(_)) || !flags.bytes().all(|f| matches!(f, b'u' | b's')) {
    return None;
  }

  let ast = Parser::new().parse(source).ok()?;
  let mut output = String::new();
  // The scanner checks matches at end of input. Allow a prefix and suffix
  // around a JS search match, except when a start anchor already fixes it.
  if !starts_at_beginning(&ast) {
    output.push_str(r"(?s:.*)");
  }
  output.push_str("(?:");
  let translator = Translator {
    source,
    unicode: flags.contains('u'),
    dot_all: flags.contains('s'),
  };
  translator.push(&ast, false, &mut output)?;
  output.push_str(r")(?s:.*)\z");
  Some(output)
}

fn starts_at_beginning(ast: &Ast) -> bool {
  match ast {
    Ast::Assertion(assertion) => assertion.kind == AssertionKind::StartLine,
    Ast::Group(group) => starts_at_beginning(&group.ast),
    Ast::Concat(concat) => concat.asts.first().is_some_and(starts_at_beginning),
    Ast::Alternation(alternation) => alternation.asts.iter().all(starts_at_beginning),
    _ => false,
  }
}

struct Translator<'a> {
  source: &'a str,
  unicode: bool,
  dot_all: bool,
}

impl Translator<'_> {
  fn push(&self, ast: &Ast, zero_or_more: bool, output: &mut String) -> Option<()> {
    match ast {
      Ast::Empty(_) => {}
      Ast::Literal(literal) if self.supports_literal(literal) => {
        write!(output, r"\x{{{:x}}}", literal.c as u32).expect("writing to a String cannot fail");
      }
      // JS without /u counts UTF-16 code units. An unrestricted run with
      // no minimum accepts the same complete strings in UTF-8.
      Ast::Dot(_) if self.unicode || zero_or_more => {
        output.push_str(if self.dot_all {
          r"(?s:.)"
        } else {
          r"[^\n\r\u{2028}\u{2029}]"
        });
      }
      Ast::Assertion(assertion) => output.push_str(match assertion.kind {
        AssertionKind::StartLine => r"\A",
        AssertionKind::EndLine => r"\z",
        AssertionKind::WordBoundary => r"(?-u:\b)",
        AssertionKind::NotWordBoundary => r"(?-u:\B)",
        _ => return None,
      }),
      Ast::ClassPerl(class) if !class.negated || self.unicode || zero_or_more => {
        let members = match class.kind {
          ClassPerlKind::Digit => "0-9",
          ClassPerlKind::Word => "A-Za-z0-9_",
          ClassPerlKind::Space => return None,
        };
        output.push('[');
        if class.negated {
          output.push('^');
        }
        output.push_str(members);
        output.push(']');
      }
      Ast::ClassBracketed(class)
        if (!class.negated || self.unicode || zero_or_more)
          && matches!(&class.kind, ClassSet::Item(item) if self.supports_class_item(item)) =>
      {
        output.push_str(&self.source[class.span.start.offset..class.span.end.offset]);
      }
      Ast::Group(group) if group.flags().is_none_or(|flags| flags.items.is_empty()) => {
        output.push_str("(?:");
        self.push(&group.ast, false, output)?;
        output.push(')');
      }
      Ast::Repetition(repetition) => {
        let zero_or_more = repetition.op.kind == RepetitionKind::ZeroOrMore;
        self.push(&repetition.ast, zero_or_more, output)?;
        let span = &repetition.op.span;
        output.push_str(&self.source[span.start.offset..span.end.offset]);
      }
      Ast::Concat(concat) => {
        for item in &concat.asts {
          self.push(item, false, output)?;
        }
      }
      Ast::Alternation(alternation) => {
        for (index, item) in alternation.asts.iter().enumerate() {
          if index != 0 {
            output.push('|');
          }
          self.push(item, false, output)?;
        }
      }
      _ => return None,
    }
    Some(())
  }

  fn supports_literal(&self, literal: &Literal) -> bool {
    if !self.unicode && (literal.c as u32) > 0xffff {
      return false;
    }
    match &literal.kind {
      LiteralKind::Verbatim | LiteralKind::Meta | LiteralKind::Superfluous => true,
      LiteralKind::HexFixed(HexLiteralKind::X | HexLiteralKind::UnicodeShort) => true,
      LiteralKind::HexBrace(HexLiteralKind::UnicodeShort) => self.unicode,
      LiteralKind::Special(kind) => {
        !matches!(kind, SpecialLiteralKind::Bell | SpecialLiteralKind::Space)
      }
      _ => false,
    }
  }

  fn supports_class_item(&self, item: &ClassSetItem) -> bool {
    match item {
      ClassSetItem::Literal(literal) => self.supports_literal(literal),
      ClassSetItem::Range(range) => {
        self.supports_literal(&range.start)
          && self.supports_literal(&range.end)
          && (self.unicode || (range.start.c as u32) > 0xdfff || (range.end.c as u32) < 0xd800)
      }
      ClassSetItem::Union(union) => union
        .items
        .iter()
        .all(|item| self.supports_class_item(item)),
      _ => false,
    }
  }
}
