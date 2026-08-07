use rspack_hash::{RspackHash, RspackHasher};
use rspack_sources::{BoxSource, ConcatSource, RawStringSource, ReplaceSource, Source, SourceExt};
use rspack_util::SpanExt;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use smallvec::SmallVec;
use swc_core::{
  atoms::Atom,
  common::{BytePos, Span, Spanned, SyntaxContext},
  ecma::visit::swc_ecma_ast,
};

use super::{
  CONCATENATION_PLACEHOLDER_PREFIX, ConcatenatedModuleInfo, FasterModuleConcatenationInfo,
  GeneratedTopLevelSymbol, MODULE_REFERENCE_PREFIX, MODULE_REFERENCE_SUFFIX,
  OriginalScopeIdentUpdate,
};
use crate::{
  ConcatenatedModuleIdent, ConcatenationScope, ConcatenationScopeIdentKind, DependencyRange,
  PendingConcatenationScopeInfo, RenderedInitFragments,
};

struct PlaceholderReplacements<'a> {
  module_references: &'a [(String, String)],
  generated_symbols: &'a [GeneratedTopLevelSymbol],
  internal_names: &'a HashMap<Atom, Atom>,
}

impl PlaceholderReplacements<'_> {
  fn is_empty(&self) -> bool {
    self.module_references.is_empty() && self.generated_symbols.is_empty()
  }

  fn get<'a>(&'a self, placeholder: &str) -> Option<&'a str> {
    self
      .module_references
      .iter()
      .find(|(candidate, _)| candidate == placeholder)
      .map(|(_, name)| name.as_str())
      .or_else(|| {
        self
          .generated_symbols
          .iter()
          .find(|symbol| symbol.placeholder == placeholder)
          .and_then(|symbol| self.internal_names.get(&symbol.placeholder))
          .map(Atom::as_str)
      })
  }
}

pub(super) fn is_plain_identifier_name(name: &str) -> bool {
  let mut chars = name.chars();
  let Some(first) = chars.next() else {
    return false;
  };
  if !(first == '_' || first == '$' || first.is_ascii_alphabetic()) {
    return false;
  }
  chars.all(|ch| ch == '_' || ch == '$' || ch.is_ascii_alphanumeric())
}

fn is_ident_removed(ident: &ConcatenatedModuleIdent, updates: &[OriginalScopeIdentUpdate]) -> bool {
  if updates.is_empty() {
    return false;
  }
  let span = ident.id.span();
  let low = span.real_lo();
  let high = span.real_hi();
  updates.iter().any(|update| match update {
    OriginalScopeIdentUpdate::Remove(range) => range.start <= low && high <= range.end,
    OriginalScopeIdentUpdate::NonShorthand(_) => false,
  })
}

fn is_ident_shorthand(
  range: DependencyRange,
  shorthand: bool,
  updates: &[OriginalScopeIdentUpdate],
) -> bool {
  shorthand
    && !updates.iter().any(|update| {
      matches!(update, OriginalScopeIdentUpdate::NonShorthand(updated_range) if *updated_range == range)
    })
}

pub(crate) fn populate_info_from_pending(
  pending: &PendingConcatenationScopeInfo,
  original_source: &str,
  module_info: &mut ConcatenatedModuleInfo,
  faster_info: &FasterModuleConcatenationInfo,
) {
  let mut symbols = SmallVec::<[(&str, Atom); 8]>::new();
  let mut symbol_from_range = |range: DependencyRange| {
    let symbol = original_source
      .get(range.start as usize..range.end as usize)
      .unwrap_or_else(|| {
        panic!(
          "concatenation scope symbol range {}..{} should be in the original source",
          range.start, range.end
        )
      });
    if let Some((_, interned)) = symbols.iter().find(|(candidate, _)| *candidate == symbol) {
      return interned.clone();
    }
    let interned = Atom::from(symbol);
    symbols.push((symbol, interned.clone()));
    interned
  };
  module_info.module_ctxt = SyntaxContext::from_u32(pending.module_ctxt);
  module_info.global_ctxt = SyntaxContext::from_u32(pending.global_ctxt);
  module_info.idents.clear();
  module_info.global_scope_ident.clear();
  module_info.binding_to_ref.clear();
  module_info.all_used_names = pending
    .idents
    .iter()
    .filter(|ident| {
      matches!(
        ident.kind,
        ConcatenationScopeIdentKind::Global | ConcatenationScopeIdentKind::UsedName
      )
    })
    .map(|ident| symbol_from_range(ident.range))
    .collect();
  module_info
    .all_used_names
    .extend(faster_info.added_used_names.iter().cloned());

  if !module_info.module_reference_placeholders.is_empty() {
    let mut seen = HashSet::default();
    module_info
      .module_reference_placeholders
      .retain(|placeholder| seen.insert(placeholder.clone()));
  }

  let mut pending_ident_to_legacy =
    |range: DependencyRange, shorthand: bool, ctxt: SyntaxContext| ConcatenatedModuleIdent {
      id: swc_ecma_ast::Ident::new(
        symbol_from_range(range),
        Span::new(
          BytePos(range.start.saturating_add(1)),
          BytePos(range.end.saturating_add(1)),
        ),
        ctxt,
      ),
      shorthand,
      is_class_expr_with_ident: false,
    };
  let mut idents = Vec::with_capacity(pending.idents.len() + faster_info.added_scope_idents.len());
  idents.extend(
    pending
      .idents
      .iter()
      .filter(|ident| ident.kind == ConcatenationScopeIdentKind::TopLevel)
      .map(|ident| {
        pending_ident_to_legacy(
          ident.range,
          is_ident_shorthand(
            ident.range,
            ident.shorthand,
            &faster_info.original_scope_ident_updates,
          ),
          module_info.module_ctxt,
        )
      })
      .filter(|ident| !is_ident_removed(ident, &faster_info.original_scope_ident_updates)),
  );
  idents.extend(
    pending
      .idents
      .iter()
      .filter(|ident| ident.kind == ConcatenationScopeIdentKind::Global)
      .map(|ident| {
        pending_ident_to_legacy(
          ident.range,
          is_ident_shorthand(
            ident.range,
            ident.shorthand,
            &faster_info.original_scope_ident_updates,
          ),
          module_info.global_ctxt,
        )
      })
      .filter(|ident| !is_ident_removed(ident, &faster_info.original_scope_ident_updates)),
  );
  idents.extend(
    faster_info
      .added_scope_idents
      .iter()
      .map(|ident| ConcatenatedModuleIdent {
        id: swc_ecma_ast::Ident::new(
          ident.symbol.clone(),
          Span::new(
            BytePos(ident.range.start.saturating_add(1)),
            BytePos(ident.range.end.saturating_add(1)),
          ),
          module_info.module_ctxt,
        ),
        shorthand: ident.shorthand,
        is_class_expr_with_ident: ident.is_class_expr_with_ident,
      }),
  );

  let mut seen = HashSet::default();
  for ident in idents.into_iter().filter(|ident| {
    seen.insert((
      ident.id.sym.clone(),
      ident.id.ctxt,
      ident.id.span().real_lo(),
      ident.id.span().real_hi(),
      ident.shorthand,
      ident.is_class_expr_with_ident,
    ))
  }) {
    let ident = ident.clone();
    let is_global = ident.id.ctxt == module_info.global_ctxt;
    if is_global {
      module_info.global_scope_ident.push(ident.clone());
      module_info.all_used_names.insert(ident.id.sym.clone());
    }
    if ident.is_class_expr_with_ident {
      module_info.all_used_names.insert(ident.id.sym.clone());
      continue;
    }
    if ident.id.ctxt != module_info.module_ctxt {
      module_info.all_used_names.insert(ident.id.sym.clone());
    }
    module_info
      .binding_to_ref
      .entry((ident.id.sym.clone(), ident.id.ctxt))
      .or_default()
      .push(ident.clone());
    module_info.idents.push(ident);
  }

  for symbol in &module_info.generated_top_level_symbols {
    module_info
      .binding_to_ref
      .entry((symbol.placeholder.clone(), module_info.module_ctxt))
      .or_default();
  }
  if let Some(export_map) = &module_info.export_map {
    for export in export_map.values() {
      let export = Atom::from(export.as_str());
      if is_plain_identifier_name(export.as_ref())
        && ConcatenationScope::match_module_reference(export.as_ref()).is_none()
      {
        module_info
          .binding_to_ref
          .entry((export, module_info.module_ctxt))
          .or_default();
      }
    }
  }
}

fn apply_placeholder_replacements(source: &mut String, replacements: &PlaceholderReplacements<'_>) {
  if replacements.is_empty() || !source.contains(CONCATENATION_PLACEHOLDER_PREFIX) {
    return;
  }
  if let Some((end, value)) = find_placeholder_replacement(source, 0, replacements)
    && end == source.len()
  {
    source.clear();
    source.push_str(value);
    return;
  }
  let mut output = String::with_capacity(source.len());
  let mut cursor = 0;
  let mut changed = false;
  while let Some(offset) = source[cursor..].find(CONCATENATION_PLACEHOLDER_PREFIX) {
    let start = cursor + offset;
    output.push_str(&source[cursor..start]);
    if let Some((end, value)) = find_placeholder_replacement(source, start, replacements) {
      output.push_str(value);
      cursor = end;
      changed = true;
    } else {
      output.push_str(CONCATENATION_PLACEHOLDER_PREFIX);
      cursor = start + CONCATENATION_PLACEHOLDER_PREFIX.len();
    }
  }
  if !changed {
    return;
  }
  output.push_str(&source[cursor..]);
  *source = output;
}

fn find_placeholder_replacement<'a>(
  source: &str,
  start: usize,
  replacements: &'a PlaceholderReplacements<'_>,
) -> Option<(usize, &'a str)> {
  let candidate = &source[start..];
  if candidate.starts_with(MODULE_REFERENCE_PREFIX) {
    let len = candidate.find(MODULE_REFERENCE_SUFFIX)? + MODULE_REFERENCE_SUFFIX.len();
    return replacements
      .get(&candidate[..len])
      .map(|value| (start + len, value));
  }

  replacements
    .generated_symbols
    .iter()
    .find(|symbol| candidate.starts_with(symbol.placeholder.as_ref()))
    .and_then(|symbol| {
      replacements
        .internal_names
        .get(&symbol.placeholder)
        .map(|value| (start + symbol.placeholder.len(), value.as_ref()))
    })
}

fn apply_placeholder_replacements_to_source(
  source: ReplaceSource,
  replacements: &PlaceholderReplacements<'_>,
) -> ReplaceSource {
  if replacements.is_empty() {
    return source;
  }
  // JavaScript placeholders are emitted by dependency templates and live in
  // replacement contents, each of which is emitted as one rope chunk. Apply
  // their replacements in the rendered coordinate space by wrapping the
  // original ReplaceSource.
  if !source.replacements().is_empty() {
    let mut rendered_offset = 0usize;
    let mut rendered_replacements = Vec::new();
    source.rope(&mut |chunk| {
      let mut cursor = 0;
      while let Some(offset) = chunk[cursor..].find(CONCATENATION_PLACEHOLDER_PREFIX) {
        let start = cursor + offset;
        if let Some((end, value)) = find_placeholder_replacement(chunk, start, replacements) {
          rendered_replacements.push((
            (rendered_offset + start) as u32,
            (rendered_offset + end) as u32,
            value.to_string(),
          ));
          cursor = end;
        } else {
          cursor = start + CONCATENATION_PLACEHOLDER_PREFIX.len();
        }
      }
      rendered_offset += chunk.len();
    });
    if rendered_replacements.is_empty() {
      return source;
    }
    let mut rendered_source = ReplaceSource::new(source);
    for (start, end, value) in rendered_replacements {
      rendered_source.replace(start, end, value, None);
    }
    return rendered_source;
  }

  // Replacement-free sources come from generators such as JSON and assets,
  // where placeholders can be in the generated body.
  let mut source = source;
  let inner = source.inner().source().into_string_lossy();
  if !inner.contains(CONCATENATION_PLACEHOLDER_PREFIX) {
    return source;
  }

  // Keep the common no-placeholder path borrowed. Only materialize a String
  // when generated code put a placeholder directly into the inner source.
  let inner = inner.into_owned();
  let mut cursor = 0;
  while let Some(offset) = inner[cursor..].find(CONCATENATION_PLACEHOLDER_PREFIX) {
    let start = cursor + offset;
    if let Some((end, value)) = find_placeholder_replacement(&inner, start, replacements) {
      source.replace(start as u32, end as u32, value.to_string(), None);
      cursor = end;
    } else {
      cursor = start + CONCATENATION_PLACEHOLDER_PREFIX.len();
    }
  }
  source
}

pub(super) fn render_concatenated_module_source(
  info: &mut ConcatenatedModuleInfo,
  module_reference_replacements: &[(String, String)],
  rendered_init_fragments_hasher: Option<&mut RspackHasher>,
) -> BoxSource {
  let source = info.source.take().expect("should have source");
  let fragments = info.rendered_init_fragments.take();
  let replacements = PlaceholderReplacements {
    module_references: module_reference_replacements,
    generated_symbols: &info.generated_top_level_symbols,
    internal_names: &info.internal_names,
  };
  if fragments.is_none() && replacements.is_empty() {
    return source.boxed();
  }

  let rendered_source = apply_placeholder_replacements_to_source(source, &replacements);
  let rendered_source = rendered_source.boxed();

  if let Some(fragments) = fragments {
    let mut start = fragments.start;
    let mut end = fragments.end;
    apply_placeholder_replacements(&mut start, &replacements);
    apply_placeholder_replacements(&mut end, &replacements);
    if let Some(hasher) = rendered_init_fragments_hasher {
      info.module.hash(hasher);
      RenderedInitFragments::hash_parts(&start, &end, hasher);
    }
    ConcatSource::new([
      RawStringSource::from(start).boxed(),
      rendered_source,
      RawStringSource::from(end).boxed(),
    ])
    .boxed()
  } else {
    rendered_source
  }
}
