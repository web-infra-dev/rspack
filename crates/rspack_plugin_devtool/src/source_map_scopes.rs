use rspack_core::{
  CodeGenerationDataImportBindings, Compilation, RuntimeSpec,
  rspack_sources::{Source, SourceMap, utf16_len},
};
use rspack_util::fx_hash::FxIndexSet;

use crate::SourceReference;

type Position = (u32, u32);

/// Add module scopes for the ESM imports rewritten during code generation.
pub(super) fn add_import_binding_scopes<'a>(
  compilation: &Compilation,
  runtime: &RuntimeSpec,
  map: &mut SourceMap,
  source: &dyn Source,
  references: impl IntoIterator<Item = &'a SourceReference>,
) {
  if map.scopes().is_some() || map.ranges().is_some() {
    return;
  }

  let bindings = references
    .into_iter()
    .map(|reference| {
      let SourceReference::Module(module) = reference else {
        return None;
      };
      compilation
        .code_generation_results
        .inner()
        .get(module)?
        .get(runtime)?
        .data()
        .get::<CodeGenerationDataImportBindings>()
        .map(|data| &data.bindings)
    })
    .collect::<Vec<_>>();
  if bindings.iter().all(Option::is_none) {
    return;
  }

  // A change of source or an unmapped segment closes the current range.
  // Encode only the runs for which code generation supplies bindings.
  let mut original_ends = vec![None; bindings.len()];
  let mut generated_ranges = Vec::new();
  let mut open = None;
  let mut previous_mapping = (0, 0);
  for mapping in map.decoded_mappings() {
    let generated = (mapping.generated_line - 1, mapping.generated_column);
    if generated < previous_mapping {
      return;
    }
    previous_mapping = generated;
    let mapped_source = mapping.original.and_then(|original| {
      let index = original.source_index as usize;
      bindings.get(index)?.as_ref()?;
      let end = (original.original_line, 0);
      original_ends[index] =
        Some(original_ends[index].map_or(end, |previous: Position| previous.max(end)));
      Some(index)
    });
    if open.map(|(index, _)| index) != mapped_source {
      if let Some((index, start)) = open
        && start < generated
      {
        generated_ranges.push((index, start, generated));
      }
      open = mapped_source.map(|index| (index, generated));
    }
  }
  if let Some((index, start)) = open {
    let mut end = (0, 0);
    source.rope(&mut |text| advance_position(&mut end, text));
    if start < end {
      generated_ranges.push((index, start, end));
    }
  }
  if generated_ranges.is_empty() {
    return;
  }

  // Keep indices of existing mapping names unchanged, including duplicates.
  let mut added_names = FxIndexSet::<String>::default();
  let mut name_index = |name: &str| {
    if let Some(index) = added_names.get_index_of(name) {
      return (map.names().len() + index) as u64;
    }
    let index = (map.names().len() + added_names.len()) as u64;
    added_names.insert(name.to_owned());
    index
  };
  // Only append names for scopes; reusing a name already in mappings is optional.
  let mut definitions = vec![None; bindings.len()];
  let mut scopes = vec![None; bindings.len()];
  for &(index, _, _) in &generated_ranges {
    if scopes[index].is_some() {
      continue;
    }
    let mut end = (0, 0);
    if let Some(content) = map.get_source_content(index) {
      advance_position(&mut end, content);
    } else {
      end = original_ends[index].expect("a generated range has an original mapping");
    }

    // Original scope start with kind, at (0, 0). Each source has its own scope
    // and variable index state. Binding expressions use absolute 1-based indices.
    let mut scope = String::from("BCAA");
    encode_unsigned(&mut scope, name_index("Module") * 2);
    scope.push_str(",D");
    let mut previous_name = 0;
    let mut expressions = String::new();
    for (name, expression) in bindings[index].expect("range has bindings") {
      let name = name_index(name.as_str());
      encode_delta(&mut scope, name, previous_name);
      previous_name = name;
      encode_unsigned(&mut expressions, name_index(expression) + 1);
    }
    scope.push_str(",C");
    encode_unsigned(&mut scope, u64::from(end.0));
    encode_unsigned(&mut scope, u64::from(end.1));
    scopes[index] = Some(scope.into());
    definitions[index] = Some(expressions);
  }

  let mut ranges = String::new();
  let mut previous_position = (0, 0);
  let mut previous_source = 0;
  for (index, start, end) in generated_ranges {
    if !ranges.is_empty() {
      ranges.push(',');
    }
    ranges.push('E');
    // A generated range start always references a definition. Its flags also
    // indicate whether the line changed since the preceding position.
    encode_unsigned(
      &mut ranges,
      if start.0 == previous_position.0 { 2 } else { 3 },
    );
    encode_position(&mut ranges, start, previous_position);
    encode_delta(&mut ranges, index as u64, previous_source);
    ranges.push('A'); // The only scope in the source has index zero.
    previous_source = index as u64;
    ranges.push_str(",G");
    ranges.push_str(
      definitions[index]
        .as_deref()
        .expect("range has a definition"),
    );
    ranges.push_str(",F");
    encode_position(&mut ranges, end, start);
    previous_position = end;
  }
  let mut map_names = map.names().to_vec();
  map_names.extend(added_names.into_iter().map(Into::into));
  map.set_names(map_names);
  map.set_scopes(scopes, vec![ranges.into()]);
}

fn advance_position(position: &mut Position, text: &str) {
  if let Some((_, last_line)) = text.rsplit_once('\n') {
    position.0 += memchr::memchr_iter(b'\n', text.as_bytes()).count() as u32;
    position.1 = utf16_len(last_line) as u32;
  } else {
    position.1 += utf16_len(text) as u32;
  }
}

fn encode_position(out: &mut String, position: Position, previous: Position) {
  if position.0 == previous.0 {
    encode_unsigned(out, u64::from(position.1 - previous.1));
  } else {
    encode_unsigned(out, u64::from(position.0 - previous.0));
    encode_unsigned(out, u64::from(position.1));
  }
}

fn encode_delta(out: &mut String, index: u64, previous: u64) {
  encode_unsigned(
    out,
    2 * index.abs_diff(previous) + u64::from(index < previous),
  );
}

fn encode_unsigned(out: &mut String, mut value: u64) {
  const BASE64: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
  while value >= 32 {
    out.push(char::from(BASE64[(value as usize & 31) | 32]));
    value >>= 5;
  }
  out.push(char::from(BASE64[value as usize]));
}
