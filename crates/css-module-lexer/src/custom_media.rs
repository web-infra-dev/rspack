use crate::{
  TokenKind, TokenWithTrivia,
  css_syntax::{MAX_CSS_KEYWORD_LEN, dashed_ident_name, decode_css_keyword},
  dependency_types::{
    CustomMediaDefinition, CustomMediaDefinitionValue, CustomMediaPart, CustomMediaUse, MediaNode,
    MediaQueryList, MediaSlot, Range,
  },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BlockKind {
  Parenthesis,
  Square,
  Curly,
}

// Transient component structure for one at-rule prelude. Only the semantic
// `MediaNode` dependency payload survives the dependency scan.
#[derive(Debug)]
struct Component {
  range: Range,
  kind: ComponentKind,
}

#[derive(Debug)]
enum ComponentKind {
  Whitespace,
  Ident(String),
  SimpleBlock(BlockKind, Vec<Component>),
  Function(Vec<Component>),
  Comma,
  Other,
}

pub(crate) fn parse_custom_media_definition(
  input: &str,
  tokens: &[TokenWithTrivia],
) -> Option<CustomMediaDefinition> {
  let components = parse_component_list(input, tokens);
  let (start, end) = trimmed_bounds(&components, 0, components.len());
  let ComponentKind::Ident(name) = &components.get(start)?.kind else {
    return None;
  };
  if !is_dashed_ident(name) {
    return None;
  }
  let (value_start, value_end) = trimmed_bounds(&components, start + 1, end);
  Some(CustomMediaDefinition {
    name: name.clone(),
    value: classify_value(input, &components, value_start, value_end),
  })
}

pub(crate) fn parse_media_query(input: &str, tokens: &[TokenWithTrivia]) -> Option<MediaQueryList> {
  let components = parse_component_list(input, tokens);
  let (start, end) = trimmed_bounds(&components, 0, components.len());
  if start >= end || !has_custom_media_ref(&components[start..end]) {
    return None;
  }
  let mut queries = Vec::new();
  let mut query_start = start;
  for index in start..=end {
    if index != end && !matches!(components[index].kind, ComponentKind::Comma) {
      continue;
    }
    queries.push(capture_media_query(input, &components, query_start, index));
    query_start = index + 1;
  }
  if !queries.iter().any(node_has_use) {
    return None;
  }
  Some(MediaQueryList {
    queries,
    range: Range::new(components[start].range.start, components[end - 1].range.end),
  })
}

fn parse_component_list(input: &str, tokens: &[TokenWithTrivia]) -> Vec<Component> {
  let mut index = 0;
  parse_components(input, tokens, &mut index, None)
}

fn parse_components(
  input: &str,
  tokens: &[TokenWithTrivia],
  index: &mut usize,
  closing: Option<TokenKind>,
) -> Vec<Component> {
  let mut components = Vec::new();
  while *index < tokens.len() {
    let item = tokens[*index];
    if item.leading.range.start < item.leading.range.end {
      components.push(Component {
        range: item.leading.range,
        kind: ComponentKind::Whitespace,
      });
    }
    let token = item.token;
    if closing == Some(token.kind) {
      break;
    }
    *index += 1;

    let kind = match token.kind {
      TokenKind::Ident => ComponentKind::Ident(token_value(input, token.value_range).to_string()),
      TokenKind::Comma => ComponentKind::Comma,
      TokenKind::Function => {
        let children = parse_components(input, tokens, index, Some(TokenKind::RightParenthesis));
        let end = consume_closing(tokens, index, TokenKind::RightParenthesis, token.range.end);
        components.push(Component {
          range: Range::new(token.range.start, end),
          kind: ComponentKind::Function(children),
        });
        continue;
      }
      TokenKind::LeftParenthesis | TokenKind::LeftSquareBracket | TokenKind::LeftCurlyBracket => {
        let (block_kind, close) = match token.kind {
          TokenKind::LeftParenthesis => (BlockKind::Parenthesis, TokenKind::RightParenthesis),
          TokenKind::LeftSquareBracket => (BlockKind::Square, TokenKind::RightSquareBracket),
          TokenKind::LeftCurlyBracket => (BlockKind::Curly, TokenKind::RightCurlyBracket),
          _ => unreachable!(),
        };
        let children = parse_components(input, tokens, index, Some(close));
        let end = consume_closing(tokens, index, close, token.range.end);
        components.push(Component {
          range: Range::new(token.range.start, end),
          kind: ComponentKind::SimpleBlock(block_kind, children),
        });
        continue;
      }
      _ => ComponentKind::Other,
    };
    components.push(Component {
      range: token.range,
      kind,
    });
  }
  components
}

fn consume_closing(
  tokens: &[TokenWithTrivia],
  index: &mut usize,
  closing: TokenKind,
  fallback: u32,
) -> u32 {
  if tokens
    .get(*index)
    .is_some_and(|item| item.token.kind == closing)
  {
    let end = tokens[*index].token.range.end;
    *index += 1;
    end
  } else {
    fallback
  }
}

fn classify_value(
  input: &str,
  components: &[Component],
  start: usize,
  end: usize,
) -> CustomMediaDefinitionValue {
  if start >= end {
    return CustomMediaDefinitionValue::Unsupported;
  }
  if components[start..end]
    .iter()
    .any(|component| matches!(component.kind, ComponentKind::Comma))
  {
    return classify_or_value(input, components, start, end);
  }

  let first = &components[start];
  let text = component_text(input, components, start, end).to_string();
  if end - start == 1
    && let Some((name, has_extra)) = lone_dashed_ident(first)
    && !has_extra
  {
    return CustomMediaDefinitionValue::Alias(name);
  }
  if has_custom_media_ref(&components[start..end]) {
    let node = capture_media_query(input, components, start, end);
    if node_has_invalid_use(&node) {
      return CustomMediaDefinitionValue::Unsupported;
    }
    return CustomMediaDefinitionValue::Query(node);
  }
  if matches!(
    first.kind,
    ComponentKind::SimpleBlock(BlockKind::Parenthesis, _)
  ) {
    return CustomMediaDefinitionValue::Condition(text);
  }
  let ComponentKind::Ident(_) = &first.kind else {
    return CustomMediaDefinitionValue::Unsupported;
  };
  if end - start == 1 && is_keyword(first, "true") {
    return CustomMediaDefinitionValue::Boolean(true);
  }
  if end - start == 1 && is_keyword(first, "false") {
    return CustomMediaDefinitionValue::Boolean(false);
  }
  if is_keyword(first, "not") {
    let (next, _) = trimmed_bounds(components, start + 1, end);
    if next < end
      && matches!(
        components[next].kind,
        ComponentKind::SimpleBlock(BlockKind::Parenthesis, _)
      )
    {
      return CustomMediaDefinitionValue::Condition(format!("({text})"));
    }
  }
  CustomMediaDefinitionValue::Type(text)
}

fn classify_or_value(
  input: &str,
  components: &[Component],
  start: usize,
  end: usize,
) -> CustomMediaDefinitionValue {
  let mut parts = Vec::new();
  let mut segment_start = start;
  for index in start..=end {
    if index != end && !matches!(components[index].kind, ComponentKind::Comma) {
      continue;
    }
    let (part_start, part_end) = trimmed_bounds(components, segment_start, index);
    if part_start >= part_end {
      return CustomMediaDefinitionValue::Unsupported;
    }
    let first = &components[part_start];
    let text = component_text(input, components, part_start, part_end).to_string();
    if part_end - part_start == 1
      && matches!(
        first.kind,
        ComponentKind::SimpleBlock(BlockKind::Parenthesis, _)
      )
    {
      if let Some((name, has_extra)) = lone_dashed_ident(first) {
        if has_extra {
          return CustomMediaDefinitionValue::Unsupported;
        }
        parts.push(CustomMediaPart::Alias(name));
      } else {
        parts.push(CustomMediaPart::Text(text));
      }
      segment_start = index + 1;
      continue;
    }
    let is_condition = matches!(
      first.kind,
      ComponentKind::SimpleBlock(BlockKind::Parenthesis, _)
    ) || is_keyword(first, "not");
    if !is_condition {
      return CustomMediaDefinitionValue::Unsupported;
    }
    if !has_custom_media_ref(&components[part_start..part_end]) {
      parts.push(CustomMediaPart::Text(format!("({text})")));
    } else {
      let node = capture_media_condition(input, components, part_start, part_end, true);
      if node_has_invalid_use(&node) {
        return CustomMediaDefinitionValue::Unsupported;
      }
      parts.push(CustomMediaPart::Query(node));
    }
    segment_start = index + 1;
  }
  CustomMediaDefinitionValue::Or(parts)
}

fn capture_media_query(
  input: &str,
  components: &[Component],
  start: usize,
  end: usize,
) -> MediaNode {
  let (start, end) = trimmed_bounds(components, start, end);
  if start >= end {
    return MediaNode::Text(String::new());
  }
  let (rest, _) = trimmed_bounds(components, start + 1, end);
  let starts_with_type = matches!(components[start].kind, ComponentKind::Ident(_))
    && (!is_keyword(&components[start], "not")
      || rest >= end
      || !matches!(
        components[rest].kind,
        ComponentKind::SimpleBlock(BlockKind::Parenthesis, _)
      ));
  if !starts_with_type {
    return capture_media_condition(input, components, start, end, true);
  }

  let split = (start..end)
    .find(|index| is_keyword(&components[*index], "and"))
    .unwrap_or(end);
  let (_, type_end) = trimmed_bounds(components, start, split);
  MediaNode::Typed {
    text: component_text(input, components, start, type_end).to_string(),
    rest: (split != end).then(|| {
      Box::new(capture_media_condition(
        input,
        components,
        split + 1,
        end,
        false,
      ))
    }),
  }
}

fn capture_media_condition(
  input: &str,
  components: &[Component],
  start: usize,
  end: usize,
  alone: bool,
) -> MediaNode {
  let (start, end) = trimmed_bounds(components, start, end);
  if start >= end {
    return MediaNode::Text(String::new());
  }
  if is_keyword(&components[start], "not") {
    return MediaNode::Not(Box::new(capture_media_in_parens(
      input,
      components,
      start + 1,
      end,
      false,
      MediaSlot::Or,
    )));
  }

  let mut is_or = false;
  let mut single = true;
  for component in &components[start..end] {
    if is_keyword(component, "or") {
      is_or = true;
      single = false;
    } else if is_keyword(component, "and") {
      single = false;
    }
  }
  let slot = if single && alone {
    MediaSlot::Alone
  } else if is_or {
    MediaSlot::Or
  } else {
    MediaSlot::And
  };

  let mut terms = Vec::new();
  let mut term_start = start;
  for index in start..=end {
    if index != end
      && !is_keyword(&components[index], "and")
      && !is_keyword(&components[index], "or")
    {
      continue;
    }
    terms.push(capture_media_in_parens(
      input,
      components,
      term_start,
      index,
      term_start == start,
      slot,
    ));
    term_start = index + 1;
  }
  if terms.len() == 1 {
    terms.pop().expect("one media term")
  } else {
    MediaNode::Chain { is_or, terms }
  }
}

fn capture_media_in_parens(
  input: &str,
  components: &[Component],
  start: usize,
  end: usize,
  leading: bool,
  slot: MediaSlot,
) -> MediaNode {
  let (start, end) = trimmed_bounds(components, start, end);
  if start >= end {
    return MediaNode::Text(String::new());
  }
  let text = component_text(input, components, start, end).to_string();
  if end - start != 1 {
    return MediaNode::Text(text);
  }
  let component = &components[start];
  let ComponentKind::SimpleBlock(BlockKind::Parenthesis, children) = &component.kind else {
    return MediaNode::Text(text);
  };
  if let Some((name, invalid)) = lone_dashed_ident(component) {
    return MediaNode::Ref {
      text,
      usage: CustomMediaUse {
        name,
        range: component.range,
        invalid,
        leading,
        slot,
      },
    };
  }
  let inner = capture_media_condition(input, children, 0, children.len(), true);
  if matches!(inner, MediaNode::Text(_)) {
    MediaNode::Text(text)
  } else {
    MediaNode::Group(Box::new(inner))
  }
}

fn node_has_use(node: &MediaNode) -> bool {
  match node {
    MediaNode::Ref { .. } => true,
    MediaNode::Group(operand) | MediaNode::Not(operand) => node_has_use(operand),
    MediaNode::Chain { terms, .. } => terms.iter().any(node_has_use),
    MediaNode::Typed {
      rest: Some(rest), ..
    } => node_has_use(rest),
    MediaNode::Text(_) | MediaNode::Typed { rest: None, .. } => false,
  }
}

fn node_has_invalid_use(node: &MediaNode) -> bool {
  match node {
    MediaNode::Ref { usage, .. } => usage.invalid,
    MediaNode::Group(operand) | MediaNode::Not(operand) => node_has_invalid_use(operand),
    MediaNode::Chain { terms, .. } => terms.iter().any(node_has_invalid_use),
    MediaNode::Typed {
      rest: Some(rest), ..
    } => node_has_invalid_use(rest),
    MediaNode::Text(_) | MediaNode::Typed { rest: None, .. } => false,
  }
}

fn has_custom_media_ref(components: &[Component]) -> bool {
  components.iter().any(has_custom_media_ref_in)
}

fn has_custom_media_ref_in(component: &Component) -> bool {
  match &component.kind {
    ComponentKind::SimpleBlock(BlockKind::Parenthesis, children) => {
      lone_dashed_ident(component).is_some() || children.iter().any(has_custom_media_ref_in)
    }
    ComponentKind::SimpleBlock(_, children) | ComponentKind::Function(children) => {
      children.iter().any(has_custom_media_ref_in)
    }
    _ => false,
  }
}

fn lone_dashed_ident(component: &Component) -> Option<(String, bool)> {
  let ComponentKind::SimpleBlock(BlockKind::Parenthesis, children) = &component.kind else {
    return None;
  };
  let mut name = None;
  for child in children {
    if matches!(child.kind, ComponentKind::Whitespace) {
      continue;
    }
    if name.is_some() {
      return name.map(|name| (name, true));
    }
    let ComponentKind::Ident(value) = &child.kind else {
      return None;
    };
    if !is_dashed_ident(value) {
      return None;
    }
    name = Some(value.clone());
  }
  name.map(|name| (name, false))
}

fn trimmed_bounds(components: &[Component], mut start: usize, mut end: usize) -> (usize, usize) {
  while start < end && matches!(components[start].kind, ComponentKind::Whitespace) {
    start += 1;
  }
  while end > start && matches!(components[end - 1].kind, ComponentKind::Whitespace) {
    end -= 1;
  }
  (start, end)
}

fn component_text<'a>(
  input: &'a str,
  components: &[Component],
  start: usize,
  end: usize,
) -> &'a str {
  if start >= end {
    return "";
  }
  &input[components[start].range.start as usize..components[end - 1].range.end as usize]
}

fn is_keyword(component: &Component, keyword: &str) -> bool {
  let ComponentKind::Ident(value) = &component.kind else {
    return false;
  };
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
