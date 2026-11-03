use crate::{
  Token, TokenKind, TokenWithTrivia,
  css_syntax::{MAX_CSS_KEYWORD_LEN, dashed_ident_name, decode_css_keyword},
  dependency_types::{
    CustomMediaDefinition, CustomMediaDefinitionValue, CustomMediaPart, CustomMediaText,
    CustomMediaUse, MediaQuery, MediaQueryInstruction, MediaQueryList, MediaSlot, Range,
  },
};

/// Walk balanced top-level token slices directly over the lexer buffer. Nested
/// CSS remains opaque unless a parenthesized slice contains a custom-media name.
struct BalancedSpans<'tokens> {
  remaining: &'tokens [TokenWithTrivia],
}

impl<'tokens> BalancedSpans<'tokens> {
  fn new(tokens: &'tokens [TokenWithTrivia]) -> Self {
    Self { remaining: tokens }
  }
}

impl<'tokens> Iterator for BalancedSpans<'tokens> {
  type Item = &'tokens [TokenWithTrivia];

  fn next(&mut self) -> Option<Self::Item> {
    if self.remaining.is_empty() {
      return None;
    }
    let len = component_len(self.remaining);
    let (span, remaining) = self.remaining.split_at(len);
    self.remaining = remaining;
    Some(span)
  }
}

struct MediaProgramBuilder<'s> {
  instructions: Vec<MediaQueryInstruction<'s>>,
  references: usize,
}

impl<'s> MediaProgramBuilder<'s> {
  fn new() -> Self {
    Self {
      instructions: Vec::new(),
      references: 0,
    }
  }

  fn opaque(&mut self, range: Range) {
    self.instructions.push(MediaQueryInstruction::Opaque(range));
  }

  fn reference(&mut self, usage: CustomMediaUse<'s>) {
    self.references += 1;
    self
      .instructions
      .push(MediaQueryInstruction::Reference(usage));
  }

  fn checkpoint(&self) -> (usize, usize) {
    (self.instructions.len(), self.references)
  }

  fn rollback(&mut self, checkpoint: (usize, usize)) {
    self.instructions.truncate(checkpoint.0);
    self.references = checkpoint.1;
  }

  fn finish(self) -> MediaQuery<'s> {
    MediaQuery {
      instructions: self.instructions,
    }
  }
}

pub(crate) fn parse_custom_media_definition<'s>(
  input: &'s str,
  tokens: &[TokenWithTrivia],
) -> Option<CustomMediaDefinition<'s>> {
  let first = BalancedSpans::new(tokens).next()?;
  let token = first[0].token;
  if token.kind != TokenKind::Ident {
    return None;
  }
  let name = token_value(input, token.value_range);
  if !is_dashed_ident(name) {
    return None;
  }
  Some(CustomMediaDefinition {
    name,
    value: classify_value(input, &tokens[first.len()..]),
  })
}

pub(crate) fn parse_media_query<'s>(
  input: &'s str,
  tokens: &[TokenWithTrivia],
) -> Option<MediaQueryList<'s>> {
  if tokens.is_empty() {
    return None;
  }

  let mut queries = Vec::new();
  let mut references = 0;
  let mut query_start = 0;
  let mut offset = 0;
  for span in BalancedSpans::new(tokens) {
    if span[0].token.kind == TokenKind::Comma {
      let query = parse_query(input, &tokens[query_start..offset]);
      references += query.references;
      queries.push(query.finish());
      query_start = offset + span.len();
    }
    offset += span.len();
  }
  let query = parse_query(input, &tokens[query_start..]);
  references += query.references;
  queries.push(query.finish());
  if references == 0 {
    return None;
  }

  Some(MediaQueryList {
    queries,
    range: source_range(tokens),
  })
}

fn classify_value<'s>(
  input: &'s str,
  tokens: &[TokenWithTrivia],
) -> CustomMediaDefinitionValue<'s> {
  if tokens.is_empty() {
    return CustomMediaDefinitionValue::Unsupported;
  }
  if BalancedSpans::new(tokens).any(|span| span[0].token.kind == TokenKind::Comma) {
    return classify_or_value(input, tokens);
  }

  let first = BalancedSpans::new(tokens)
    .next()
    .expect("non-empty custom media value");
  let single = first.len() == tokens.len();
  if single
    && let Some((name, invalid)) = lone_dashed_ident(input, first)
    && !invalid
  {
    return CustomMediaDefinitionValue::Alias(name);
  }

  let query = parse_query(input, tokens);
  if query.references != 0 {
    if query.instructions.iter().any(|instruction| {
      matches!(
        instruction,
        MediaQueryInstruction::Reference(CustomMediaUse { invalid: true, .. })
      )
    }) {
      return CustomMediaDefinitionValue::Unsupported;
    }
    return CustomMediaDefinitionValue::Query(query.finish());
  }

  let range = source_range(tokens);
  if first[0].token.kind == TokenKind::LeftParenthesis {
    return CustomMediaDefinitionValue::Condition(CustomMediaText { range, wrap: false });
  }
  if first[0].token.kind != TokenKind::Ident {
    return CustomMediaDefinitionValue::Unsupported;
  }
  if single && is_keyword(input, first[0].token, "true") {
    return CustomMediaDefinitionValue::Boolean(true);
  }
  if single && is_keyword(input, first[0].token, "false") {
    return CustomMediaDefinitionValue::Boolean(false);
  }
  if is_keyword(input, first[0].token, "not")
    && BalancedSpans::new(&tokens[first.len()..])
      .next()
      .is_some_and(|next| next[0].token.kind == TokenKind::LeftParenthesis)
  {
    return CustomMediaDefinitionValue::Condition(CustomMediaText { range, wrap: true });
  }
  CustomMediaDefinitionValue::Type(range)
}

fn classify_or_value<'s>(
  input: &'s str,
  tokens: &[TokenWithTrivia],
) -> CustomMediaDefinitionValue<'s> {
  let mut parts = Vec::new();
  let mut segment_start = 0;
  let mut offset = 0;
  for span in BalancedSpans::new(tokens) {
    if span[0].token.kind == TokenKind::Comma {
      let Some(part) = classify_or_part(input, &tokens[segment_start..offset]) else {
        return CustomMediaDefinitionValue::Unsupported;
      };
      parts.push(part);
      segment_start = offset + span.len();
    }
    offset += span.len();
  }
  let Some(part) = classify_or_part(input, &tokens[segment_start..]) else {
    return CustomMediaDefinitionValue::Unsupported;
  };
  parts.push(part);
  CustomMediaDefinitionValue::Or(parts)
}

fn classify_or_part<'s>(input: &'s str, tokens: &[TokenWithTrivia]) -> Option<CustomMediaPart<'s>> {
  let first = BalancedSpans::new(tokens).next()?;
  let range = source_range(tokens);
  if first.len() == tokens.len() && first[0].token.kind == TokenKind::LeftParenthesis {
    if let Some((name, invalid)) = lone_dashed_ident(input, first) {
      return (!invalid).then_some(CustomMediaPart::Alias(name));
    }
    return Some(CustomMediaPart::Text(CustomMediaText {
      range,
      wrap: false,
    }));
  }

  if first[0].token.kind != TokenKind::LeftParenthesis && !is_keyword(input, first[0].token, "not")
  {
    return None;
  }

  let mut query = MediaProgramBuilder::new();
  parse_condition(input, tokens, true, &mut query);
  if query.references == 0 {
    return Some(CustomMediaPart::Text(CustomMediaText { range, wrap: true }));
  }
  if query.instructions.iter().any(|instruction| {
    matches!(
      instruction,
      MediaQueryInstruction::Reference(CustomMediaUse { invalid: true, .. })
    )
  }) {
    return None;
  }
  Some(CustomMediaPart::Query(query.finish()))
}

fn parse_query<'s>(input: &'s str, tokens: &[TokenWithTrivia]) -> MediaProgramBuilder<'s> {
  let mut query = MediaProgramBuilder::new();
  if tokens.is_empty() {
    query.opaque(source_range(tokens));
    return query;
  }

  let first = BalancedSpans::new(tokens)
    .next()
    .expect("non-empty media query");
  let rest = &tokens[first.len()..];
  let starts_with_type = first[0].token.kind == TokenKind::Ident
    && (!is_keyword(input, first[0].token, "not")
      || rest.is_empty()
      || rest[0].token.kind != TokenKind::LeftParenthesis);
  if !starts_with_type {
    parse_condition(input, tokens, true, &mut query);
    return query;
  }

  let mut offset = 0;
  let mut split = None;
  for span in BalancedSpans::new(tokens) {
    if is_keyword(input, span[0].token, "and") {
      split = Some((offset, span.len()));
      break;
    }
    offset += span.len();
  }
  let Some((split, separator_len)) = split else {
    query.opaque(source_range(tokens));
    return query;
  };
  query.opaque(source_range(&tokens[..split]));
  parse_condition(input, &tokens[split + separator_len..], false, &mut query);
  query.instructions.push(MediaQueryInstruction::And(2));
  query
}

fn parse_condition<'s>(
  input: &'s str,
  tokens: &[TokenWithTrivia],
  alone: bool,
  query: &mut MediaProgramBuilder<'s>,
) {
  if tokens.is_empty() {
    query.opaque(source_range(tokens));
    return;
  }

  let first = BalancedSpans::new(tokens)
    .next()
    .expect("non-empty media condition");
  if is_keyword(input, first[0].token, "not") {
    parse_media_in_parens(input, &tokens[first.len()..], false, MediaSlot::Or, query);
    query.instructions.push(MediaQueryInstruction::Not);
    return;
  }

  let mut is_or = false;
  let mut term_count = 1usize;
  for span in BalancedSpans::new(tokens) {
    if is_keyword(input, span[0].token, "or") {
      is_or = true;
      term_count += 1;
    } else if is_keyword(input, span[0].token, "and") {
      term_count += 1;
    }
  }
  let slot = if term_count == 1 && alone {
    MediaSlot::Alone
  } else if is_or {
    MediaSlot::Or
  } else {
    MediaSlot::And
  };

  let mut term_start = 0;
  let mut offset = 0;
  let mut leading = true;
  for span in BalancedSpans::new(tokens) {
    if is_keyword(input, span[0].token, "and") || is_keyword(input, span[0].token, "or") {
      parse_media_in_parens(input, &tokens[term_start..offset], leading, slot, query);
      leading = false;
      term_start = offset + span.len();
    }
    offset += span.len();
  }
  parse_media_in_parens(input, &tokens[term_start..], leading, slot, query);
  if term_count > 1 {
    let term_count = u32::try_from(term_count).expect("media condition has too many terms");
    query.instructions.push(if is_or {
      MediaQueryInstruction::Or(term_count)
    } else {
      MediaQueryInstruction::And(term_count)
    });
  }
}

fn parse_media_in_parens<'s>(
  input: &'s str,
  tokens: &[TokenWithTrivia],
  leading: bool,
  slot: MediaSlot,
  query: &mut MediaProgramBuilder<'s>,
) {
  let Some(span) = BalancedSpans::new(tokens).next() else {
    query.opaque(source_range(tokens));
    return;
  };
  if span.len() != tokens.len() || span[0].token.kind != TokenKind::LeftParenthesis {
    query.opaque(source_range(tokens));
    return;
  }
  if let Some((name, invalid)) = lone_dashed_ident(input, span) {
    query.reference(CustomMediaUse {
      name,
      range: source_range(span),
      invalid,
      leading,
      slot,
    });
    return;
  }

  let range = source_range(tokens);
  let Some(children) = parenthesized_children(span) else {
    query.opaque(range);
    return;
  };
  let checkpoint = query.checkpoint();
  parse_condition(input, children, true, query);
  if query.references == checkpoint.1 {
    query.rollback(checkpoint);
    query.opaque(range);
  } else {
    query.instructions.push(MediaQueryInstruction::Group);
  }
}

fn lone_dashed_ident<'s>(input: &'s str, span: &[TokenWithTrivia]) -> Option<(&'s str, bool)> {
  let children = parenthesized_children(span)?;
  let first = BalancedSpans::new(children).next()?;
  let token = first[0].token;
  if token.kind != TokenKind::Ident {
    return None;
  }
  let name = token_value(input, token.value_range);
  if !is_dashed_ident(name) {
    return None;
  }
  Some((name, first.len() != children.len()))
}

fn parenthesized_children(span: &[TokenWithTrivia]) -> Option<&[TokenWithTrivia]> {
  if span[0].token.kind != TokenKind::LeftParenthesis {
    return None;
  }
  let end = if span.len() > 1 && span.last()?.token.kind == TokenKind::RightParenthesis {
    span.len() - 1
  } else {
    span.len()
  };
  Some(&span[1..end])
}

fn component_len(tokens: &[TokenWithTrivia]) -> usize {
  let closing = match tokens[0].token.kind {
    TokenKind::Function | TokenKind::LeftParenthesis => TokenKind::RightParenthesis,
    TokenKind::LeftSquareBracket => TokenKind::RightSquareBracket,
    TokenKind::LeftCurlyBracket => TokenKind::RightCurlyBracket,
    _ => return 1,
  };
  let mut len = 1;
  while len < tokens.len() {
    if tokens[len].token.kind == closing {
      return len + 1;
    }
    len += component_len(&tokens[len..]);
  }
  len
}

fn source_range(tokens: &[TokenWithTrivia]) -> Range {
  match (tokens.first(), tokens.last()) {
    (Some(first), Some(last)) => Range::new(first.token.range.start, last.token.range.end),
    _ => Range::new(0, 0),
  }
}

fn is_keyword(input: &str, token: Token, keyword: &str) -> bool {
  if token.kind != TokenKind::Ident {
    return false;
  }
  let value = token_value(input, token.value_range);
  if value.eq_ignore_ascii_case(keyword) {
    return true;
  }
  let mut normalized = [0; MAX_CSS_KEYWORD_LEN];
  decode_css_keyword(value, &mut normalized) == Some(keyword)
}

fn is_dashed_ident(name: &str) -> bool {
  dashed_ident_name(name).is_some_and(|name| !name.is_empty())
}

fn token_value(input: &str, range: Range) -> &str {
  &input[range.start as usize..range.end as usize]
}
