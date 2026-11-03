use css_module_lexer::{
  CustomMediaDefinitionValue, CustomMediaPart, CustomMediaText, CustomMediaUse, Dependency,
  DependencyContext, MediaQuery, MediaQueryInstruction, MediaSlot, Range,
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

struct CustomMediaResolver<'definitions, 'source> {
  source: &'source str,
  definitions: &'definitions FxHashMap<String, &'definitions CustomMediaDefinitionValue<'source>>,
  values: FxHashMap<String, CustomMediaValue>,
}

impl<'definitions, 'source> CustomMediaResolver<'definitions, 'source> {
  fn new(
    source: &'source str,
    definitions: &'definitions FxHashMap<String, &'definitions CustomMediaDefinitionValue<'source>>,
  ) -> Self {
    Self {
      source,
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
      CustomMediaDefinitionValue::Condition(text) => {
        CustomMediaValue::Condition(self.custom_media_text(*text))
      }
      CustomMediaDefinitionValue::Type(range) => {
        CustomMediaValue::Type(self.source_text(*range).to_string())
      }
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
    parts: &[CustomMediaPart<'source>],
    resolving: &mut FxHashSet<String>,
  ) -> CustomMediaValue {
    let mut texts = Vec::new();
    let mut any_true = false;
    for part in parts {
      let value = match part {
        CustomMediaPart::Text(text) => {
          texts.push(self.custom_media_text(*text));
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
    query: &MediaQuery<'source>,
    resolving: &mut FxHashSet<String>,
  ) -> CustomMediaValue {
    for usage in media_query_uses(query) {
      match self.resolve_value(usage.name, resolving) {
        CustomMediaValue::Unsupported => return CustomMediaValue::Unsupported,
        CustomMediaValue::Type(_) if !usage.leading => {
          return CustomMediaValue::Unsupported;
        }
        _ => {}
      }
    }

    match self.fold_query(query, resolving) {
      FoldedMedia::Boolean(value) => CustomMediaValue::Boolean(value),
      FoldedMedia::Text(text) if is_condition_text(&text) => CustomMediaValue::Condition(text),
      FoldedMedia::Text(text) => CustomMediaValue::Type(text),
    }
  }

  fn fold_query(
    &mut self,
    query: &MediaQuery<'source>,
    resolving: &mut FxHashSet<String>,
  ) -> FoldedMedia {
    let mut stack = Vec::new();
    for instruction in &query.instructions {
      match instruction {
        MediaQueryInstruction::Opaque(range) => {
          stack.push(FoldedMedia::Text(self.source_text(*range).to_string()));
        }
        MediaQueryInstruction::Reference(usage) => {
          let folded = if usage.invalid {
            FoldedMedia::Text(self.source_text(usage.range).to_string())
          } else {
            match self.resolve_value(usage.name, resolving) {
              CustomMediaValue::Boolean(value) => FoldedMedia::Boolean(value),
              CustomMediaValue::Condition(text) => {
                FoldedMedia::Text(media_condition_for(&text, usage.slot))
              }
              CustomMediaValue::Type(text) if usage.leading => FoldedMedia::Text(text),
              CustomMediaValue::Unsupported | CustomMediaValue::Type(_) => {
                FoldedMedia::Text(self.source_text(usage.range).to_string())
              }
            }
          };
          stack.push(folded);
        }
        MediaQueryInstruction::Group => {
          let operand = stack.pop().expect("group has one media operand");
          stack.push(match operand {
            FoldedMedia::Boolean(value) => FoldedMedia::Boolean(value),
            FoldedMedia::Text(text) => FoldedMedia::Text(format!("({text})")),
          });
        }
        MediaQueryInstruction::Not => {
          let operand = stack.pop().expect("not has one media operand");
          stack.push(match operand {
            FoldedMedia::Boolean(value) => FoldedMedia::Boolean(!value),
            FoldedMedia::Text(text) => FoldedMedia::Text(format!("not {text}")),
          });
        }
        MediaQueryInstruction::And(count) => {
          fold_media_terms(&mut stack, *count as usize, false);
        }
        MediaQueryInstruction::Or(count) => {
          fold_media_terms(&mut stack, *count as usize, true);
        }
      }
    }
    debug_assert_eq!(stack.len(), 1);
    stack
      .pop()
      .unwrap_or_else(|| FoldedMedia::Text(String::new()))
  }

  fn source_text(&self, range: Range) -> &'source str {
    &self.source[range.start as usize..range.end as usize]
  }

  fn custom_media_text(&self, text: CustomMediaText) -> String {
    let source = self.source_text(text.range);
    if text.wrap {
      format!("({source})")
    } else {
      source.to_string()
    }
  }
}

enum FoldedMedia {
  Boolean(bool),
  Text(String),
}

fn fold_media_terms(stack: &mut Vec<FoldedMedia>, count: usize, is_or: bool) {
  let start = stack
    .len()
    .checked_sub(count)
    .expect("media chain has enough operands");
  let terms = stack.split_off(start);
  let mut kept = Vec::new();
  for term in terms {
    match term {
      FoldedMedia::Boolean(value) if value == is_or => {
        stack.push(FoldedMedia::Boolean(is_or));
        return;
      }
      FoldedMedia::Boolean(_) => {}
      FoldedMedia::Text(text) => kept.push(text),
    }
  }
  stack.push(if kept.is_empty() {
    FoldedMedia::Boolean(!is_or)
  } else if kept.len() == 1 {
    FoldedMedia::Text(kept.pop().expect("one media term"))
  } else {
    FoldedMedia::Text(kept.join(if is_or { " or " } else { " and " }))
  });
}

pub(super) fn transform_custom_media<'s>(
  source: &'s str,
  dependencies: &DependencyContext<'s>,
) -> CustomMediaTransform {
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
          unescape_identifier(definition.name).into_owned(),
          &definition.value,
        );
      }
      Dependency::Media { query } => media.push(dependencies.media_query(*query)),
      _ => {}
    }
  }

  let mut replacements = Vec::new();
  let mut warnings = Vec::new();
  let mut resolver = CustomMediaResolver::new(source, &definitions);
  for prelude in media {
    let mut has_boolean = false;
    for usage in prelude.queries.iter().flat_map(media_query_uses) {
      let name = unescape_identifier(usage.name);
      if usage.invalid {
        warnings.push(CustomMediaWarning {
          range: usage.range,
          message: format!("Custom media query '{name}' must be used in a boolean context"),
        });
        continue;
      }
      match resolver.resolve_value(usage.name, &mut FxHashSet::default()) {
        CustomMediaValue::Unsupported => warnings.push(CustomMediaWarning {
          range: usage.range,
          message: if resolver.has_definition(usage.name) {
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

    for usage in prelude.queries.iter().flat_map(media_query_uses) {
      if usage.invalid {
        continue;
      }
      let content = match resolver.resolve_value(usage.name, &mut FxHashSet::default()) {
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

fn fold_media_prelude<'source>(
  resolver: &mut CustomMediaResolver<'_, 'source>,
  queries: &[MediaQuery<'source>],
) -> String {
  let mut kept = Vec::new();
  for query in queries {
    match resolver.fold_query(query, &mut FxHashSet::default()) {
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

fn media_query_uses<'query, 'source>(
  query: &'query MediaQuery<'source>,
) -> impl Iterator<Item = &'query CustomMediaUse<'source>> {
  query
    .instructions
    .iter()
    .filter_map(|instruction| match instruction {
      MediaQueryInstruction::Reference(usage) => Some(usage),
      _ => None,
    })
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
