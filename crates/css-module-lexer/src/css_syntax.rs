pub(crate) const MAX_CSS_KEYWORD_LEN: usize = "-webkit-animation-name".len();

#[inline]
pub(crate) fn is_css_space_byte(byte: u8) -> bool {
  matches!(byte, b' ' | b'\t' | b'\n' | b'\r' | 0x0C)
}

pub(crate) fn is_css_white_space_char(c: char) -> bool {
  matches!(c, ' ' | '\t' | '\n' | '\r' | '\u{c}')
}

pub(crate) fn trim_css_whitespace(input: &str) -> &str {
  let bytes = input.as_bytes();
  let mut start = 0usize;
  let mut end = bytes.len();
  while start < end && is_css_space_byte(bytes[start]) {
    start += 1;
  }
  while end > start && is_css_space_byte(bytes[end - 1]) {
    end -= 1;
  }
  &input[start..end]
}

pub(crate) fn is_css_modules_magic_comment(input: &str, keyword: &str) -> bool {
  let input = trim_css_whitespace(input);
  let Some(rest) = input.strip_prefix(keyword) else {
    return false;
  };
  rest.is_empty()
    || rest
      .as_bytes()
      .first()
      .is_some_and(|byte| is_css_space_byte(*byte))
}

pub(crate) fn is_css_modules_pure_magic_comment(input: &str) -> bool {
  let input = trim_css_whitespace(input);
  let Some(keyword) = input.strip_prefix("cssmodules-pure-") else {
    return false;
  };
  ["ignore", "no-check"].into_iter().any(|expected| {
    keyword.strip_prefix(expected).is_some_and(|rest| {
      rest.is_empty()
        || rest
          .as_bytes()
          .first()
          .is_some_and(|byte| is_css_space_byte(*byte))
    })
  })
}

/// Return the name after one logical `--` prefix from a dashed identifier.
///
/// The common literal spelling exits immediately. Escaped prefixes are decoded
/// only far enough to inspect the first two code points, without allocating.
pub(crate) fn dashed_ident_name(input: &str) -> Option<&str> {
  let name_start = dashed_ident_name_start(input)?;
  input.get(name_start..)
}

/// Return the byte offset after one logical `--` prefix.
pub(crate) fn dashed_ident_name_start(input: &str) -> Option<usize> {
  if input.starts_with("--") {
    return Some(2);
  }
  let bytes = input.as_bytes();
  let (first, position) = next_css_code_point(bytes, 0)?;
  if first != b'-' as u32 {
    return None;
  }
  let (second, position) = next_css_code_point(bytes, position)?;
  (second == b'-' as u32).then_some(position)
}

fn next_css_code_point(input: &[u8], position: usize) -> Option<(u32, usize)> {
  let &first = input.get(position)?;
  if first != b'\\' {
    return Some((first as u32, position + 1));
  }

  let mut position = position + 1;
  let &first = input.get(position)?;
  if !first.is_ascii_hexdigit() {
    if matches!(first, b'\n' | b'\r' | 0x0C) {
      return None;
    }
    return Some((first as u32, position + 1));
  }

  let mut value = 0u32;
  let mut digits = 0;
  while let Some(&digit) = input.get(position) {
    if digits == 6 || !digit.is_ascii_hexdigit() {
      break;
    }
    value = value * 16 + hex_digit_value(digit) as u32;
    position += 1;
    digits += 1;
  }
  if input
    .get(position)
    .is_some_and(|byte| is_css_space_byte(*byte))
  {
    let whitespace = input[position];
    position += 1;
    if whitespace == b'\r' && input.get(position) == Some(&b'\n') {
      position += 1;
    }
  }
  Some((value, position))
}

/// Lowercase a keyword that does not contain CSS escapes.
pub(crate) fn lowercase_ascii_keyword<'a>(
  input: &str,
  output: &'a mut [u8; MAX_CSS_KEYWORD_LEN],
) -> Option<&'a str> {
  let output = output.get_mut(..input.len())?;
  for (output, input) in output.iter_mut().zip(input.bytes()) {
    *output = input.to_ascii_lowercase();
  }
  // ASCII case conversion preserves valid UTF-8.
  Some(unsafe { std::str::from_utf8_unchecked(output) })
}

/// Decode an escaped CSS identifier into a lowercase ASCII keyword.
///
/// The dependency parser only compares CSS syntax keywords, so non-ASCII or
/// replacement-character escapes can stop early. The original source slice is
/// still retained for dependency names and ranges.
pub(crate) fn decode_css_keyword<'a>(
  input: &str,
  output: &'a mut [u8; MAX_CSS_KEYWORD_LEN],
) -> Option<&'a str> {
  let input = input.as_bytes();
  let mut input_position = 0usize;
  let mut output_position = 0usize;

  while input_position < input.len() {
    let byte = if input[input_position] == b'\\' {
      input_position += 1;
      let &first = input.get(input_position)?;
      if first.is_ascii_hexdigit() {
        let mut value = 0u32;
        let mut digits = 0;
        while let Some(&digit) = input.get(input_position) {
          if digits == 6 || !digit.is_ascii_hexdigit() {
            break;
          }
          value = value * 16 + hex_digit_value(digit) as u32;
          input_position += 1;
          digits += 1;
        }
        if input
          .get(input_position)
          .is_some_and(|byte| is_css_space_byte(*byte))
        {
          input_position += 1;
          if input.get(input_position - 1) == Some(&b'\r')
            && input.get(input_position) == Some(&b'\n')
          {
            input_position += 1;
          }
        }
        let byte = u8::try_from(value).ok()?;
        (byte != 0 && byte.is_ascii()).then_some(byte)?
      } else {
        if !first.is_ascii() || matches!(first, b'\n' | b'\r' | 0x0C) {
          return None;
        }
        input_position += 1;
        first
      }
    } else {
      let byte = input[input_position];
      if !byte.is_ascii() || byte == 0 {
        return None;
      }
      input_position += 1;
      byte
    };

    *output.get_mut(output_position)? = byte.to_ascii_lowercase();
    output_position += 1;
  }

  // All written bytes are lowercase ASCII.
  Some(unsafe { std::str::from_utf8_unchecked(&output[..output_position]) })
}

pub(crate) fn strip_vendor_prefix(input: &str) -> Option<&str> {
  ["-webkit-", "-moz-", "-ms-", "-o-"]
    .into_iter()
    .find_map(|prefix| input.strip_prefix(prefix))
}

fn hex_digit_value(byte: u8) -> u8 {
  match byte {
    b'0'..=b'9' => byte - b'0',
    b'a'..=b'f' => byte - b'a' + 10,
    b'A'..=b'F' => byte - b'A' + 10,
    _ => unreachable!("hex digit checked by caller"),
  }
}
