use rspack_core::{
  CodeGenerationDataImportBindings, Compilation, ModuleIdentifier, RuntimeSpec,
  rspack_sources::{Source, SourceMap, utf16_len},
};
use rspack_util::{fx_hash::FxIndexSet, swc::AstSubRangeExt};
use rustc_hash::{FxHashMap, FxHashSet};
use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{ImportDeclarationSpecifierData, Lang, SourceType, StmtData};
use swc_next_ecma_parser::{NoTokenParserConfig, Options, Parser};
use swc_next_ecma_semantic::{AnalyzeOptions, SyntaxChecks, analyze, scope::ScopeKind};

use crate::SourceReference;

type Position = (u32, u32);

struct SourceScopes<'a> {
  bindings: Vec<(&'a str, &'a str)>,
  end: Position,
  nested_scopes: Vec<(Position, Position)>,
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
  let mut candidates = FxHashSet::default();
  let mut resources = FxHashMap::default();
  for identifier in modules {
    candidates.insert(identifier);
    if let Some(path) = module_graph
      .module_by_identifier(&identifier)
      .and_then(|module| module.as_normal_module())
      .and_then(|module| module.resource_resolved_data().path())
    {
      resources
        .entry(path.as_str())
        .and_modify(|owner| *owner = None)
        .or_insert(Some(identifier));
    }
  }

  let sources = references
    .into_iter()
    .enumerate()
    .map(|(index, reference)| {
      let module = match reference {
        SourceReference::Module(module) => {
          if !candidates.contains(module)
            || resources.get(module.as_str()).is_some_and(Option::is_none)
          {
            return None;
          }
          *module
        }
        // Loader maps name the original resource rather than the module request.
        SourceReference::Source(path) => (*resources.get(path.as_ref())?)?,
      };
      let bindings = compilation
        .code_generation_results
        .inner()
        .get(&module)?
        .get(runtime)?
        .data()
        .get::<CodeGenerationDataImportBindings>()?;
      let path = match reference {
        SourceReference::Module(_) => module_graph
          .module_by_identifier(&module)?
          .as_normal_module()?
          .resource_resolved_data()
          .path()?
          .as_str(),
        SourceReference::Source(path) => path.as_ref(),
      };
      authored_scopes(path, map.get_source_content(index)?, bindings)
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
    for &(name, expression) in &source.bindings {
      let name = name_index(name);
      encode_delta(&mut scope, name, previous_name);
      previous_name = name;
      encode_unsigned(&mut expressions, name_index(expression) + 1);
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

fn authored_scopes<'a>(
  path: &str,
  content: &str,
  data: &'a CodeGenerationDataImportBindings,
) -> Option<SourceScopes<'a>> {
  let allocator = Allocator::new();
  let result = Parser::init(
    &allocator,
    content,
    Options {
      source_type: SourceType::Module,
      lang: Lang::from_path(path),
      ..Default::default()
    },
    NoTokenParserConfig,
  )
  .parse();
  if !result.diagnostics.is_empty() {
    return None;
  }

  let analyzed = analyze(
    result.ast,
    AnalyzeOptions {
      check_syntax: SyntaxChecks::empty(),
    },
  );
  let (ast, semantic) = analyzed.semantic.into_parts();
  // SAFETY: We only read nodes owned by this parsed AST.
  let ast = unsafe { ast.ast() };
  let mut authored_imports = FxHashSet::default();
  for statement in ast.nodes(ast.root_program().body(ast)) {
    if let StmtData::ImportDeclaration(import) = ast.stmt_data(statement) {
      for specifier in ast.nodes(import.specifiers(ast)) {
        let local = match ast.import_declaration_specifier_data(specifier) {
          ImportDeclarationSpecifierData::ImportSpecifier(specifier) => specifier.local(ast),
          ImportDeclarationSpecifierData::ImportDefaultSpecifier(specifier) => specifier.local(ast),
          ImportDeclarationSpecifierData::ImportNamespaceSpecifier(specifier) => {
            specifier.local(ast)
          }
        };
        authored_imports.insert(ast.get_utf8(local.name(ast)));
      }
    }
  }
  let bindings = data
    .bindings
    .iter()
    .filter(|(name, _)| authored_imports.contains(name.as_str()))
    .map(|(name, expression)| (name.as_str(), expression.as_str()))
    .collect::<Vec<_>>();
  if bindings.is_empty() {
    return None;
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
