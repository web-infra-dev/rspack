use std::fmt;

use console::{style, Style};
use similar::{ChangeTag, TextDiff};
use std::fmt::Write;

struct Line(Option<usize>);

impl fmt::Display for Line {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self.0 {
      None => write!(f, "    "),
      Some(idx) => write!(f, "{:<4}", idx + 1),
    }
  }
}

pub fn pretty_diff_printer<'a>(diff: &'a TextDiff<'a, 'a, 'a, str>) -> String {
  let mut output = String::new();

  for (idx, group) in diff.grouped_ops(3).iter().enumerate() {
    if idx > 0 {
      writeln!(output, "{:-^1$}", "-", 80).unwrap();
    }
    for op in group {
      for change in diff.iter_inline_changes(op) {
        let (sign, s) = match change.tag() {
          ChangeTag::Delete => ("-", Style::new().red()),
          ChangeTag::Insert => ("+", Style::new().green()),
          ChangeTag::Equal => (" ", Style::new().dim()),
        };
        write!(
          output,
          "{}{} |{}",
          style(Line(change.old_index())).dim(),
          style(Line(change.new_index())).dim(),
          s.apply_to(sign).bold(),
        )
        .unwrap();
        for (emphasized, value) in change.iter_strings_lossy() {
          if emphasized {
            write!(output, "{}", s.apply_to(value).underlined().on_black()).unwrap();
          } else {
            write!(output, "{}", s.apply_to(value)).unwrap();
          }
        }
        if change.missing_newline() {
          writeln!(output).unwrap()
        }
      }
    }
  }
  output
}

pub fn diff_and_print(expected: &str, actual: &str) -> String {
  let diff = TextDiff::from_lines(expected, actual);
  pretty_diff_printer(&diff)
}
