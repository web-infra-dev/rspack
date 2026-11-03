use crate::{
  TokenKind, TokenWithTrivia,
  css_syntax::{MAX_CSS_KEYWORD_LEN, dashed_ident_name, decode_css_keyword},
  dependency_types::{
    CustomMediaDefinition, CustomMediaDefinitionValue, CustomMediaPart, CustomMediaText,
    CustomMediaUse, MediaQuery, MediaQueryInstruction, MediaQueryList, MediaSlot, Range,
  },
};

#[derive(Debug, Clone, Copy)]
struct ComponentSpan {
  start: usize,
  end: usize,
}

/// Walk balanced top-level spans directly over the lexer buffer. Nested CSS
/// remains opaque unless a parenthesized span contains a custom-media name.
struct BalancedSpans<'tokens> {
  tokens: &'tokens [TokenWithTrivia],
  index: usize,
  end: usize,
}

impl<'tokens> BalancedSpans<'tokens> {
  fn new(tokens: &'tokens [TokenWithTrivia], start: usize, end: usize) -> Self {
    Self {
      tokens,
      index: start,
      end,
    }
  }
}

impl Iterator for BalancedSpans<'_> {
  type Item = ComponentSpan;

  fn next(&mut self) -> Option<Self::Item> {
    if self.index >= self.end {
      return None;
    }
    let start = self.index;
    let end = component_end(self.tokens, start, self.end);
    self.index = end;
    Some(ComponentSpan { start, end })
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
  let first = BalancedSpans::new(tokens, 0, tokens.len()).next()?;
  let token = tokens[first.start].token;
  if token.kind != TokenKind::Ident {
    return None;
  }
  let name = token_value(input, token.value_range);
  if !is_dashed_ident(name) {
    return None;
  }
  Some(CustomMediaDefinition {
    name,
    value: classify_value(input, tokens, first.end, tokens.len()),
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
  for component in BalancedSpans::new(tokens, 0, tokens.len()) {
    if tokens[component.start].token.kind != TokenKind::Comma {
      continue;
    }
    let query = parse_query(input, tokens, query_start, component.start);
    references += query.references;
    queries.push(query.finish());
    query_start = component.end;
  }
  let query = parse_query(input, tokens, query_start, tokens.len());
  references += query.references;
  queries.push(query.finish());
  if references == 0 {
    return None;
  }

  Some(MediaQueryList {
    queries,
    range: source_range(tokens, 0, tokens.len()),
  })
}

fn classify_value<'s>(
  input: &'s str,
  tokens: &[TokenWithTrivia],
  start: usize,
  end: usize,
) -> CustomMediaDefinitionValue<'s> {
  if start >= end {
    return CustomMediaDefinitionValue::Unsupported;
  }
  if BalancedSpans::new(tokens, start, end)
    .any(|component| tokens[component.start].token.kind == TokenKind::Comma)
  {
    return classify_or_value(input, tokens, start, end);
  }

  let first = BalancedSpans::new(tokens, start, end)
    .next()
    .expect("non-empty custom media value");
  let single = first.end == end;
  if single
    && let Some((name, invalid)) = lone_dashed_ident(input, tokens, first)
    && !invalid
  {
    return CustomMediaDefinitionValue::Alias(name);
  }

  let query = parse_query(input, tokens, start, end);
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

  let range = source_range(tokens, start, end);
  if tokens[first.start].token.kind == TokenKind::LeftParenthesis {
    return CustomMediaDefinitionValue::Condition(CustomMediaText { range, wrap: false });
  }
  if tokens[first.start].token.kind != TokenKind::Ident {
    return CustomMediaDefinitionValue::Unsupported;
  }
  if single && is_keyword(input, tokens, first.start, "true") {
    return CustomMediaDefinitionValue::Boolean(true);
  }
  if single && is_keyword(input, tokens, first.start, "false") {
    return CustomMediaDefinitionValue::Boolean(false);
  }
  if is_keyword(input, tokens, first.start, "not") {
    let next = BalancedSpans::new(tokens, first.end, end).next();
    if next.is_some_and(|next| tokens[next.start].token.kind == TokenKind::LeftParenthesis) {
      return CustomMediaDefinitionValue::Condition(CustomMediaText { range, wrap: true });
    }
  }
  CustomMediaDefinitionValue::Type(range)
}

fn classify_or_value<'s>(
  input: &'s str,
  tokens: &[TokenWithTrivia],
  start: usize,
  end: usize,
) -> CustomMediaDefinitionValue<'s> {
  let mut parts = Vec::new();
  let mut segment_start = start;
  for component in BalancedSpans::new(tokens, start, end) {
    if tokens[component.start].token.kind != TokenKind::Comma {
      continue;
    }
    let Some(part) = classify_or_part(input, tokens, segment_start, component.start) else {
      return CustomMediaDefinitionValue::Unsupported;
    };
    parts.push(part);
    segment_start = component.end;
  }
  let Some(part) = classify_or_part(input, tokens, segment_start, end) else {
    return CustomMediaDefinitionValue::Unsupported;
  };
  parts.push(part);
  CustomMediaDefinitionValue::Or(parts)
}

fn classify_or_part<'s>(
  input: &'s str,
  tokens: &[TokenWithTrivia],
  start: usize,
  end: usize,
) -> Option<CustomMediaPart<'s>> {
  let first = BalancedSpans::new(tokens, start, end).next()?;
  let range = source_range(tokens, start, end);
  if first.end == end && tokens[first.start].token.kind == TokenKind::LeftParenthesis {
    if let Some((name, invalid)) = lone_dashed_ident(input, tokens, first) {
      return (!invalid).then_some(CustomMediaPart::Alias(name));
    }
    return Some(CustomMediaPart::Text(CustomMediaText {
      range,
      wrap: false,
    }));
  }

  if tokens[first.start].token.kind != TokenKind::LeftParenthesis
    && !is_keyword(input, tokens, first.start, "not")
  {
    return None;
  }

  let mut query = MediaProgramBuilder::new();
  parse_condition(input, tokens, start, end, true, &mut query);
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

fn parse_query<'s>(
  input: &'s str,
  tokens: &[TokenWithTrivia],
  start: usize,
  end: usize,
) -> MediaProgramBuilder<'s> {
  let mut query = MediaProgramBuilder::new();
  if start >= end {
    query.opaque(source_range(tokens, start, end));
    return query;
  }

  let first = BalancedSpans::new(tokens, start, end)
    .next()
    .expect("non-empty media query");
  let rest = first.end;
  let starts_with_type = tokens[first.start].token.kind == TokenKind::Ident
    && (!is_keyword(input, tokens, first.start, "not")
      || rest >= end
      || tokens[rest].token.kind != TokenKind::LeftParenthesis);
  if !starts_with_type {
    parse_condition(input, tokens, start, end, true, &mut query);
    return query;
  }

  let split = BalancedSpans::new(tokens, start, end)
    .find(|component| is_keyword(input, tokens, component.start, "and"));
  let Some(split) = split else {
    query.opaque(source_range(tokens, start, end));
    return query;
  };
  query.opaque(source_range(tokens, start, split.start));
  parse_condition(input, tokens, split.end, end, false, &mut query);
  query.instructions.push(MediaQueryInstruction::And(2));
  query
}

fn parse_condition<'s>(
  input: &'s str,
  tokens: &[TokenWithTrivia],
  start: usize,
  end: usize,
  alone: bool,
  query: &mut MediaProgramBuilder<'s>,
) {
  if start >= end {
    query.opaque(source_range(tokens, start, end));
    return;
  }

  let first = BalancedSpans::new(tokens, start, end)
    .next()
    .expect("non-empty media condition");
  if is_keyword(input, tokens, first.start, "not") {
    parse_media_in_parens(input, tokens, first.end, end, false, MediaSlot::Or, query);
    query.instructions.push(MediaQueryInstruction::Not);
    return;
  }

  let mut is_or = false;
  let mut term_count = 1usize;
  for component in BalancedSpans::new(tokens, start, end) {
    if is_keyword(input, tokens, component.start, "or") {
      is_or = true;
      term_count += 1;
    } else if is_keyword(input, tokens, component.start, "and") {
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

  let mut term_start = start;
  let mut leading = true;
  for component in BalancedSpans::new(tokens, start, end) {
    if !is_keyword(input, tokens, component.start, "and")
      && !is_keyword(input, tokens, component.start, "or")
    {
      continue;
    }
    parse_media_in_parens(
      input,
      tokens,
      term_start,
      component.start,
      leading,
      slot,
      query,
    );
    leading = false;
    term_start = component.end;
  }
  parse_media_in_parens(input, tokens, term_start, end, leading, slot, query);
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
  start: usize,
  end: usize,
  leading: bool,
  slot: MediaSlot,
  query: &mut MediaProgramBuilder<'s>,
) {
  let Some(component) = BalancedSpans::new(tokens, start, end).next() else {
    query.opaque(source_range(tokens, start, end));
    return;
  };
  if component.end != end || tokens[component.start].token.kind != TokenKind::LeftParenthesis {
    query.opaque(source_range(tokens, start, end));
    return;
  }
  if let Some((name, invalid)) = lone_dashed_ident(input, tokens, component) {
    query.reference(CustomMediaUse {
      name,
      range: source_range(tokens, component.start, component.end),
      invalid,
      leading,
      slot,
    });
    return;
  }

  let range = source_range(tokens, start, end);
  let Some((child_start, child_end)) = parenthesized_children(tokens, component) else {
    query.opaque(range);
    return;
  };
  let checkpoint = query.checkpoint();
  parse_condition(input, tokens, child_start, child_end, true, query);
  if query.references == checkpoint.1 {
    query.rollback(checkpoint);
    query.opaque(range);
  } else {
    query.instructions.push(MediaQueryInstruction::Group);
  }
}

fn lone_dashed_ident<'s>(
  input: &'s str,
  tokens: &[TokenWithTrivia],
  component: ComponentSpan,
) -> Option<(&'s str, bool)> {
  let (start, end) = parenthesized_children(tokens, component)?;
  let first = BalancedSpans::new(tokens, start, end).next()?;
  let token = tokens[first.start].token;
  if token.kind != TokenKind::Ident {
    return None;
  }
  let name = token_value(input, token.value_range);
  if !is_dashed_ident(name) {
    return None;
  }
  Some((name, first.end != end))
}

fn parenthesized_children(
  tokens: &[TokenWithTrivia],
  component: ComponentSpan,
) -> Option<(usize, usize)> {
  if tokens[component.start].token.kind != TokenKind::LeftParenthesis {
    return None;
  }
  let start = component.start + 1;
  let end = if component.end > start
    && tokens[component.end - 1].token.kind == TokenKind::RightParenthesis
  {
    component.end - 1
  } else {
    component.end
  };
  Some((start, end))
}

fn component_end(tokens: &[TokenWithTrivia], start: usize, end: usize) -> usize {
  let closing = match tokens[start].token.kind {
    TokenKind::Function | TokenKind::LeftParenthesis => TokenKind::RightParenthesis,
    TokenKind::LeftSquareBracket => TokenKind::RightSquareBracket,
    TokenKind::LeftCurlyBracket => TokenKind::RightCurlyBracket,
    _ => return start + 1,
  };
  let mut index = start + 1;
  while index < end {
    if tokens[index].token.kind == closing {
      return index + 1;
    }
    index = component_end(tokens, index, end);
  }
  end
}

fn source_range(tokens: &[TokenWithTrivia], start: usize, end: usize) -> Range {
  if start < end {
    return Range::new(
      tokens[start].token.range.start,
      tokens[end - 1].token.range.end,
    );
  }
  let position = tokens
    .get(start)
    .map(|item| item.token.range.start)
    .or_else(|| tokens.last().map(|item| item.token.range.end))
    .unwrap_or(0);
  Range::new(position, position)
}

fn is_keyword(input: &str, tokens: &[TokenWithTrivia], index: usize, keyword: &str) -> bool {
  let token = tokens[index].token;
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
