use std::fmt::Write;

/// A byte-oriented, anchored glob expression ready to be compiled. Literal
/// directory prefixes come from the filesystem and bypass glob parsing.
pub struct GlobExpression(String);

impl GlobExpression {
  /// Patterns with special backtracking semantics can use the existing matcher.
  pub fn new(directory: &str, glob: &str) -> Option<Self> {
    fast_glob::validate(glob).ok()?;
    if glob.starts_with('!') {
      return None;
    }
    let mut regex = String::from("(?-u:\\A");
    for byte in directory.bytes() {
      push_literal(&mut regex, byte);
    }
    let bytes = glob.as_bytes();
    let mut index = 0;
    let mut braces = 0;
    while let Some(&byte) = bytes.get(index) {
      let start = index;
      index += 1;
      match byte {
        b'*' => {
          // The legacy interpreter backtracks across brace alternatives
          // containing stars. Keep those patterns on that implementation.
          if braces != 0 {
            return None;
          }
          if bytes.get(index) == Some(&b'*') {
            index += 1;
            while bytes[index..].starts_with(b"/**/") {
              index += 3;
            }
            if &bytes[index..] == b"/**" {
              index += 3;
            }
            if (start == 0 || bytes[index - 3] == b'/')
              && (index == bytes.len() || bytes[index] == b'/')
            {
              if index == bytes.len() {
                regex.push_str("[\\x00-\\xff]*");
              } else {
                index += 1;
                regex.push_str("(?:[^/]*/)*");
              }
              continue;
            }
          }
          regex.push_str("[^/]*");
        }
        b'?' => regex.push_str("[^/]"),
        b'[' => push_class(&mut regex, bytes, &mut index),
        b'{' => {
          braces += 1;
          regex.push_str("(?:");
        }
        b',' if braces > 0 => regex.push('|'),
        b'}' if braces > 0 => {
          braces -= 1;
          regex.push(')');
        }
        b'\\' => push_literal(&mut regex, escaped_byte(bytes, &mut index)),
        _ => push_literal(&mut regex, byte),
      }
    }
    regex.push_str("\\z)");
    Some(Self(regex))
  }
}

fn push_literal(regex: &mut String, byte: u8) {
  write!(regex, "\\x{byte:02x}").expect("writing to a String cannot fail");
}

fn escaped_byte(bytes: &[u8], index: &mut usize) -> u8 {
  let byte = bytes[*index];
  *index += 1;
  match byte {
    b'b' => b'\x08',
    b'n' => b'\n',
    b'r' => b'\r',
    b't' => b'\t',
    byte => byte,
  }
}

fn class_byte(bytes: &[u8], index: &mut usize) -> u8 {
  let byte = bytes[*index];
  *index += 1;
  if byte == b'\\' {
    escaped_byte(bytes, index)
  } else {
    byte
  }
}

fn push_class(regex: &mut String, bytes: &[u8], index: &mut usize) {
  let negated = matches!(bytes[*index], b'^' | b'!');
  *index += usize::from(negated);
  let mut included = [false; 256];
  loop {
    let low = class_byte(bytes, index);
    let high = if bytes[*index] == b'-' && bytes[*index + 1] != b']' {
      *index += 1;
      class_byte(bytes, index)
    } else {
      low
    };
    for byte in low..=high {
      included[usize::from(byte)] = true;
    }
    if bytes[*index] == b']' {
      *index += 1;
      break;
    }
  }

  regex.push('[');
  let mut runs = (0..=u8::MAX)
    .filter(|&byte| byte != b'/' && included[usize::from(byte)] != negated)
    .peekable();
  if runs.peek().is_none() {
    regex.push_str("^\\x00-\\xff");
  }
  while let Some(low) = runs.next() {
    let mut high = low;
    while let Some(&next) = runs.peek()
      && next == high.saturating_add(1)
    {
      high = next;
      runs.next();
    }
    push_literal(regex, low);
    if high != low {
      regex.push('-');
      push_literal(regex, high);
    }
  }
  regex.push(']');
}

impl AsRef<str> for GlobExpression {
  fn as_ref(&self) -> &str {
    &self.0
  }
}
