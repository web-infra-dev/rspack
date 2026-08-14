use std::borrow::Cow;

const REPLACEMENT_CHARACTER: char = '\u{FFFD}';

#[derive(Clone, Copy)]
struct Escape {
  end: usize,
  value: Option<char>,
}

fn decode_css_escapes(input: &str, trim_url_whitespace: bool) -> Cow<'_, str> {
  let first_escape_or_null = input.bytes().position(|byte| matches!(byte, b'\\' | b'\0'));

  let Some(first_escape_or_null) = first_escape_or_null else {
    return if trim_url_whitespace {
      Cow::Borrowed(input.trim_matches(is_css_whitespace))
    } else {
      Cow::Borrowed(input)
    };
  };

  let mut output = String::with_capacity(input.len());
  append_raw_segment(
    &mut output,
    &input[..first_escape_or_null],
    trim_url_whitespace,
  );
  let mut last_significant_len = if trim_url_whitespace {
    output.trim_end_matches(is_css_whitespace).len()
  } else {
    output.len()
  };
  let mut position = first_escape_or_null;

  while position < input.len() {
    match input.as_bytes()[position] {
      b'\\' => {
        let escape = parse_escape(input, position);
        if let Some(value) = escape.value {
          output.push(value);
          // Whitespace produced by an escape is part of the URL, rather than
          // raw edge whitespace that normalize_url is allowed to trim.
          last_significant_len = output.len();
        }
        position = escape.end;
      }
      b'\0' => {
        output.push(REPLACEMENT_CHARACTER);
        last_significant_len = output.len();
        position += 1;
      }
      _ => unreachable!("scanner only stops at a CSS escape or null"),
    }

    let next_special = input.as_bytes()[position..]
      .iter()
      .position(|byte| matches!(byte, b'\\' | b'\0'))
      .map_or(input.len(), |offset| position + offset);
    let segment = &input[position..next_special];
    let previous_len = output.len();
    let significant_segment_len = append_raw_segment(&mut output, segment, trim_url_whitespace);
    if trim_url_whitespace {
      if significant_segment_len != 0 {
        last_significant_len = previous_len + significant_segment_len;
      }
    } else {
      last_significant_len = output.len();
    }
    position = next_special;
  }

  if trim_url_whitespace {
    output.truncate(last_significant_len);
  }
  Cow::Owned(output)
}

fn append_raw_segment(output: &mut String, mut segment: &str, trim_url_whitespace: bool) -> usize {
  if trim_url_whitespace && output.is_empty() {
    segment = segment.trim_start_matches(is_css_whitespace);
  }
  output.push_str(segment);
  segment.trim_end_matches(is_css_whitespace).len()
}

fn parse_escape(input: &str, start: usize) -> Escape {
  debug_assert_eq!(input.as_bytes()[start], b'\\');

  let bytes = input.as_bytes();
  let mut position = start + 1;
  if position == bytes.len() {
    // Preserve malformed trailing escapes for compatibility with the source
    // parser. Valid CSS tokens never reach this branch.
    return Escape {
      end: position,
      value: Some('\\'),
    };
  }

  if matches!(bytes[position], b'\n' | b'\r' | b'\x0C') {
    if bytes[position] == b'\r' && bytes.get(position + 1) == Some(&b'\n') {
      position += 1;
    }
    return Escape {
      end: position + 1,
      value: None,
    };
  }

  if bytes[position].is_ascii_hexdigit() {
    let mut value = 0;
    let mut digits = 0;
    while position < bytes.len() && digits < 6 {
      let Some(digit) = hex_digit(bytes[position]) else {
        break;
      };
      value = value * 16 + digit;
      position += 1;
      digits += 1;
    }

    if position < bytes.len() {
      match bytes[position] {
        b' ' | b'\t' | b'\n' | b'\x0C' => position += 1,
        b'\r' => {
          position += 1;
          if bytes.get(position) == Some(&b'\n') {
            position += 1;
          }
        }
        _ => {}
      }
    }

    return Escape {
      end: position,
      value: Some(
        char::from_u32(value)
          .filter(|_| value != 0)
          .unwrap_or(REPLACEMENT_CHARACTER),
      ),
    };
  }

  if bytes[position] == b'\0' {
    return Escape {
      end: position + 1,
      value: Some(REPLACEMENT_CHARACTER),
    };
  }

  let value = input[position..]
    .chars()
    .next()
    .expect("position is within the input");
  Escape {
    end: position + value.len_utf8(),
    value: Some(value),
  }
}

fn hex_digit(byte: u8) -> Option<u32> {
  match byte {
    b'0'..=b'9' => Some((byte - b'0') as u32),
    b'a'..=b'f' => Some((byte - b'a' + 10) as u32),
    b'A'..=b'F' => Some((byte - b'A' + 10) as u32),
    _ => None,
  }
}

const fn is_css_whitespace(character: char) -> bool {
  matches!(character, ' ' | '\t' | '\n' | '\r' | '\u{000C}')
}

pub(crate) fn unescape_identifier(input: &str) -> Cow<'_, str> {
  decode_css_escapes(input, false)
}

pub(crate) fn escape_identifier(input: &str) -> Cow<'_, str> {
  if !identifier_needs_escape(input) {
    return Cow::Borrowed(input);
  }

  let mut output = String::with_capacity(input.len() + 2);
  cssparser::serialize_identifier(input, &mut output)
    .expect("writing CSS identifier to String should not fail");
  Cow::Owned(output)
}

fn identifier_needs_escape(input: &str) -> bool {
  let bytes = input.as_bytes();
  if bytes.is_empty() {
    return false;
  }
  if bytes == b"-"
    || bytes[0].is_ascii_digit()
    || bytes.starts_with(b"-") && bytes[1].is_ascii_digit()
  {
    return true;
  }

  bytes.iter().any(|&byte| {
    byte.is_ascii() && !matches!(byte, b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z' | b'_' | b'-')
  })
}

/// Serialize a URL replacement as a CSS string. Quote minimization is left to
/// the CSS minimizer.
pub(crate) fn serialize_url_value(input: &str) -> String {
  let mut output = String::with_capacity(input.len() + 2);
  cssparser::serialize_string(input, &mut output)
    .expect("writing CSS URL string to String should not fail");
  output
}

pub(crate) fn normalize_url(input: &str) -> Cow<'_, str> {
  let result = decode_css_escapes(input, true);
  if result
    .get(..5)
    .is_some_and(|prefix| prefix.eq_ignore_ascii_case("data:"))
  {
    return result;
  }
  if result.contains('%')
    && let Ok(decoded) = urlencoding::decode(&result)
  {
    return Cow::Owned(decoded.into_owned());
  }
  result
}
