//! Hand-written replacements for the regular expressions in `parser.js`.
//!
//! Each function is anchored at the start of `s` (like the `^` of the
//! original regex) and returns the length of the match in bytes. The original
//! regex is quoted in the doc comment.

#[inline]
pub fn is_word(b: u8) -> bool {
  b.is_ascii_alphanumeric() || b == b'_'
}
#[inline]
pub fn is_word_dash(b: u8) -> bool {
  is_word(b) || b == b'-'
}
#[inline]
pub fn is_hex(b: u8) -> bool {
  b.is_ascii_hexdigit()
}
#[inline]
pub fn is_space(b: u8) -> bool {
  matches!(b, b' ' | b'\t' | b'\n' | b'\r' | 0x0b | 0x0c)
}

/// UTF-8 sequence length of the char starting with byte `b`.
#[inline]
pub fn utf8_len(b: u8) -> usize {
  if b < 0x80 {
    1
  } else if b >> 5 == 0b110 {
    2
  } else if b >> 4 == 0b1110 {
    3
  } else {
    4
  }
}

/// Decodes the code point at `s[i]` (assumes valid UTF-8).
pub fn code_point_at(s: &[u8], i: usize) -> (u32, usize) {
  let b = s[i];
  let n = utf8_len(b);
  let cp = match n {
    1 => b as u32,
    2 => ((b as u32 & 0x1f) << 6) | (s[i + 1] as u32 & 0x3f),
    3 => ((b as u32 & 0x0f) << 12) | ((s[i + 1] as u32 & 0x3f) << 6) | (s[i + 2] as u32 & 0x3f),
    _ => {
      ((b as u32 & 0x07) << 18)
        | ((s[i + 1] as u32 & 0x3f) << 12)
        | ((s[i + 2] as u32 & 0x3f) << 6)
        | (s[i + 3] as u32 & 0x3f)
    }
  };
  (cp, n)
}

/// `\\(?:[A-Fa-f0-9]{1,6} ?|[^A-Fa-f0-9])` at `i` (which must be `\\`).
fn css_escape(s: &[u8], i: usize) -> Option<usize> {
  if s.get(i) != Some(&b'\\') {
    return None;
  }
  let mut j = i + 1;
  let c = *s.get(j)?;
  if is_hex(c) {
    let start = j;
    while j < s.len() && j - start < 6 && is_hex(s[j]) {
      j += 1;
    }
    if s.get(j) == Some(&b' ') {
      j += 1;
    }
    Some(j - i)
  } else {
    // `[^A-Fa-f0-9]` matches any single UTF-16 unit (incl. newlines).
    Some(1 + utf8_len(c))
  }
}

/// `(?:[\w-]|\\(?:[A-Fa-f0-9]{1,6} ?|[^A-Fa-f0-9]))+` starting at `i`.
fn ident_escaped_run(s: &[u8], mut i: usize) -> usize {
  let start = i;
  while i < s.len() {
    let c = s[i];
    if is_word_dash(c) {
      i += 1;
    } else if c == b'\\' {
      match css_escape(s, i) {
        Some(n) => i += n,
        None => break,
      }
    } else {
      break;
    }
  }
  i - start
}

/// keyword: `/^\[?(?:[\w-]|\\(?:[A-Fa-f0-9]{1,6} ?|[^A-Fa-f0-9]))+\]?/`
pub fn keyword(s: &[u8]) -> Option<usize> {
  let mut i = 0;
  if s.first() == Some(&b'[') {
    i = 1;
  }
  let n = ident_escaped_run(s, i);
  if n == 0 {
    return None;
  }
  i += n;
  if s.get(i) == Some(&b']') {
    i += 1;
  }
  Some(i)
}

/// call: `/^([\w-]+|%|~|progid:[\w.]+)\(/` → (total length, name length)
pub fn call_name(s: &[u8]) -> Option<(usize, usize)> {
  if s.starts_with(b"progid:") {
    let mut i = 7;
    while i < s.len() && (is_word(s[i]) || s[i] == b'.') {
      i += 1;
    }
    if i > 7 && s.get(i) == Some(&b'(') {
      return Some((i + 1, i));
    }
    // fall through: `progid` could still match [\w-]+ but then ':' fails.
  }
  let first = *s.first()?;
  if first == b'%' || first == b'~' {
    if s.get(1) == Some(&b'(') {
      return Some((2, 1));
    }
    return None;
  }
  let mut i = 0;
  while i < s.len() && is_word_dash(s[i]) {
    i += 1;
  }
  if i > 0 && s.get(i) == Some(&b'(') {
    return Some((i + 1, i));
  }
  None
}

/// `/^[\w]+\(/`
pub fn word_paren(s: &[u8]) -> Option<usize> {
  let mut i = 0;
  while i < s.len() && is_word(s[i]) {
    i += 1;
  }
  if i > 0 && s.get(i) == Some(&b'(') {
    Some(i + 1)
  } else {
    None
  }
}

/// assignment key: `/^\w+(?=\s?=)/i`
pub fn assignment_key(s: &[u8]) -> Option<usize> {
  let mut i = 0;
  while i < s.len() && is_word(s[i]) {
    i += 1;
  }
  if i == 0 {
    return None;
  }
  // The `+` is greedy but can backtrack: any prefix followed by `\s?=`.
  // Only the full run can be followed by `=` or space, since a shorter
  // prefix is followed by a word char.
  let rest = &s[i..];
  if rest.first() == Some(&b'=')
    || (rest.first().map(|&b| is_space(b)).unwrap_or(false) && rest.get(1) == Some(&b'='))
  {
    Some(i)
  } else {
    None
  }
}

/// url body: `/^(?:(?:\\[()'"])|[^()'"])+/`
pub fn url_body(s: &[u8]) -> Option<usize> {
  let mut i = 0;
  while i < s.len() {
    let c = s[i];
    if c == b'\\' && matches!(s.get(i + 1), Some(b'(' | b')' | b'\'' | b'"')) {
      i += 2;
    } else if matches!(c, b'(' | b')' | b'\'' | b'"') {
      break;
    } else {
      i += utf8_len(c);
    }
  }
  if i == 0 { None } else { Some(i.min(s.len())) }
}

/// `/^@@?[\w-]+/`
pub fn variable_name(s: &[u8]) -> Option<usize> {
  if s.first() != Some(&b'@') {
    return None;
  }
  let mut i = 1;
  if s.get(1) == Some(&b'@') {
    i = 2;
  }
  let start = i;
  while i < s.len() && is_word_dash(s[i]) {
    i += 1;
  }
  if i == start {
    // `@@` followed by nothing: try with a single `@`.
    if start == 2 {
      return None;
    }
    return None;
  }
  Some(i)
}

/// `/^@\{([\w-]+)/` → (length, name)
pub fn variable_curly_start(s: &[u8]) -> Option<usize> {
  if !s.starts_with(b"@{") {
    return None;
  }
  let mut i = 2;
  while i < s.len() && is_word_dash(s[i]) {
    i += 1;
  }
  if i == 2 { None } else { Some(i) }
}

/// `/^\$[\w-]+/`
pub fn property_name(s: &[u8]) -> Option<usize> {
  if s.first() != Some(&b'$') {
    return None;
  }
  let mut i = 1;
  while i < s.len() && is_word_dash(s[i]) {
    i += 1;
  }
  if i == 1 { None } else { Some(i) }
}

/// color: `/^#([A-Fa-f0-9]{8}|[A-Fa-f0-9]{6}|[A-Fa-f0-9]{3,4})([\w.#[])?/`
/// → (length, hex digits length, has trailing group)
pub fn hex_color(s: &[u8]) -> Option<(usize, usize, bool)> {
  if s.first() != Some(&b'#') {
    return None;
  }
  let mut n = 0;
  while 1 + n < s.len() && is_hex(s[1 + n]) && n < 8 {
    n += 1;
  }
  let digits = if n >= 8 {
    8
  } else if n >= 6 {
    6
  } else if n >= 3 {
    n.min(4)
  } else {
    return None;
  };
  let mut len = 1 + digits;
  let trailing = s
    .get(len)
    .map(|&b| is_word(b) || b == b'.' || b == b'#' || b == b'[')
    .unwrap_or(false);
  if trailing {
    len += 1;
  }
  Some((len, digits, trailing))
}

/// colorKeyword: `/^[_A-Za-z-][_A-Za-z0-9-]+/`
pub fn color_keyword(s: &[u8]) -> Option<usize> {
  let first = *s.first()?;
  if !(first == b'_' || first == b'-' || first.is_ascii_alphabetic()) {
    return None;
  }
  let mut i = 1;
  while i < s.len() && is_word_dash(s[i]) {
    i += 1;
  }
  if i < 2 { None } else { Some(i) }
}

/// dimension: `/^([+-]?\d*\.?\d+)(%|[a-z_]+)?/i` → (length, number length)
pub fn dimension(s: &[u8]) -> Option<(usize, usize)> {
  let mut i = 0;
  if matches!(s.first(), Some(b'+' | b'-')) {
    i = 1;
  }
  let int_start = i;
  while i < s.len() && s[i].is_ascii_digit() {
    i += 1;
  }
  let int_end = i;
  let num_end;
  if s.get(i) == Some(&b'.') && s.get(i + 1).map(|b| b.is_ascii_digit()).unwrap_or(false) {
    i += 1;
    while i < s.len() && s[i].is_ascii_digit() {
      i += 1;
    }
    num_end = i;
  } else if int_end > int_start {
    // `\d*\.?\d+` with no dot: the digits themselves.
    num_end = int_end;
  } else {
    return None;
  }
  let mut j = num_end;
  if s.get(j) == Some(&b'%') {
    j += 1;
  } else {
    while j < s.len() && (s[j].is_ascii_alphabetic() || s[j] == b'_') {
      j += 1;
    }
  }
  Some((j, num_end))
}

/// unicodeDescriptor: `/^U\+[0-9a-fA-F?]+(-[0-9a-fA-F?]+)?/`
pub fn unicode_descriptor(s: &[u8]) -> Option<usize> {
  if !s.starts_with(b"U+") {
    return None;
  }
  let hx = |b: u8| is_hex(b) || b == b'?';
  let mut i = 2;
  while i < s.len() && hx(s[i]) {
    i += 1;
  }
  if i == 2 {
    return None;
  }
  if s.get(i) == Some(&b'-') && s.get(i + 1).map(|&b| hx(b)).unwrap_or(false) {
    i += 1;
    while i < s.len() && hx(s[i]) {
      i += 1;
    }
  }
  Some(i)
}

/// `/^[^`]*`/`
pub fn until_backtick(s: &[u8]) -> Option<usize> {
  s.iter().position(|&b| b == b'`').map(|p| p + 1)
}

/// variable definition: `/^(@[\w-]+)\s*:/` → (length, name length)
pub fn variable_def(s: &[u8]) -> Option<(usize, usize)> {
  if s.first() != Some(&b'@') {
    return None;
  }
  let mut i = 1;
  while i < s.len() && is_word_dash(s[i]) {
    i += 1;
  }
  if i == 1 {
    return None;
  }
  let name = i;
  while i < s.len() && is_space(s[i]) {
    i += 1;
  }
  if s.get(i) == Some(&b':') {
    Some((i + 1, name))
  } else {
    None
  }
}

/// variableCall: `/^(@[\w-]+)(\(\s*\))?/` → (length, name length, has parens)
pub fn variable_call(s: &[u8]) -> Option<(usize, usize, bool)> {
  if s.first() != Some(&b'@') {
    return None;
  }
  let mut i = 1;
  while i < s.len() && is_word_dash(s[i]) {
    i += 1;
  }
  if i == 1 {
    return None;
  }
  let name = i;
  if s.get(i) == Some(&b'(') {
    let mut j = i + 1;
    while j < s.len() && is_space(s[j]) {
      j += 1;
    }
    if s.get(j) == Some(&b')') {
      return Some((j + 1, name, true));
    }
  }
  Some((name, name, false))
}

/// extend option: `/^(!?all)(?=\s*(\)|,))/`
pub fn extend_all(s: &[u8]) -> Option<usize> {
  let mut i = 0;
  if s.first() == Some(&b'!') {
    i = 1;
  }
  if !s[i..].starts_with(b"all") {
    return None;
  }
  i += 3;
  let mut j = i;
  while j < s.len() && is_space(s[j]) {
    j += 1;
  }
  if matches!(s.get(j), Some(b')' | b',')) {
    Some(i)
  } else {
    None
  }
}

/// mixin element: `/^[#.](?:[\w-]|\\(?:[A-Fa-f0-9]{1,6} ?|[^A-Fa-f0-9]))+/`
pub fn mixin_element(s: &[u8]) -> Option<usize> {
  if !matches!(s.first(), Some(b'#' | b'.')) {
    return None;
  }
  let n = ident_escaped_run(s, 1);
  if n == 0 { None } else { Some(1 + n) }
}

/// mixin definition: `/^([#.](?:[\w-]|\\(?:...))+)\s*\(/` → (length, name length)
pub fn mixin_def_start(s: &[u8]) -> Option<(usize, usize)> {
  let name = mixin_element(s)?;
  let mut i = name;
  while i < s.len() && is_space(s[i]) {
    i += 1;
  }
  if s.get(i) == Some(&b'(') {
    Some((i + 1, name))
  } else {
    None
  }
}

/// lookupValue: `/^(?:[@$]{0,2})[_a-zA-Z0-9-]*/` (may match empty)
pub fn lookup_value(s: &[u8]) -> usize {
  let mut i = 0;
  while i < 2 && matches!(s.get(i), Some(b'@' | b'$')) {
    i += 1;
  }
  while i < s.len() && is_word_dash(s[i]) {
    i += 1;
  }
  i
}

/// `/^\d+/`
pub fn digits(s: &[u8]) -> Option<usize> {
  let n = s.iter().take_while(|b| b.is_ascii_digit()).count();
  if n == 0 { None } else { Some(n) }
}

/// element: `/^(?:\d+\.\d+|\d+)%/`
pub fn percentage_element(s: &[u8]) -> Option<usize> {
  let mut i = digits(s)?;
  if s.get(i) == Some(&b'.')
    && let Some(n) = digits(&s[i + 1..])
    && s.get(i + 1 + n) == Some(&b'%')
  {
    return Some(i + 1 + n + 1);
  }
  if s.get(i) == Some(&b'%') {
    i += 1;
    return Some(i);
  }
  None
}

/// element: `/^(?:[.#]?|:*)(?:[\w-]|[^\x00-\x9f]|\\(?:[A-Fa-f0-9]{1,6} ?|[^A-Fa-f0-9]))+/`
pub fn element_name(s: &[u8]) -> Option<usize> {
  fn run(s: &[u8], mut i: usize) -> usize {
    let start = i;
    while i < s.len() {
      let c = s[i];
      if is_word_dash(c) {
        i += 1;
      } else if c >= 0x80 {
        let (cp, n) = code_point_at(s, i);
        if cp > 0x9f {
          i += n;
        } else {
          break;
        }
      } else if c == b'\\' {
        match css_escape(s, i) {
          Some(n) => i += n,
          None => break,
        }
      } else {
        break;
      }
    }
    i - start
  }
  // Alternative 1: `[.#]?` (greedy, may backtrack to empty).
  let first = *s.first()?;
  if first == b'.' || first == b'#' {
    let n = run(s, 1);
    if n > 0 {
      return Some(1 + n);
    }
    // backtrack: `[.#]` empty → run from 0 starts with '.', fails.
    return None;
  }
  // Alternative `:*`
  let mut i = 0;
  while i < s.len() && s[i] == b':' {
    i += 1;
  }
  let n = run(s, i);
  if n > 0 {
    return Some(i + n);
  }
  None
}

/// `/^\([^&()@]+\)/`
pub fn simple_paren(s: &[u8]) -> Option<usize> {
  if s.first() != Some(&b'(') {
    return None;
  }
  let mut i = 1;
  while i < s.len() && !matches!(s[i], b'&' | b'(' | b')' | b'@') {
    i += 1;
  }
  if i > 1 && s.get(i) == Some(&b')') {
    Some(i + 1)
  } else {
    None
  }
}

/// `/^[.#:](?=@)/`
pub fn sigil_before_var(s: &[u8]) -> Option<usize> {
  if matches!(s.first(), Some(b'.' | b'#' | b':')) && s.get(1) == Some(&b'@') {
    Some(1)
  } else {
    None
  }
}

/// combinator: `/^\/[a-z]+\//i`
pub fn slashed_combinator(s: &[u8]) -> Option<usize> {
  if s.first() != Some(&b'/') {
    return None;
  }
  let mut i = 1;
  while i < s.len() && s[i].is_ascii_alphabetic() {
    i += 1;
  }
  if i > 1 && s.get(i) == Some(&b'/') {
    Some(i + 1)
  } else {
    None
  }
}

/// attribute key: `/^(?:[_A-Za-z0-9-*]*\|)?(?:[_A-Za-z0-9-]|\\.)+/`
pub fn attribute_key(s: &[u8]) -> Option<usize> {
  let ns_char = |b: u8| is_word_dash(b) || b == b'*';
  let main = |s: &[u8], mut i: usize| -> usize {
    let start = i;
    while i < s.len() {
      if is_word_dash(s[i]) {
        i += 1;
      } else if s[i] == b'\\' && i + 1 < s.len() && s[i + 1] != b'\n' && s[i + 1] != b'\r' {
        i += 1 + utf8_len(s[i + 1]);
      } else {
        break;
      }
    }
    i - start
  };
  // Optional namespace prefix.
  let mut j = 0;
  while j < s.len() && ns_char(s[j]) {
    j += 1;
  }
  if s.get(j) == Some(&b'|') {
    let n = main(s, j + 1);
    if n > 0 {
      return Some(j + 1 + n);
    }
  }
  let n = main(s, 0);
  if n > 0 { Some(n) } else { None }
}

/// attribute operator: `/^[|~*$^]?=/`
pub fn attribute_op(s: &[u8]) -> Option<usize> {
  match s.first() {
    Some(b'=') => Some(1),
    Some(b'|' | b'~' | b'*' | b'$' | b'^') if s.get(1) == Some(&b'=') => Some(2),
    _ => None,
  }
}

/// `/^[0-9]+%/`
pub fn int_percent(s: &[u8]) -> Option<usize> {
  let n = digits(s)?;
  if s.get(n) == Some(&b'%') {
    Some(n + 1)
  } else {
    None
  }
}

/// `/^[\w-]+/`
pub fn word_dash(s: &[u8]) -> Option<usize> {
  let n = s.iter().take_while(|&&b| is_word_dash(b)).count();
  if n == 0 { None } else { Some(n) }
}

/// anonymousValue: `/^([^.#@$+/'"*`(;{}-]*);/` → length of the value (without `;`)
pub fn anonymous_value(s: &[u8]) -> Option<usize> {
  for (i, &b) in s.iter().enumerate() {
    match b {
      b';' => return Some(i),
      b'.' | b'#' | b'@' | b'$' | b'+' | b'/' | b'\'' | b'"' | b'*' | b'`' | b'(' | b'{' | b'}'
      | b'-' => return None,
      _ => {}
    }
  }
  None
}

/// `/^@import\s+/`
pub fn at_import(s: &[u8]) -> Option<usize> {
  at_keyword_ws(s, b"@import")
}

/// `/^@plugin\s+/`
pub fn at_plugin(s: &[u8]) -> Option<usize> {
  at_keyword_ws(s, b"@plugin")
}

fn at_keyword_ws(s: &[u8], kw: &[u8]) -> Option<usize> {
  if !s.starts_with(kw) {
    return None;
  }
  let n = s[kw.len()..].iter().take_while(|&&b| is_space(b)).count();
  if n == 0 { None } else { Some(kw.len() + n) }
}

/// importOption: `/^(less|css|multiple|once|inline|reference|optional)/`
pub fn import_option(s: &[u8]) -> Option<usize> {
  for kw in [
    "less",
    "css",
    "multiple",
    "once",
    "inline",
    "reference",
    "optional",
  ] {
    if s.starts_with(kw.as_bytes()) {
      return Some(kw.len());
    }
  }
  None
}

/// The CSS keyword of `mediaFeature`:
/// `/^(?:--|-?(?:[_a-zA-Z0-9]|[^\0-\x7F]|\\[0-9a-fA-F]{1,6}\s?|\\[^\n\r\f0-9a-fA-F]))(?:[-_a-zA-Z0-9]|[^\0-\x7F]|\\[0-9a-fA-F]{1,6}\s?|\\[^\n\r\f0-9a-fA-F])*/`
pub fn css_keyword(s: &[u8]) -> Option<usize> {
  fn unit(s: &[u8], i: usize, allow_dash: bool) -> Option<usize> {
    let c = *s.get(i)?;
    if c == b'_' || c.is_ascii_alphanumeric() || (allow_dash && c == b'-') {
      return Some(1);
    }
    if c >= 0x80 {
      return Some(utf8_len(c));
    }
    if c == b'\\' {
      let d = *s.get(i + 1)?;
      if is_hex(d) {
        let mut j = i + 1;
        while j < s.len() && j - (i + 1) < 6 && is_hex(s[j]) {
          j += 1;
        }
        if j < s.len() && is_space(s[j]) {
          j += 1;
        }
        return Some(j - i);
      }
      if d != b'\n' && d != b'\r' && d != 0x0c {
        return Some(1 + utf8_len(d));
      }
    }
    None
  }
  let mut i;
  if s.starts_with(b"--") {
    i = 2;
  } else {
    i = 0;
    if s.first() == Some(&b'-') {
      i = 1;
    }
    match unit(s, i, false) {
      Some(n) => i += n,
      None => {
        return None;
      }
    }
  }
  while let Some(n) = unit(s, i, true) {
    i += n;
  }
  Some(i)
}

/// `css_keyword` followed by `\s+\(`.
pub fn css_keyword_space_paren(s: &[u8]) -> bool {
  if let Some(n) = css_keyword(s) {
    let ws = s[n..].iter().take_while(|&&b| is_space(b)).count();
    ws > 0 && s.get(n + ws) == Some(&b'(')
  } else {
    false
  }
}

/// `/^(?:[^()]|\([^()]*\))*\s*([<>]=|<=|>=|[<>]|=)/` (test only)
pub fn range_query_ahead(s: &[u8]) -> bool {
  // The leading group is greedy over everything except parens (and
  // balanced simple parens), so the first comparison operator found while
  // scanning decides; backtracking can only move to an earlier operator.
  let mut i = 0;
  while i < s.len() {
    match s[i] {
      b'<' | b'>' | b'=' => return true,
      b'(' => {
        let mut j = i + 1;
        while j < s.len() && s[j] != b'(' && s[j] != b')' {
          j += 1;
        }
        if s.get(j) == Some(&b')') {
          // `\([^()]*\)` is consumed as a whole: operators inside
          // the parens do not count.
          i = j + 1;
        } else {
          return false;
        }
      }
      b')' => return false,
      _ => i += 1,
    }
  }
  false
}

/// pluginArgs: `/^\s*([^);]+)\)\s*/` → (length, args range)
pub fn plugin_args(s: &[u8]) -> Option<(usize, usize, usize)> {
  let mut i = s.iter().take_while(|&&b| is_space(b)).count();
  let start = i;
  while i < s.len() && s[i] != b')' && s[i] != b';' {
    i += 1;
  }
  if i == start || s.get(i) != Some(&b')') {
    return None;
  }
  let end = i;
  i += 1;
  i += s[i..].iter().take_while(|&&b| is_space(b)).count();
  Some((i, start, end))
}

/// at-rule name: `/^@[a-z-]+/`
pub fn at_rule_name(s: &[u8]) -> Option<usize> {
  if s.first() != Some(&b'@') {
    return None;
  }
  let n = s[1..]
    .iter()
    .take_while(|&&b| b.is_ascii_lowercase() || b == b'-')
    .count();
  if n == 0 { None } else { Some(1 + n) }
}

/// `/^! *important/`
pub fn important(s: &[u8]) -> Option<usize> {
  if s.first() != Some(&b'!') {
    return None;
  }
  let mut i = 1;
  while s.get(i) == Some(&b' ') {
    i += 1;
  }
  if s[i..].starts_with(b"important") {
    Some(i + 9)
  } else {
    None
  }
}

/// colorOperand: `/^([lchrgbs])(?=\s|[,/*)]|$)/`
pub fn color_operand(s: &[u8]) -> Option<usize> {
  let c = *s.first()?;
  if !matches!(c, b'l' | b'c' | b'h' | b'r' | b'g' | b'b' | b's') {
    return None;
  }
  match s.get(1) {
    None => Some(1),
    Some(&b) if is_space(b) || matches!(b, b',' | b'/' | b'*' | b')') => Some(1),
    _ => None,
  }
}

/// addition operator: `/^[-+]\s+/`
pub fn spaced_add_op(s: &[u8]) -> Option<usize> {
  if !matches!(s.first(), Some(b'-' | b'+')) {
    return None;
  }
  let n = s[1..].iter().take_while(|&&b| is_space(b)).count();
  if n == 0 { None } else { Some(1 + n) }
}

/// conditions peek: `/^,\s*(not\s*)?\(/`
pub fn comma_condition_ahead(s: &[u8]) -> bool {
  if s.first() != Some(&b',') {
    return false;
  }
  let mut i = 1;
  while i < s.len() && is_space(s[i]) {
    i += 1;
  }
  if s.get(i) == Some(&b'(') {
    return true;
  }
  if s[i..].starts_with(b"not") {
    i += 3;
    while i < s.len() && is_space(s[i]) {
      i += 1;
    }
    return s.get(i) == Some(&b'(');
  }
  false
}

/// property: `/^(\*?-?[_a-zA-Z0-9-]+)\s*:/` → (length, name length)
pub fn property(s: &[u8]) -> Option<(usize, usize)> {
  let mut i = 0;
  if s.first() == Some(&b'*') {
    i = 1;
  }
  let start = i;
  while i < s.len() && is_word_dash(s[i]) {
    i += 1;
  }
  if i == start {
    return None;
  }
  let name = i;
  while i < s.len() && is_space(s[i]) {
    i += 1;
  }
  if s.get(i) == Some(&b':') {
    Some((i + 1, name))
  } else {
    None
  }
}

/// simple ruleProperty: `/^([_a-zA-Z0-9-]+)\s*:/`
pub fn simple_property(s: &[u8]) -> Option<(usize, usize)> {
  let n = s.iter().take_while(|&&b| is_word_dash(b)).count();
  if n == 0 {
    return None;
  }
  let mut i = n;
  while i < s.len() && is_space(s[i]) {
    i += 1;
  }
  if s.get(i) == Some(&b':') {
    Some((i + 1, n))
  } else {
    None
  }
}

/// `LOOKUP_KEY` chain: `(?:\[(?:[@$]{0,2})[_a-zA-Z0-9-]*\])*`
fn lookup_chain(s: &[u8], mut i: usize) -> usize {
  loop {
    if s.get(i) != Some(&b'[') {
      return i;
    }
    let n = lookup_value(&s[i + 1..]);
    if s.get(i + 1 + n) == Some(&b']') {
      i = i + 1 + n + 1;
    } else {
      return i;
    }
  }
}

/// RULE_PROPERTY_PARTICLE:
/// `^((?:[\w-]+)|(?:@\{[\w-]+LOOKUP_CHAIN\})|(?:\$\{[\w-]+\}))`
pub fn rule_property_particle(s: &[u8]) -> Option<usize> {
  if let Some(n) = word_dash(s) {
    return Some(n);
  }
  if s.starts_with(b"@{") {
    let n = s[2..].iter().take_while(|&&b| is_word_dash(b)).count();
    if n > 0 {
      let j = lookup_chain(s, 2 + n);
      if s.get(j) == Some(&b'}') {
        return Some(j + 1);
      }
    }
    return None;
  }
  if s.starts_with(b"${") {
    let n = s[2..].iter().take_while(|&&b| is_word_dash(b)).count();
    if n > 0 && s.get(2 + n) == Some(&b'}') {
      return Some(3 + n);
    }
  }
  None
}

/// `/^((?:\+_|\+)?)\s*:/` → (length, merge length)
pub fn merge_colon(s: &[u8]) -> Option<(usize, usize)> {
  let m = if s.starts_with(b"+_") {
    2
  } else if s.starts_with(b"+") {
    1
  } else {
    0
  };
  let mut i = m;
  while i < s.len() && is_space(s[i]) {
    i += 1;
  }
  if s.get(i) == Some(&b':') {
    Some((i + 1, m))
  } else {
    None
  }
}

/// `/^url\(/i`
pub fn url_start_ci(s: &[u8]) -> bool {
  s.len() >= 4 && s[..3].eq_ignore_ascii_case(b"url") && s[3] == b'('
}

/// `/^[^{]*\}/`
pub fn no_brace_before_close(s: &[u8]) -> bool {
  for &b in s {
    if b == b'{' {
      return false;
    }
    if b == b'}' {
      return true;
    }
  }
  false
}

/// `VARIABLE_INTERPOLATION` / `hasInterpolation` helpers: finds `@{name[..]}`
/// at `s[i..]`; returns (end, name range).
pub fn interpolation_at(s: &[u8], i: usize) -> Option<usize> {
  if !s[i..].starts_with(b"@{") {
    return None;
  }
  let n = s[i + 2..]
    .iter()
    .take_while(|&&b| is_word(b) || b == b'-')
    .count();
  if n == 0 {
    return None;
  }
  let j = lookup_chain(s, i + 2 + n);
  if s.get(j) == Some(&b'}') {
    Some(j + 1)
  } else {
    None
  }
}

/// `${name}` at `s[i..]`.
pub fn prop_interpolation_at(s: &[u8], i: usize) -> Option<usize> {
  if !s[i..].starts_with(b"${") {
    return None;
  }
  let n = s[i + 2..]
    .iter()
    .take_while(|&&b| is_word(b) || b == b'-')
    .count();
  if n > 0 && s.get(i + 2 + n) == Some(&b'}') {
    Some(i + 3 + n)
  } else {
    None
  }
}

/// `hasInterpolation(text)`
pub fn has_interpolation(text: &str) -> bool {
  let s = text.as_bytes();
  for i in 0..s.len() {
    if (s[i] == b'@' && interpolation_at(s, i).is_some())
      || (s[i] == b'$' && prop_interpolation_at(s, i).is_some())
    {
      return true;
    }
  }
  false
}

/// `@[\w-]+LOOKUP_CHAIN` (bare variable with lookups) at `s[i..]` (which is `@`).
pub fn bare_variable_with_lookups(s: &[u8], i: usize) -> Option<usize> {
  if s.get(i) != Some(&b'@') {
    return None;
  }
  let n = s[i + 1..]
    .iter()
    .take_while(|&&b| is_word(b) || b == b'-')
    .count();
  if n == 0 {
    return None;
  }
  Some(lookup_chain(s, i + 1 + n))
}

/// `splitLookups(raw)` → (name, lookups)
pub fn split_lookups(raw: &str) -> (&str, Option<Vec<&str>>) {
  match raw.find('[') {
    None => (raw, None),
    Some(open) => {
      let s = raw.as_bytes();
      let mut lookups = Vec::new();
      let mut i = open;
      while i < s.len() {
        if s[i] == b'[' {
          let n = lookup_value(&s[i + 1..]);
          if s.get(i + 1 + n) == Some(&b']') {
            lookups.push(&raw[i + 1..i + 1 + n]);
            i = i + 2 + n;
            continue;
          }
        }
        i += 1;
      }
      (&raw[..open], Some(lookups))
    }
  }
}
