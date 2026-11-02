use css_module_lexer::{Lexer, Range, Token, TokenKind};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::css_syntax::unescape_identifier;

#[derive(Debug)]
pub(super) struct CustomMediaReplacement {
  pub range: Range,
  pub content: String,
}

#[derive(Debug)]
pub(super) struct CustomMediaWarning {
  pub range: Range,
  pub message: String,
}

#[derive(Debug, Default)]
pub(super) struct CustomMediaTransform {
  pub replacements: Vec<CustomMediaReplacement>,
  pub warnings: Vec<CustomMediaWarning>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BlockKind {
  Parenthesis,
  Square,
  Curly,
}

#[derive(Debug, Clone)]
struct Component {
  range: Range,
  kind: ComponentKind,
}

#[derive(Debug, Clone)]
enum ComponentKind {
  Whitespace,
  Ident(String),
  SimpleBlock(BlockKind, Vec<Component>),
  Function(Vec<Component>),
  Comma,
  Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MediaSlot {
  Alone,
  And,
  Or,
}

#[derive(Debug, Clone)]
struct CustomMediaUse {
  name: String,
  range: Range,
  invalid: bool,
  leading: bool,
  slot: MediaSlot,
}

#[derive(Debug, Clone)]
enum MediaNode {
  Text(String),
  Ref {
    text: String,
    usage: CustomMediaUse,
  },
  Group(Box<MediaNode>),
  Not(Box<MediaNode>),
  Chain {
    is_or: bool,
    terms: Vec<MediaNode>,
  },
  Typed {
    text: String,
    rest: Option<Box<MediaNode>>,
  },
}

#[derive(Debug, Clone)]
struct CustomMediaQuery {
  node: MediaNode,
}

#[derive(Debug, Clone)]
enum CustomMediaPart {
  Text(String),
  Alias(String),
  Query(CustomMediaQuery),
}

#[derive(Debug, Clone)]
enum CustomMediaDefinition {
  Unsupported,
  Boolean(bool),
  Condition(String),
  Type(String),
  Alias(String),
  Or(Vec<CustomMediaPart>),
  Query(CustomMediaQuery),
}

#[derive(Debug, Clone)]
enum CustomMediaValue {
  Unsupported,
  Boolean(bool),
  Condition(String),
  Type(String),
}

#[derive(Debug)]
struct MediaPrelude {
  queries: Vec<MediaNode>,
  range: Range,
}

#[derive(Debug, Clone, Copy)]
enum AtRuleEnd {
  Semicolon(usize),
  Block(usize),
  Boundary(usize),
}

impl AtRuleEnd {
  fn token_index(self) -> usize {
    match self {
      Self::Semicolon(index) | Self::Block(index) | Self::Boundary(index) => index,
    }
  }
}

struct CustomMediaResolver<'a> {
  definitions: &'a FxHashMap<String, CustomMediaDefinition>,
  values: FxHashMap<String, CustomMediaValue>,
}

impl<'a> CustomMediaResolver<'a> {
  fn new(definitions: &'a FxHashMap<String, CustomMediaDefinition>) -> Self {
    Self {
      definitions,
      values: FxHashMap::default(),
    }
  }

  fn resolve_value(&mut self, name: &str, resolving: &mut FxHashSet<String>) -> CustomMediaValue {
    if let Some(value) = self.values.get(name) {
      return value.clone();
    }
    let Some(definition) = self.definitions.get(name).cloned() else {
      return CustomMediaValue::Unsupported;
    };
    if !resolving.insert(name.to_string()) {
      return CustomMediaValue::Unsupported;
    }

    let value = match definition {
      CustomMediaDefinition::Unsupported => CustomMediaValue::Unsupported,
      CustomMediaDefinition::Boolean(value) => CustomMediaValue::Boolean(value),
      CustomMediaDefinition::Condition(text) => CustomMediaValue::Condition(text),
      CustomMediaDefinition::Type(text) => CustomMediaValue::Type(text),
      CustomMediaDefinition::Alias(alias) => match self.resolve_value(&alias, resolving) {
        CustomMediaValue::Type(_) => CustomMediaValue::Unsupported,
        value => value,
      },
      CustomMediaDefinition::Or(parts) => self.resolve_or(parts, resolving),
      CustomMediaDefinition::Query(query) => self.resolve_query(&query.node, resolving),
    };

    resolving.remove(name);
    self.values.insert(name.to_string(), value.clone());
    value
  }

  fn resolve_or(
    &mut self,
    parts: Vec<CustomMediaPart>,
    resolving: &mut FxHashSet<String>,
  ) -> CustomMediaValue {
    let mut texts = Vec::new();
    let mut any_true = false;
    for part in parts {
      let value = match part {
        CustomMediaPart::Text(text) => {
          texts.push(text);
          continue;
        }
        CustomMediaPart::Alias(alias) => self.resolve_value(&alias, resolving),
        CustomMediaPart::Query(query) => self.resolve_query(&query.node, resolving),
      };
      match value {
        CustomMediaValue::Boolean(value) => any_true |= value,
        CustomMediaValue::Condition(text) => {
          texts.push(media_condition_for(&text, MediaSlot::Or));
        }
        CustomMediaValue::Unsupported | CustomMediaValue::Type(_) => {
          return CustomMediaValue::Unsupported;
        }
      }
    }

    if any_true {
      CustomMediaValue::Boolean(true)
    } else if texts.is_empty() {
      CustomMediaValue::Boolean(false)
    } else if texts.len() == 1 {
      CustomMediaValue::Condition(texts.pop().expect("one custom media condition"))
    } else {
      CustomMediaValue::Condition(format!("({})", texts.join(" or ")))
    }
  }

  fn resolve_query(
    &mut self,
    node: &MediaNode,
    resolving: &mut FxHashSet<String>,
  ) -> CustomMediaValue {
    let mut uses = Vec::new();
    collect_uses(node, &mut uses);
    for usage in uses {
      match self.resolve_value(&usage.name, resolving) {
        CustomMediaValue::Unsupported => return CustomMediaValue::Unsupported,
        CustomMediaValue::Type(_) if !usage.leading => {
          return CustomMediaValue::Unsupported;
        }
        _ => {}
      }
    }

    match self.fold_node(node, resolving) {
      FoldedMedia::Boolean(value) => CustomMediaValue::Boolean(value),
      FoldedMedia::Text(text) if is_condition_text(&text) => CustomMediaValue::Condition(text),
      FoldedMedia::Text(text) => CustomMediaValue::Type(text),
    }
  }

  fn fold_node(&mut self, node: &MediaNode, resolving: &mut FxHashSet<String>) -> FoldedMedia {
    match node {
      MediaNode::Text(text) => FoldedMedia::Text(text.clone()),
      MediaNode::Ref { text, usage } => {
        if usage.invalid {
          return FoldedMedia::Text(text.clone());
        }
        match self.resolve_value(&usage.name, resolving) {
          CustomMediaValue::Boolean(value) => FoldedMedia::Boolean(value),
          CustomMediaValue::Condition(text) => {
            FoldedMedia::Text(media_condition_for(&text, usage.slot))
          }
          CustomMediaValue::Type(text) if usage.leading => FoldedMedia::Text(text),
          CustomMediaValue::Unsupported | CustomMediaValue::Type(_) => {
            FoldedMedia::Text(text.clone())
          }
        }
      }
      MediaNode::Group(operand) => match self.fold_node(operand, resolving) {
        FoldedMedia::Boolean(value) => FoldedMedia::Boolean(value),
        FoldedMedia::Text(text) => FoldedMedia::Text(format!("({text})")),
      },
      MediaNode::Not(operand) => match self.fold_node(operand, resolving) {
        FoldedMedia::Boolean(value) => FoldedMedia::Boolean(!value),
        FoldedMedia::Text(text) => FoldedMedia::Text(format!("not {text}")),
      },
      MediaNode::Chain { is_or, terms } => {
        let mut kept = Vec::new();
        for term in terms {
          match self.fold_node(term, resolving) {
            FoldedMedia::Boolean(value) if value == *is_or => {
              return FoldedMedia::Boolean(*is_or);
            }
            FoldedMedia::Boolean(_) => {}
            FoldedMedia::Text(text) => kept.push(text),
          }
        }
        if kept.is_empty() {
          FoldedMedia::Boolean(!*is_or)
        } else if kept.len() == 1 {
          FoldedMedia::Text(kept.pop().expect("one media term"))
        } else {
          FoldedMedia::Text(kept.join(if *is_or { " or " } else { " and " }))
        }
      }
      MediaNode::Typed { text, rest } => {
        let Some(rest) = rest else {
          return FoldedMedia::Text(text.clone());
        };
        match self.fold_node(rest, resolving) {
          FoldedMedia::Boolean(false) => FoldedMedia::Boolean(false),
          FoldedMedia::Boolean(true) => FoldedMedia::Text(text.clone()),
          FoldedMedia::Text(rest) => FoldedMedia::Text(format!("{text} and {rest}")),
        }
      }
    }
  }
}

enum FoldedMedia {
  Boolean(bool),
  Text(String),
}

pub(super) fn transform_custom_media(input: &str) -> CustomMediaTransform {
  // Match webpack's cheap guard. The common path pays no tokenization cost.
  if !input.contains("@custom-media") {
    return CustomMediaTransform::default();
  }

  let tokens = tokenize(input);
  let mut definitions = FxHashMap::default();
  let mut media = Vec::new();
  let mut replacements = Vec::new();
  collect_at_rules(
    input,
    &tokens,
    &mut definitions,
    &mut media,
    &mut replacements,
  );

  let mut warnings = Vec::new();
  let mut resolver = CustomMediaResolver::new(&definitions);
  for prelude in media {
    let mut uses = Vec::new();
    for query in &prelude.queries {
      collect_uses(query, &mut uses);
    }

    let mut has_boolean = false;
    for usage in &uses {
      if usage.invalid {
        warnings.push(CustomMediaWarning {
          range: usage.range,
          message: format!(
            "Custom media query '{}' must be used in a boolean context",
            usage.name
          ),
        });
        continue;
      }
      match resolver.resolve_value(&usage.name, &mut FxHashSet::default()) {
        CustomMediaValue::Unsupported => warnings.push(CustomMediaWarning {
          range: usage.range,
          message: if definitions.contains_key(&usage.name) {
            format!(
              "Custom media query '{}' has a value that cannot be resolved and was left as written",
              usage.name
            )
          } else {
            format!("Unknown custom media query '{}'", usage.name)
          },
        }),
        CustomMediaValue::Boolean(_) => has_boolean = true,
        CustomMediaValue::Type(_) if !usage.leading => warnings.push(CustomMediaWarning {
          range: usage.range,
          message: format!(
            "Custom media query '{}' resolves to a media type and can only be used at the start of a media query",
            usage.name
          ),
        }),
        CustomMediaValue::Condition(_) | CustomMediaValue::Type(_) => {}
      }
    }

    if has_boolean {
      replacements.push(CustomMediaReplacement {
        range: prelude.range,
        content: fold_media_prelude(&mut resolver, &prelude.queries),
      });
      continue;
    }

    for usage in uses {
      if usage.invalid {
        continue;
      }
      let content = match resolver.resolve_value(&usage.name, &mut FxHashSet::default()) {
        CustomMediaValue::Condition(text) => media_condition_for(&text, usage.slot),
        CustomMediaValue::Type(text) if usage.leading => text,
        CustomMediaValue::Unsupported
        | CustomMediaValue::Boolean(_)
        | CustomMediaValue::Type(_) => continue,
      };
      replacements.push(CustomMediaReplacement {
        range: usage.range,
        content,
      });
    }
  }

  CustomMediaTransform {
    replacements,
    warnings,
  }
}

fn tokenize(input: &str) -> Vec<Token> {
  let mut lexer = Lexer::new(input, ());
  let mut tokens = Vec::new();
  loop {
    let token = lexer.next_token();
    tokens.push(token);
    if token.kind == TokenKind::Eof {
      return tokens;
    }
  }
}

fn collect_at_rules(
  input: &str,
  tokens: &[Token],
  definitions: &mut FxHashMap<String, CustomMediaDefinition>,
  media: &mut Vec<MediaPrelude>,
  replacements: &mut Vec<CustomMediaReplacement>,
) {
  let mut index = 0;
  let mut item_start = true;
  while index < tokens.len() {
    let token = tokens[index];
    if is_whitespace(token.kind) {
      index += 1;
      continue;
    }

    if item_start && token.kind == TokenKind::AtKeyword {
      let name = token_value(input, token);
      let name = unescape_identifier(name);
      if name.eq_ignore_ascii_case("custom-media") || name.eq_ignore_ascii_case("media") {
        let end = find_at_rule_end(tokens, index + 1);
        let end_index = end.token_index();
        let components = parse_component_list(input, &tokens[index + 1..end_index]);
        if name.eq_ignore_ascii_case("custom-media") {
          let rule_end = match end {
            AtRuleEnd::Semicolon(end_index) => tokens[end_index].range.end,
            AtRuleEnd::Block(block_index) => find_block_end(tokens, block_index),
            AtRuleEnd::Boundary(end_index) => tokens[end_index].range.start,
          };
          replacements.push(CustomMediaReplacement {
            range: Range::new(token.range.start, rule_end),
            content: String::new(),
          });
          if let Some((name, definition)) = classify_definition(input, &components) {
            definitions.insert(name, definition);
          }
          index = index_after_rule(tokens, end);
          item_start = true;
          continue;
        }

        if let Some(prelude) = capture_media_prelude(input, &components) {
          media.push(prelude);
        }
        index = end_index.saturating_add(1);
        item_start = true;
        continue;
      }
    }

    item_start = match token.kind {
      TokenKind::Semicolon | TokenKind::LeftCurlyBracket | TokenKind::RightCurlyBracket => true,
      TokenKind::Eof => break,
      _ => false,
    };
    index += 1;
  }
}

fn find_at_rule_end(tokens: &[Token], start: usize) -> AtRuleEnd {
  let mut closing = Vec::new();
  for (index, token) in tokens.iter().enumerate().skip(start) {
    match token.kind {
      TokenKind::Function | TokenKind::LeftParenthesis => {
        closing.push(TokenKind::RightParenthesis);
      }
      TokenKind::LeftSquareBracket => closing.push(TokenKind::RightSquareBracket),
      TokenKind::LeftCurlyBracket if closing.is_empty() => return AtRuleEnd::Block(index),
      TokenKind::LeftCurlyBracket => closing.push(TokenKind::RightCurlyBracket),
      TokenKind::Semicolon if closing.is_empty() => return AtRuleEnd::Semicolon(index),
      TokenKind::RightCurlyBracket if closing.is_empty() => {
        return AtRuleEnd::Boundary(index);
      }
      TokenKind::Eof => {
        return AtRuleEnd::Boundary(index);
      }
      kind if closing.last() == Some(&kind) => {
        closing.pop();
      }
      _ => {}
    }
  }
  AtRuleEnd::Boundary(tokens.len().saturating_sub(1))
}

fn find_block_end(tokens: &[Token], block_start: usize) -> u32 {
  let mut depth = 0usize;
  for token in &tokens[block_start..] {
    match token.kind {
      TokenKind::LeftCurlyBracket => depth += 1,
      TokenKind::RightCurlyBracket => {
        depth = depth.saturating_sub(1);
        if depth == 0 {
          return token.range.end;
        }
      }
      TokenKind::Eof => return token.range.start,
      _ => {}
    }
  }
  tokens.last().map_or(0, |token| token.range.end)
}

fn index_after_rule(tokens: &[Token], end: AtRuleEnd) -> usize {
  match end {
    AtRuleEnd::Block(block_start) => {
      let block_end = find_block_end(tokens, block_start);
      tokens[block_start..]
        .iter()
        .position(|token| token.range.end >= block_end)
        .map_or(tokens.len(), |offset| block_start + offset + 1)
    }
    AtRuleEnd::Semicolon(index) | AtRuleEnd::Boundary(index) => index.saturating_add(1),
  }
}

fn parse_component_list(input: &str, tokens: &[Token]) -> Vec<Component> {
  let mut index = 0;
  parse_components(input, tokens, &mut index, None)
}

fn parse_components(
  input: &str,
  tokens: &[Token],
  index: &mut usize,
  closing: Option<TokenKind>,
) -> Vec<Component> {
  let mut components = Vec::new();
  while *index < tokens.len() {
    let token = tokens[*index];
    if closing == Some(token.kind) {
      break;
    }
    *index += 1;

    let kind = match token.kind {
      TokenKind::WhiteSpace | TokenKind::Comment | TokenKind::BadComment => {
        ComponentKind::Whitespace
      }
      TokenKind::Ident => {
        ComponentKind::Ident(unescape_identifier(token_value(input, token)).into_owned())
      }
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

fn consume_closing(tokens: &[Token], index: &mut usize, closing: TokenKind, fallback: u32) -> u32 {
  if tokens
    .get(*index)
    .is_some_and(|token| token.kind == closing)
  {
    let end = tokens[*index].range.end;
    *index += 1;
    end
  } else {
    fallback
  }
}

fn classify_definition(
  input: &str,
  components: &[Component],
) -> Option<(String, CustomMediaDefinition)> {
  let (start, end) = trimmed_bounds(components, 0, components.len());
  let ComponentKind::Ident(name) = &components.get(start)?.kind else {
    return None;
  };
  if !is_dashed_ident(name) {
    return None;
  }
  let (value_start, value_end) = trimmed_bounds(components, start + 1, end);
  Some((
    name.clone(),
    classify_value(input, components, value_start, value_end),
  ))
}

fn classify_value(
  input: &str,
  components: &[Component],
  start: usize,
  end: usize,
) -> CustomMediaDefinition {
  if start >= end {
    return CustomMediaDefinition::Unsupported;
  }
  let has_comma = components[start..end]
    .iter()
    .any(|component| matches!(component.kind, ComponentKind::Comma));
  if has_comma {
    return classify_or_value(input, components, start, end);
  }

  let first = &components[start];
  let text = component_text(input, components, start, end).to_string();
  if end - start == 1
    && let Some((name, has_extra)) = lone_dashed_ident(first)
    && !has_extra
  {
    return CustomMediaDefinition::Alias(name);
  }
  if has_custom_media_ref(&components[start..end]) {
    let node = capture_media_query(input, components, start, end);
    if node_has_invalid_use(&node) {
      return CustomMediaDefinition::Unsupported;
    }
    return CustomMediaDefinition::Query(CustomMediaQuery { node });
  }
  if matches!(
    first.kind,
    ComponentKind::SimpleBlock(BlockKind::Parenthesis, _)
  ) {
    return CustomMediaDefinition::Condition(text);
  }
  let ComponentKind::Ident(first_name) = &first.kind else {
    return CustomMediaDefinition::Unsupported;
  };
  if end - start == 1 && first_name.eq_ignore_ascii_case("true") {
    return CustomMediaDefinition::Boolean(true);
  }
  if end - start == 1 && first_name.eq_ignore_ascii_case("false") {
    return CustomMediaDefinition::Boolean(false);
  }
  if first_name.eq_ignore_ascii_case("not") {
    let (next, _) = trimmed_bounds(components, start + 1, end);
    if next < end
      && matches!(
        components[next].kind,
        ComponentKind::SimpleBlock(BlockKind::Parenthesis, _)
      )
    {
      return CustomMediaDefinition::Condition(format!("({text})"));
    }
  }
  CustomMediaDefinition::Type(text)
}

fn classify_or_value(
  input: &str,
  components: &[Component],
  start: usize,
  end: usize,
) -> CustomMediaDefinition {
  let mut parts = Vec::new();
  let mut segment_start = start;
  for index in start..=end {
    if index != end && !matches!(components[index].kind, ComponentKind::Comma) {
      continue;
    }
    let (part_start, part_end) = trimmed_bounds(components, segment_start, index);
    if part_start >= part_end {
      return CustomMediaDefinition::Unsupported;
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
          return CustomMediaDefinition::Unsupported;
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
      return CustomMediaDefinition::Unsupported;
    }
    if !has_custom_media_ref(&components[part_start..part_end]) {
      parts.push(CustomMediaPart::Text(format!("({text})")));
    } else {
      let node = capture_media_condition(input, components, part_start, part_end, true);
      if node_has_invalid_use(&node) {
        return CustomMediaDefinition::Unsupported;
      }
      parts.push(CustomMediaPart::Query(CustomMediaQuery { node }));
    }
    segment_start = index + 1;
  }
  CustomMediaDefinition::Or(parts)
}

fn capture_media_prelude(input: &str, components: &[Component]) -> Option<MediaPrelude> {
  let (start, end) = trimmed_bounds(components, 0, components.len());
  if start >= end || !has_custom_media_ref(&components[start..end]) {
    return None;
  }
  let mut queries = Vec::new();
  let mut query_start = start;
  for index in start..=end {
    if index != end && !matches!(components[index].kind, ComponentKind::Comma) {
      continue;
    }
    queries.push(capture_media_query(input, components, query_start, index));
    query_start = index + 1;
  }
  if !queries.iter().any(node_has_use) {
    return None;
  }
  Some(MediaPrelude {
    queries,
    range: Range::new(components[start].range.start, components[end - 1].range.end),
  })
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

fn fold_media_prelude(resolver: &mut CustomMediaResolver<'_>, queries: &[MediaNode]) -> String {
  let mut kept = Vec::new();
  for query in queries {
    match resolver.fold_node(query, &mut FxHashSet::default()) {
      FoldedMedia::Boolean(true) => return "all".to_string(),
      FoldedMedia::Boolean(false) => {}
      FoldedMedia::Text(text) if !text.is_empty() => kept.push(text),
      FoldedMedia::Text(_) => {}
    }
  }
  if kept.is_empty() {
    "not all".to_string()
  } else {
    kept.join(", ")
  }
}

fn collect_uses<'a>(node: &'a MediaNode, uses: &mut Vec<&'a CustomMediaUse>) {
  match node {
    MediaNode::Ref { usage, .. } => uses.push(usage),
    MediaNode::Group(operand) | MediaNode::Not(operand) => collect_uses(operand, uses),
    MediaNode::Chain { terms, .. } => {
      for term in terms {
        collect_uses(term, uses);
      }
    }
    MediaNode::Typed {
      rest: Some(rest), ..
    } => collect_uses(rest, uses),
    MediaNode::Text(_) | MediaNode::Typed { rest: None, .. } => {}
  }
}

fn node_has_use(node: &MediaNode) -> bool {
  let mut uses = Vec::new();
  collect_uses(node, &mut uses);
  !uses.is_empty()
}

fn node_has_invalid_use(node: &MediaNode) -> bool {
  let mut uses = Vec::new();
  collect_uses(node, &mut uses);
  uses.into_iter().any(|usage| usage.invalid)
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

fn media_condition_for(text: &str, slot: MediaSlot) -> String {
  if slot == MediaSlot::Alone {
    return text.to_string();
  }
  let mut depth = 0i32;
  let mut groups = 0usize;
  let mut outside = String::new();
  for character in text.chars() {
    match character {
      '(' => {
        if depth == 0 {
          groups += 1;
        }
        depth += 1;
      }
      ')' => depth -= 1,
      _ if depth == 0 => outside.push(character),
      _ => {}
    }
  }
  let words = ascii_words(&outside);
  if groups == 1 && words.is_empty() {
    return text.to_string();
  }
  if slot == MediaSlot::And && words.iter().all(|word| word.eq_ignore_ascii_case("and")) {
    return text.to_string();
  }
  format!("({text})")
}

fn ascii_words(input: &str) -> Vec<&str> {
  let mut words = Vec::new();
  let mut start = None;
  for (index, byte) in input
    .as_bytes()
    .iter()
    .copied()
    .chain(std::iter::once(b' '))
    .enumerate()
  {
    if byte.is_ascii_alphabetic() {
      start.get_or_insert(index);
    } else if let Some(word_start) = start.take() {
      words.push(&input[word_start..index]);
    }
  }
  words
}

fn is_condition_text(text: &str) -> bool {
  let text = text.trim_start();
  if text.starts_with('(') {
    return true;
  }
  let Some(rest) = text.get(3..) else {
    return false;
  };
  text[..3].eq_ignore_ascii_case("not") && rest.trim_start().starts_with('(')
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
  matches!(&component.kind, ComponentKind::Ident(value) if value.eq_ignore_ascii_case(keyword))
}

fn is_dashed_ident(name: &str) -> bool {
  name.starts_with("--") && name.len() > 2
}

fn is_whitespace(kind: TokenKind) -> bool {
  matches!(
    kind,
    TokenKind::WhiteSpace | TokenKind::Comment | TokenKind::BadComment
  )
}

fn token_value(input: &str, token: Token) -> &str {
  Lexer::slice_range(input, &token.value_range).expect("lexer token range should be valid")
}
