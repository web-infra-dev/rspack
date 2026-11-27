use rspack_core::{
  CodeGenerationDataImportBindings, Compilation, ModuleIdentifier, RuntimeSpec, SourceType,
  rspack_sources::{MapOptions, Mapping, Source, SourceMap, utf16_len},
};
use rspack_util::fx_hash::FxIndexSet;
use rustc_hash::{FxHashMap, FxHashSet};
use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{Ast, AstCtx, Lang, ScopeId, SourceType as AstSourceType};
use swc_next_ecma_parser::{NoTokenParserConfig, Options, Parser};
use swc_next_ecma_semantic::{
  AnalyzeOptions, Semantic, SymbolFlags, SyntaxChecks, analyze, scope::ScopeKind,
};

use crate::SourceReference;

type Position = (u32, u32);

struct SourceScopes {
  bindings: Vec<(String, Option<String>)>,
  end: Position,
  nested_scopes: Vec<(Position, Position)>,
}

enum SourceOwner {
  Unmapped,
  Module(ModuleIdentifier, u32),
  Shared,
}

/// Add module scopes for the ESM imports rewritten during code generation.
pub(super) fn add_import_binding_scopes<'a>(
  compilation: &Compilation,
  runtime: &RuntimeSpec,
  map: &mut SourceMap,
  source: &dyn Source,
  references: impl IntoIterator<Item = &'a SourceReference>,
  modules: impl IntoIterator<Item = ModuleIdentifier>,
) {
  if map.scopes().is_some() || map.ranges().is_some() {
    return;
  }

  let module_graph = compilation.get_module_graph();
  let codegen = compilation.code_generation_results.inner();
  let mut owners: FxHashMap<&str, SourceOwner> = map
    .sources()
    .iter()
    .map(|source| (source.as_str(), SourceOwner::Unmapped))
    .collect();
  let mut modules_with_bindings = FxHashMap::default();
  let options = MapOptions::new(true);
  for identifier in modules {
    let Some(result) = codegen
      .get(&identifier)
      .and_then(|entry| entry.get(runtime))
    else {
      continue;
    };
    let Some(generated) = result.get(&SourceType::JavaScript) else {
      continue;
    };
    let Some(module_map) = generated.map(&options) else {
      continue;
    };
    // ConcatSource deduplicates these names across all modules. Inspect each
    // module map before that provenance is lost, including modules that have
    // no import metadata of their own.
    for (index, name) in module_map.sources().iter().enumerate() {
      if let Some(owner) = owners.get_mut(name.as_str()) {
        *owner = if matches!(owner, SourceOwner::Unmapped) {
          SourceOwner::Module(identifier, index as u32)
        } else {
          SourceOwner::Shared
        };
      }
    }
    if let Some(bindings) = result.data().get::<CodeGenerationDataImportBindings>() {
      modules_with_bindings.insert(identifier, (generated, module_map, bindings));
    }
  }

  let sources = references
    .into_iter()
    .enumerate()
    .map(|(index, reference)| {
      let name = map.get_source(index)?;
      let &SourceOwner::Module(module, source_index) = owners.get(name)? else {
        return None;
      };
      let (generated, module_map, bindings) = modules_with_bindings.get(&module)?;
      let path = module_graph
        .module_by_identifier(&module)?
        .as_normal_module()?
        .resource_resolved_data()
        .path()?
        .as_str();
      match reference {
        SourceReference::Module(reference) if *reference == module => {}
        // Loader maps can name the authored resource instead of the module request.
        SourceReference::Source(reference) if reference.as_ref() == path => {}
        _ => return None,
      }
      authored_scopes(
        path,
        map.get_source_content(index)?,
        bindings,
        generated.as_ref(),
        module_map,
        source_index,
      )
    })
    .collect::<Vec<_>>();
  if sources.iter().all(Option::is_none) {
    return;
  }

  // Chrome treats the encoded scope chain as authoritative. Leave nested
  // scopes to its normal debugger when we can only describe the module scope.
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
      let source = sources.get(index)?.as_ref()?;
      let position = (original.original_line - 1, original.original_column);
      let preceding = source
        .nested_scopes
        .partition_point(|&(start, _)| start <= position);
      (preceding == 0 || source.nested_scopes[preceding - 1].1 <= position).then_some(index)
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
  let mut definitions = vec![None; sources.len()];
  let mut scopes = vec![None; sources.len()];
  for &(index, _, _) in &generated_ranges {
    if scopes[index].is_some() {
      continue;
    }
    let source = sources[index]
      .as_ref()
      .expect("range has an original scope");

    // Original scope start with kind, at (0, 0). Each source has its own scope
    // and variable index state. Binding expressions use absolute 1-based indices.
    let mut scope = String::from("BCAA");
    encode_unsigned(&mut scope, name_index("Module") * 2);
    scope.push_str(",D");
    let mut previous_name = 0;
    let mut expressions = String::new();
    for (name, expression) in &source.bindings {
      let name = name_index(name);
      encode_delta(&mut scope, name, previous_name);
      previous_name = name;
      encode_unsigned(
        &mut expressions,
        expression
          .as_deref()
          .map_or(0, |value| name_index(value) + 1),
      );
    }
    scope.push_str(",C");
    encode_unsigned(&mut scope, u64::from(source.end.0));
    encode_unsigned(&mut scope, u64::from(source.end.1));
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

fn parse_source<'a>(
  allocator: &'a Allocator,
  content: &'a str,
  lang: Lang,
  source_type: AstSourceType,
) -> Option<AstCtx<'a>> {
  let parse = |lang| {
    let result = Parser::init(
      allocator,
      content,
      Options {
        source_type,
        lang,
        ..Default::default()
      },
      NoTokenParserConfig,
    )
    .parse();
    result.diagnostics.is_empty().then_some(result.ast)
  };
  parse(lang).or_else(|| match lang {
    // Loaders can enable JSX in files without an x suffix.
    Lang::Js => parse(Lang::Jsx),
    Lang::Ts => parse(Lang::Tsx),
    _ => None,
  })
}

fn authored_scopes(
  path: &str,
  content: &str,
  data: &CodeGenerationDataImportBindings,
  generated: &dyn Source,
  module_map: &SourceMap,
  source_index: u32,
) -> Option<SourceScopes> {
  let allocator = Allocator::new();
  let parsed = parse_source(
    &allocator,
    content,
    Lang::from_path(path),
    AstSourceType::Module,
  )?;
  let analyzed = analyze(
    parsed,
    AnalyzeOptions {
      check_syntax: SyntaxChecks::empty(),
    },
  );
  let (ast, semantic) = analyzed.semantic.into_parts();
  // SAFETY: We only read nodes owned by this parsed AST.
  let ast = unsafe { ast.ast() };
  let imports = semantic
    .iter_symbols()
    .filter(|entry| {
      entry.symbol.flags.contains(SymbolFlags::IMPORT)
        && semantic.scope_of_symbol(entry.id) == ScopeId::MODULE
    })
    .filter_map(|entry| ast.get_wtf8(entry.symbol.name).as_str())
    .collect::<FxHashSet<_>>();
  let mut bindings = data
    .bindings
    .iter()
    .filter(|(name, _)| imports.contains(name.as_str()))
    .map(|(name, value)| (name.as_str().to_owned(), Some(value.clone())))
    .collect::<Vec<_>>();
  if bindings.is_empty() {
    return None;
  }

  let authored_lines = SourceLines::new(content);
  let mut locals = FxHashMap::default();
  // Include authored value declarations so DevTools can show them alongside
  // imports. A declaration without a matching generated binding is unavailable.
  for entry in semantic.iter_symbols() {
    if !is_local_value(entry.symbol.flags) || semantic.scope_of_symbol(entry.id) != ScopeId::MODULE
    {
      continue;
    }
    let name = ast.get_wtf8(entry.symbol.name).as_str()?;
    let index = bindings.len();
    bindings.push((name.to_owned(), None));
    for declaration in semantic.declarations(entry.id) {
      if is_local_value(declaration.flags)
        && let Some(position) = authored_lines.position(ast.span(declaration.node).start as usize)
      {
        locals.insert(position, index);
      }
    }
  }
  if !locals.is_empty() {
    match_local_values(
      &mut bindings,
      &locals,
      &authored_lines,
      generated,
      module_map,
      source_index,
    );
  }

  let mut spans = semantic
    .iter_scopes()
    .filter_map(|entry| {
      let scope = entry.scope;
      (!matches!(scope.kind, ScopeKind::Global | ScopeKind::Module)).then(|| {
        let span = ast.span(scope.node);
        (span.start as usize, span.end as usize)
      })
    })
    .collect::<Vec<_>>();
  spans.sort_unstable();

  let mut nested_scopes: Vec<(Position, Position)> = Vec::new();
  let mut position = (0, 0);
  let mut offset = 0;
  for (start, end) in spans {
    if start >= end || end <= offset {
      continue;
    }
    if start < offset {
      advance_position(&mut position, content.get(offset..end)?);
      nested_scopes.last_mut()?.1 = position;
    } else {
      advance_position(&mut position, content.get(offset..start)?);
      let start_position = position;
      advance_position(&mut position, content.get(start..end)?);
      nested_scopes.push((start_position, position));
    }
    offset = end;
  }
  advance_position(&mut position, content.get(offset..)?);
  Some(SourceScopes {
    bindings,
    end: position,
    nested_scopes,
  })
}

fn is_local_value(flags: SymbolFlags) -> bool {
  flags.in_value_space()
    && !flags.intersects(SymbolFlags::ANY_IMPORT | SymbolFlags::AMBIENT | SymbolFlags::CONST_ENUM)
}

fn match_local_values(
  bindings: &mut [(String, Option<String>)],
  locals: &FxHashMap<Position, usize>,
  authored: &SourceLines<'_>,
  generated: &dyn Source,
  map: &SourceMap,
  source_index: u32,
) {
  let generated = generated.source().into_string_lossy();
  let allocator = Allocator::new();
  let Some(parsed) = parse_source(&allocator, &generated, Lang::Js, AstSourceType::Unambiguous)
  else {
    return;
  };
  let (ast, semantic) = analyze(
    parsed,
    AnalyzeOptions {
      check_syntax: SyntaxChecks::empty(),
    },
  )
  .semantic
  .into_parts();
  // SAFETY: We only read nodes owned by this parsed AST.
  let ast = unsafe { ast.ast() };
  let generated = SourceLines::new(&generated);
  let mut declarations = generated_declarations(ast, &semantic, &generated);
  declarations.sort_unstable_by_key(|&(position, _)| position);

  let position = |mapping: &Mapping| (mapping.generated_line - 1, mapping.generated_column);
  let mut mappings = map.decoded_mappings().peekable();
  let mut active = None;
  let mut matched = FxHashMap::default();
  for ((line, column), name) in declarations {
    while mappings
      .peek()
      .is_some_and(|mapping| position(mapping) <= (line, column))
    {
      active = mappings.next();
    }
    let Some(mapping) = active.filter(|mapping| position(mapping).0 == line) else {
      continue;
    };
    let Some(original) = mapping
      .original
      .filter(|original| original.source_index == source_index)
    else {
      continue;
    };
    let delta = column - mapping.generated_column;
    let Some(original_line) = original.original_line.checked_sub(1) else {
      continue;
    };
    let Some(original_column) = original.original_column.checked_add(delta) else {
      continue;
    };
    let Some(index) = locals.get(&(original_line, original_column)) else {
      continue;
    };
    // Unchanged stretches of text can have just one mapping at their start.
    // Interpolate only within a line and require the prefix and name to match.
    if delta != 0
      && (bindings[*index].0 != name
        || !generated
          .line(line)
          .encode_utf16()
          .skip(mapping.generated_column as usize)
          .take(delta as usize)
          .eq(
            authored
              .line(original_line)
              .encode_utf16()
              .skip(original.original_column as usize)
              .take(delta as usize),
          ))
    {
      continue;
    }
    matched
      .entry(*index)
      .and_modify(|value| {
        if *value != Some(name) {
          *value = None;
        }
      })
      .or_insert(Some(name));
  }
  for (index, name) in matched {
    bindings[index].1 = name.map(str::to_owned);
  }
}

fn generated_declarations<'a>(
  ast: &'a Ast<'_>,
  semantic: &Semantic<'_>,
  source: &SourceLines<'_>,
) -> Vec<(Position, &'a str)> {
  let root = ScopeId::program(ast.source_type());
  let mut declarations = Vec::new();
  for entry in semantic.iter_symbols() {
    if !is_local_value(entry.symbol.flags) || semantic.scope_of_symbol(entry.id) != root {
      continue;
    }
    let Some(name) = ast.get_wtf8(entry.symbol.name).as_str() else {
      continue;
    };
    for declaration in semantic.declarations(entry.id) {
      if let Some(position) = source.position(ast.span(declaration.node).start as usize) {
        declarations.push((position, name));
      }
    }
  }
  declarations
}

struct SourceLines<'a> {
  text: &'a str,
  starts: Vec<usize>,
}

impl<'a> SourceLines<'a> {
  fn new(text: &'a str) -> Self {
    let starts = std::iter::once(0)
      .chain(memchr::memchr_iter(b'\n', text.as_bytes()).map(|offset| offset + 1))
      .collect();
    Self { text, starts }
  }

  fn position(&self, offset: usize) -> Option<Position> {
    let line = self.starts.partition_point(|&start| start <= offset) - 1;
    let column = utf16_len(self.text.get(self.starts[line]..offset)?);
    Some((line as u32, column as u32))
  }

  fn line(&self, line: u32) -> &str {
    let line = line as usize;
    self.starts.get(line).map_or("", |&start| {
      &self.text[start
        ..self
          .starts
          .get(line + 1)
          .copied()
          .unwrap_or(self.text.len())]
    })
  }
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
