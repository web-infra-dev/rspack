use css_module_lexer::{
  CustomMediaDefinitionValue, CustomMediaPart, CustomMediaUse, Dependency, DependencyContext,
  MediaNode, MediaSlot, Range,
};
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

#[derive(Debug, Clone)]
enum CustomMediaValue {
  Unsupported,
  Boolean(bool),
  Condition(String),
  Type(String),
}

struct CustomMediaResolver<'definitions, 'context> {
  definitions: &'definitions FxHashMap<String, &'context CustomMediaDefinitionValue>,
  values: FxHashMap<String, CustomMediaValue>,
}

impl<'definitions, 'context> CustomMediaResolver<'definitions, 'context> {
  fn new(
    definitions: &'definitions FxHashMap<String, &'context CustomMediaDefinitionValue>,
  ) -> Self {
    Self {
      definitions,
      values: FxHashMap::default(),
    }
  }

  fn has_definition(&self, name: &str) -> bool {
    let name = unescape_identifier(name);
    self.definitions.contains_key(name.as_ref())
  }

  fn resolve_value(&mut self, name: &str, resolving: &mut FxHashSet<String>) -> CustomMediaValue {
    let name = unescape_identifier(name);
    self.resolve_normalized_value(name.as_ref(), resolving)
  }

  fn resolve_normalized_value(
    &mut self,
    name: &str,
    resolving: &mut FxHashSet<String>,
  ) -> CustomMediaValue {
    if let Some(value) = self.values.get(name) {
      return value.clone();
    }
    let Some(definition) = self.definitions.get(name).copied() else {
      return CustomMediaValue::Unsupported;
    };
    if !resolving.insert(name.to_string()) {
      return CustomMediaValue::Unsupported;
    }

    let value = match definition {
      CustomMediaDefinitionValue::Unsupported => CustomMediaValue::Unsupported,
      CustomMediaDefinitionValue::Boolean(value) => CustomMediaValue::Boolean(*value),
      CustomMediaDefinitionValue::Condition(text) => CustomMediaValue::Condition(text.clone()),
      CustomMediaDefinitionValue::Type(text) => CustomMediaValue::Type(text.clone()),
      CustomMediaDefinitionValue::Alias(alias) => match self.resolve_value(alias, resolving) {
        CustomMediaValue::Type(_) => CustomMediaValue::Unsupported,
        value => value,
      },
      CustomMediaDefinitionValue::Or(parts) => self.resolve_or(parts, resolving),
      CustomMediaDefinitionValue::Query(node) => self.resolve_query(node, resolving),
    };

    resolving.remove(name);
    self.values.insert(name.to_string(), value.clone());
    value
  }

  fn resolve_or(
    &mut self,
    parts: &[CustomMediaPart],
    resolving: &mut FxHashSet<String>,
  ) -> CustomMediaValue {
    let mut texts = Vec::new();
    let mut any_true = false;
    for part in parts {
      let value = match part {
        CustomMediaPart::Text(text) => {
          texts.push(text.clone());
          continue;
        }
        CustomMediaPart::Alias(alias) => self.resolve_value(alias, resolving),
        CustomMediaPart::Query(node) => self.resolve_query(node, resolving),
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

pub(super) fn transform_custom_media(dependencies: &DependencyContext<'_>) -> CustomMediaTransform {
  let mut definitions = FxHashMap::default();
  let mut media = Vec::new();
  for dependency in dependencies {
    match dependency {
      Dependency::CustomMedia {
        definition: Some(definition),
        ..
      } => {
        let definition = dependencies.custom_media_definition(*definition);
        definitions.insert(
          unescape_identifier(&definition.name).into_owned(),
          &definition.value,
        );
      }
      Dependency::Media { query } => media.push(dependencies.media_query(*query)),
      _ => {}
    }
  }

  let mut replacements = Vec::new();
  let mut warnings = Vec::new();
  let mut resolver = CustomMediaResolver::new(&definitions);
  for prelude in media {
    let mut uses = Vec::new();
    for query in &prelude.queries {
      collect_uses(query, &mut uses);
    }

    let mut has_boolean = false;
    for usage in &uses {
      let name = unescape_identifier(&usage.name);
      if usage.invalid {
        warnings.push(CustomMediaWarning {
          range: usage.range,
          message: format!("Custom media query '{name}' must be used in a boolean context"),
        });
        continue;
      }
      match resolver.resolve_value(&usage.name, &mut FxHashSet::default()) {
        CustomMediaValue::Unsupported => warnings.push(CustomMediaWarning {
          range: usage.range,
          message: if resolver.has_definition(&usage.name) {
            format!(
              "Custom media query '{name}' has a value that cannot be resolved and was left as written"
            )
          } else {
            format!("Unknown custom media query '{name}'")
          },
        }),
        CustomMediaValue::Boolean(_) => has_boolean = true,
        CustomMediaValue::Type(_) if !usage.leading => warnings.push(CustomMediaWarning {
          range: usage.range,
          message: format!(
            "Custom media query '{name}' resolves to a media type and can only be used at the start of a media query"
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

fn fold_media_prelude(resolver: &mut CustomMediaResolver<'_, '_>, queries: &[MediaNode]) -> String {
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
