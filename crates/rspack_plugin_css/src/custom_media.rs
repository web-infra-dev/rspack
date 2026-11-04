use css_module_lexer::{
  CustomMediaDefinitionValue, CustomMediaPart, CustomMediaText, CustomMediaUse, Dependency,
  DependencyContext, MediaQuery, MediaQueryInstruction, MediaQueryList, MediaSlot, Range,
};
use rustc_hash::FxHashMap;

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

#[derive(Debug)]
pub(super) struct CustomMediaResolution {
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

pub(super) struct CustomMediaResolver<'definitions, 'source> {
  source: &'source str,
  names: FxHashMap<String, usize>,
  definitions: Vec<&'definitions CustomMediaDefinitionValue<'source>>,
  values: Vec<Option<CustomMediaValue>>,
  resolving: Vec<bool>,
}

impl<'definitions, 'source> CustomMediaResolver<'definitions, 'source> {
  pub(super) fn new(
    source: &'source str,
    dependencies: &'definitions DependencyContext<'source>,
  ) -> Self {
    let mut names = FxHashMap::default();
    let mut definitions = Vec::new();
    for dependency in dependencies {
      let Dependency::CustomMedia {
        definition: Some(definition),
        ..
      } = dependency
      else {
        continue;
      };
      // The side-table index is the file-local definition ID. Updating the
      // name map while retaining every node gives custom media its last-wins
      // semantics without another name-based graph.
      let id = definition.index() as usize;
      debug_assert_eq!(id, definitions.len());
      let definition = dependencies.custom_media_definition(*definition);
      definitions.push(&definition.value);
      names.insert(unescape_identifier(definition.name).into_owned(), id);
    }

    let definition_count = definitions.len();
    Self {
      source,
      names,
      definitions,
      values: vec![None; definition_count],
      resolving: vec![false; definition_count],
    }
  }

  fn has_definition(&self, name: &str) -> bool {
    let name = unescape_identifier(name);
    self.names.contains_key(name.as_ref())
  }

  fn resolve_value(&mut self, name: &str) -> CustomMediaValue {
    let name = unescape_identifier(name);
    let Some(&id) = self.names.get(name.as_ref()) else {
      return CustomMediaValue::Unsupported;
    };
    self.resolve_definition(id)
  }

  fn resolve_definition(&mut self, id: usize) -> CustomMediaValue {
    if let Some(value) = &self.values[id] {
      return value.clone();
    }
    if std::mem::replace(&mut self.resolving[id], true) {
      return CustomMediaValue::Unsupported;
    }

    let definition = self.definitions[id];
    let value = match definition {
      CustomMediaDefinitionValue::Unsupported => CustomMediaValue::Unsupported,
      CustomMediaDefinitionValue::Boolean(value) => CustomMediaValue::Boolean(*value),
      CustomMediaDefinitionValue::Condition(text) => {
        CustomMediaValue::Condition(self.custom_media_text(*text))
      }
      CustomMediaDefinitionValue::Type(range) => {
        CustomMediaValue::Type(self.source_text(*range).to_string())
      }
      CustomMediaDefinitionValue::Alias(alias) => match self.resolve_value(alias) {
        CustomMediaValue::Type(_) => CustomMediaValue::Unsupported,
        value => value,
      },
      CustomMediaDefinitionValue::Or(parts) => self.resolve_or(parts),
      CustomMediaDefinitionValue::Query(node) => self.resolve_query(node),
    };

    self.resolving[id] = false;
    self.values[id] = Some(value.clone());
    value
  }

  fn resolve_or(&mut self, parts: &[CustomMediaPart<'source>]) -> CustomMediaValue {
    let mut texts = Vec::new();
    let mut any_true = false;
    for part in parts {
      let value = match part {
        CustomMediaPart::Text(text) => {
          texts.push(self.custom_media_text(*text));
          continue;
        }
        CustomMediaPart::Alias(alias) => self.resolve_value(alias),
        CustomMediaPart::Query(node) => self.resolve_query(node),
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

  fn resolve_query(&mut self, query: &MediaQuery<'source>) -> CustomMediaValue {
    for usage in media_query_uses(query) {
      match self.resolve_value(usage.name) {
        CustomMediaValue::Unsupported => return CustomMediaValue::Unsupported,
        CustomMediaValue::Type(_) if !usage.leading => {
          return CustomMediaValue::Unsupported;
        }
        _ => {}
      }
    }

    match self.fold_query(query) {
      FoldedMedia::Boolean(value) => CustomMediaValue::Boolean(value),
      FoldedMedia::Text(text) if is_condition_text(&text) => CustomMediaValue::Condition(text),
      FoldedMedia::Text(text) => CustomMediaValue::Type(text),
    }
  }

  fn fold_query(&mut self, query: &MediaQuery<'source>) -> FoldedMedia {
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
            match self.resolve_value(usage.name) {
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

  pub(super) fn resolve_media_query_list(
    &mut self,
    prelude: &MediaQueryList<'source>,
  ) -> CustomMediaResolution {
    let mut replacements = Vec::new();
    let mut warnings = Vec::new();
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
      match self.resolve_value(usage.name) {
        CustomMediaValue::Unsupported => warnings.push(CustomMediaWarning {
          range: usage.range,
          message: if self.has_definition(usage.name) {
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
        content: fold_media_prelude(self, &prelude.queries),
      });
    } else {
      for usage in prelude.queries.iter().flat_map(media_query_uses) {
        if usage.invalid {
          continue;
        }
        let content = match self.resolve_value(usage.name) {
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

    CustomMediaResolution {
      replacements,
      warnings,
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

fn fold_media_prelude<'source>(
  resolver: &mut CustomMediaResolver<'_, 'source>,
  queries: &[MediaQuery<'source>],
) -> String {
  let mut kept = Vec::new();
  for query in queries {
    match resolver.fold_query(query) {
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
