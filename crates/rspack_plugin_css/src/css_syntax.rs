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
  serialize_identifier(input, &mut output);
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

fn serialize_identifier(mut input: &str, output: &mut String) {
  if input.is_empty() {
    return;
  }

  if let Some(name) = input.strip_prefix("--") {
    output.push_str("--");
    serialize_name(name, output);
    return;
  }
  if input == "-" {
    output.push_str("\\-");
    return;
  }
  if let Some(name) = input.strip_prefix('-') {
    output.push('-');
    input = name;
  }
  if let Some(digit @ b'0'..=b'9') = input.as_bytes().first().copied() {
    push_hex_escape(output, digit);
    input = &input[1..];
  }
  serialize_name(input, output);
}

fn serialize_name(input: &str, output: &mut String) {
  let mut chunk_start = 0;
  for (index, byte) in input.bytes().enumerate() {
    let escape = match byte {
      b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z' | b'_' | b'-' => continue,
      byte if !byte.is_ascii() => continue,
      b'\0' => CssEscape::Replacement,
      b'\x01'..=b'\x1F' | b'\x7F' => CssEscape::Hex,
      _ => CssEscape::Character,
    };
    output.push_str(&input[chunk_start..index]);
    push_escape(output, byte, escape);
    chunk_start = index + 1;
  }
  output.push_str(&input[chunk_start..]);
}

#[derive(Clone, Copy)]
enum CssEscape {
  Character,
  Hex,
  Replacement,
}

fn push_escape(output: &mut String, byte: u8, escape: CssEscape) {
  match escape {
    CssEscape::Character => {
      output.push('\\');
      output.push(char::from(byte));
    }
    CssEscape::Hex => push_hex_escape(output, byte),
    CssEscape::Replacement => output.push(REPLACEMENT_CHARACTER),
  }
}

fn push_hex_escape(output: &mut String, byte: u8) {
  const HEX_DIGITS: &[u8; 16] = b"0123456789abcdef";

  output.push('\\');
  if byte > 0x0F {
    output.push(char::from(HEX_DIGITS[(byte >> 4) as usize]));
  }
  output.push(char::from(HEX_DIGITS[(byte & 0x0F) as usize]));
  output.push(' ');
}

#[derive(Clone, Copy)]
enum UrlSerialization {
  Unquoted,
  Quoted(u8),
}

/// Serialize a URL replacement as either an unquoted URL value or a CSS
/// string, choosing the same compact representation as webpack while using
/// CSS Syntax-compliant escaping for both forms.
pub(crate) fn serialize_url_value(input: &str) -> String {
  let serialization = choose_url_serialization(input);
  let mut output = String::with_capacity(input.len() + 2);
  if let UrlSerialization::Quoted(quote) = serialization {
    output.push(char::from(quote));
  }
  serialize_url_contents(input, serialization, &mut output);
  if let UrlSerialization::Quoted(quote) = serialization {
    output.push(char::from(quote));
  }
  output
}

fn choose_url_serialization(input: &str) -> UrlSerialization {
  let mut whitespace_or_brackets = 0;
  let mut double_quotes = 0;
  let mut single_quotes = 0;

  for byte in input.bytes() {
    match byte {
      b'\t' | b'\n' | b' ' | b'(' | b')' => whitespace_or_brackets += 1,
      b'"' => double_quotes += 1,
      b'\'' => single_quotes += 1,
      _ => {}
    }
  }

  if whitespace_or_brackets < 2 {
    UrlSerialization::Unquoted
  } else if double_quotes <= single_quotes {
    UrlSerialization::Quoted(b'"')
  } else {
    UrlSerialization::Quoted(b'\'')
  }
}

fn serialize_url_contents(input: &str, serialization: UrlSerialization, output: &mut String) {
  let mut chunk_start = 0;
  for (index, byte) in input.bytes().enumerate() {
    let escape = match byte {
      b'\0' => CssEscape::Replacement,
      b'\x01'..=b'\x1F' | b'\x7F' => CssEscape::Hex,
      b'(' | b')' | b'"' | b'\'' | b'\\' if matches!(serialization, UrlSerialization::Unquoted) => {
        CssEscape::Character
      }
      b'\\' => CssEscape::Character,
      byte if matches!(serialization, UrlSerialization::Quoted(quote) if byte == quote) => {
        CssEscape::Character
      }
      _ => continue,
    };
    output.push_str(&input[chunk_start..index]);
    push_escape(output, byte, escape);
    chunk_start = index + 1;
  }
  output.push_str(&input[chunk_start..]);
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
