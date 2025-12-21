use std::collections::BTreeSet;

use rspack_util::quote_meta;
use rustc_hash::FxHashMap as HashMap;

pub enum BooleanMatcher {
  Condition(bool),
  Matcher(Box<dyn Fn(String) -> String + Send + Sync + 'static>),
}

impl BooleanMatcher {
  pub fn render(&self, value: &str) -> String {
    match self {
      Self::Condition(c) => c.to_string(),
      Self::Matcher(m) => m(value.to_string()),
    }
  }
}

fn to_simple_string(input: &str) -> String {
  if input.parse::<f64>().is_ok_and(|n| input == n.to_string()) {
    input.to_string()
  } else {
    serde_json::to_string(input).unwrap_or_default()
  }
}

pub fn compile_boolean_matcher_from_lists(
  positive_items: Vec<String>,
  negative_items: Vec<String>,
) -> BooleanMatcher {
  if positive_items.is_empty() {
    BooleanMatcher::Matcher(Box::new(|_| "false".to_string()))
  } else if negative_items.is_empty() {
    BooleanMatcher::Matcher(Box::new(|_| "true".to_string()))
  } else if positive_items.len() == 1 {
    let item = to_simple_string(&positive_items[0]);
    BooleanMatcher::Matcher(Box::new(move |value| format!("{item} == {value}")))
  } else if negative_items.len() == 1 {
    let item = to_simple_string(&negative_items[0]);
    BooleanMatcher::Matcher(Box::new(move |value| format!("{item} != {value}")))
  } else {
    let positive_regexp = items_to_regexp(positive_items);
    let negative_regexp = items_to_regexp(negative_items);

    if positive_regexp.len() <= negative_regexp.len() {
      BooleanMatcher::Matcher(Box::new(move |value| {
        format!("/^{positive_regexp}$/.test({value})")
      }))
    } else {
      BooleanMatcher::Matcher(Box::new(move |value| {
        format!("!/^{negative_regexp}$/.test({value})")
      }))
    }
  }
}

pub fn compile_boolean_matcher(map: &HashMap<String, bool>) -> BooleanMatcher {
  let positive_items = map
    .iter()
    .filter(|(_, v)| **v)
    .map(|(k, _)| k.to_owned())
    .collect::<Vec<_>>();
  if positive_items.is_empty() {
    return BooleanMatcher::Condition(false);
  }
  let negative_items = map
    .iter()
    .filter(|(_, v)| !**v)
    .map(|(k, _)| k.to_owned())
    .collect::<Vec<_>>();
  if negative_items.is_empty() {
    return BooleanMatcher::Condition(true);
  }

  compile_boolean_matcher_from_lists(positive_items, negative_items)
}

/// AOT regex optimization, copy from webpack https://github.com/webpack/webpack/blob/1f99ad6367f2b8a6ef17cce0e058f7a67fb7db18/lib/util/compileBooleanMatcher.js#L134-L233
pub(crate) fn items_to_regexp(items_arr: Vec<String>) -> String {
  if items_arr.len() == 1 {
    return quote_meta(&items_arr[0]);
  }

  let mut finished_items = Vec::new();
  let mut items_set: Vec<&str> = items_arr.iter().map(|s| s.as_str()).collect();
  items_set.sort_unstable();

  // Merge single char items: (a|b|c|d|ef) => ([abcd]|ef)
  let count_of_single_char_items = items_set
    .iter()
    .filter(|&item| item.chars().count() == 1)
    .count();

  // Special case for only single char items
  if count_of_single_char_items == items_set.len() {
    let mut items_arr = items_set.into_iter().collect::<Vec<_>>();
    items_arr.sort_unstable();
    let single_char_items = items_arr.join("");
    return format!("[{}]", quote_meta(&single_char_items));
  }

  // align with js insertion order https://github.com/webpack/webpack/blob/1f99ad6367f2b8a6ef17cce0e058f7a67fb7db18/lib/util/compileBooleanMatcher.js#L152
  let mut items = items_arr.iter().cloned().collect::<BTreeSet<_>>();

  if count_of_single_char_items > 2 {
    let mut single_char_items: String = String::new();
    let mut new_items = BTreeSet::new();
    for item in items {
      if item.chars().count() == 1 {
        single_char_items += &item;
        continue;
      }
      new_items.insert(item);
    }
    items = new_items;
    finished_items.push(format!("[{}]", quote_meta(&single_char_items)));
  }

  // Special case for 2 items with common prefix/suffix
  if finished_items.is_empty() && items.len() == 2 {
    let prefix = get_common_prefix(items.iter().map(|item| item.as_str()));
    let suffix = get_common_suffix(items.iter().map(|item| &item[prefix.len()..]));

    if !prefix.is_empty() || !suffix.is_empty() {
      return format!(
        "{}{}{}",
        quote_meta(prefix),
        items_to_regexp(
          items
            .iter()
            .map(|item| item
              .strip_prefix(prefix)
              .expect("should strip prefix")
              .strip_suffix(&suffix)
              .expect("should strip suffix")
              .to_string())
            .collect::<Vec<_>>()
        ),
        quote_meta(suffix)
      );
    }
  }

  // Special case for 2 items with common suffix https://github.com/webpack/webpack/blob/1f99ad6367f2b8a6ef17cce0e058f7a67fb7db18/lib/util/compileBooleanMatcher.js#L178-L189
  if finished_items.is_empty() && items.len() == 2 {
    let mut it = items.iter();
    let a = it.next().expect("should have two element");
    let b = it.next().expect("should have two element");

    if !a.is_empty()
      && !b.is_empty()
      && a.ends_with(
        b.chars()
          .last()
          .expect("should have last char since b is not empty"),
      )
    {
      return format!(
        "{}{}",
        items_to_regexp(vec![
          a[0..a.len() - 1].to_string(),
          b[0..b.len() - 1].to_string()
        ]),
        quote_meta(&a[a.len() - 1..])
      );
    }
  }

  // Find common prefix: (a1|a2|a3|a4|b5) => (a(1|2|3|4)|b5)
  let prefixed = pop_common_items(
    &mut items,
    |item| {
      if !item.is_empty() {
        Some(
          item
            .chars()
            .next()
            .expect("should have at least one char")
            .to_string(),
        )
      } else {
        None
      }
    },
    |list| {
      if list.len() >= 3 {
        true
      } else if list.len() <= 1 {
        false
      } else {
        list[0].chars().nth(1) == list[1].chars().nth(1)
      }
    },
  );

  for prefixed_items in prefixed {
    let prefix = get_common_prefix(prefixed_items.iter().map(|item| item.as_str()));
    finished_items.push(format!(
      "{}{}",
      quote_meta(prefix),
      items_to_regexp(
        prefixed_items
          .iter()
          .map(|item| item
            .strip_prefix(prefix)
            .expect("should strip prefix")
            .to_string())
          .collect::<Vec<_>>()
      )
    ));
  }

  // Find common suffix: (a1|b1|c1|d1|e2) => ((a|b|c|d)1|e2)
  let suffixed = pop_common_items(
    &mut items,
    |item| {
      if !item.is_empty() {
        Some(
          item
            .chars()
            .last()
            .expect("should have at least one char")
            .to_string(),
        )
      } else {
        None
      }
    },
    |list| {
      if list.len() >= 3 {
        true
      } else if list.len() <= 1 {
        false
      } else {
        let s = if list[0].len() >= 2 {
          list[0].len() - 2
        } else {
          list[0].len()
        };
        let e = if list[1].len() >= 2 {
          list[1].len() - 2
        } else {
          list[1].len()
        };
        list[0].chars().skip(s).collect::<String>() == list[1].chars().skip(e).collect::<String>()
      }
    },
  );

  for suffixed_items in suffixed {
    let suffix = get_common_suffix(suffixed_items.iter().map(|item| item.as_str()));
    finished_items.push(format!(
      "{}{}",
      items_to_regexp(
        suffixed_items
          .iter()
          .map(|item| item
            .strip_suffix(&suffix)
            .expect("should strip suffix")
            .to_string())
          .collect::<Vec<_>>()
      ),
      quote_meta(suffix)
    ));
  }

  // TODO(from webpack) further optimize regexp, i.e., use ranges: (1|2|3|4|a) => [1-4a]
  let conditional = finished_items
    .into_iter()
    .chain(items.iter().map(|item| quote_meta(item)))
    .collect::<Vec<String>>();

  if conditional.len() == 1 {
    conditional[0].clone()
  } else {
    format!("({})", conditional.join("|"))
  }
}

fn pop_common_items<T, F, G>(items_set: &mut BTreeSet<T>, get_key: F, condition: G) -> Vec<Vec<T>>
where
  T: Clone + PartialEq + Eq + std::hash::Hash + Ord,
  F: Fn(&T) -> Option<String>,
  G: Fn(&[T]) -> bool,
{
  let mut map: HashMap<String, Vec<T>> = HashMap::default();

  for item in items_set.iter() {
    if let Some(key) = get_key(item) {
      let list = map.entry(key).or_default();
      list.push(item.clone());
    }
  }

  let mut result = Vec::new();

  for list in map.values() {
    if condition(list) {
      for item in list {
        items_set.remove(item);
      }
      result.push(list.clone());
    }
  }

  result
}

fn get_common_prefix<'a>(mut items: impl Iterator<Item = &'a str> + Clone) -> &'a str {
  let mut prefix = if let Some(prefix) = items.next() {
    prefix
  } else {
    return "";
  };

  for item in items {
    for (char_index, (byte_index, c)) in item.char_indices().enumerate() {
      if let Some(prefix_char) = prefix.chars().nth(char_index) {
        if c != prefix_char {
          prefix = &prefix[..byte_index];
          break;
        }
      } else {
        break;
      }
    }
  }

  prefix
}

fn is_utf8_start_byte(c: u8) -> bool {
  c.is_ascii()
    || ((c & 0b1110_0000) == 0b1100_0000)
    || ((c & 0b1111_0000) == 0b1110_0000)
    || ((c & 0b1111_1000) == 0b1111_0000)
}

fn get_common_suffix<'a, I: Iterator<Item = &'a str>>(mut items: I) -> &'a str {
  let mut suffix = if let Some(suffix) = items.next() {
    suffix.as_bytes()
  } else {
    return "";
  };

  for item in items {
    let item = item.as_bytes();

    let mut p = item.len();
    let mut s = suffix.len();

    while s > 0 {
      s -= 1;
      let suffix_byte = suffix[s];

      if p > 0 {
        let item_byte = item[p - 1];
        if suffix_byte == item_byte {
          p -= 1;
          continue;
        }
      }
      suffix = &suffix[s + 1..];
      break;
    }
  }

  while !suffix.is_empty() && !is_utf8_start_byte(suffix[0]) {
    suffix = &suffix[1..]
  }
  unsafe { std::str::from_utf8_unchecked(suffix) }
}
