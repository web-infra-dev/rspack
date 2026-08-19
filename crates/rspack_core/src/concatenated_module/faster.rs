use std::borrow::Cow;

use rspack_hash::{RspackHash, RspackHasher};
use rspack_sources::{BoxSource, ConcatSource, RawStringSource, ReplaceSource, SourceExt};
use rspack_util::SpanExt;
use rustc_hash::FxHashSet as HashSet;
use smallvec::SmallVec;
use swc_core::{
  atoms::Atom,
  common::{BytePos, Span, Spanned, SyntaxContext},
  ecma::visit::swc_ecma_ast,
};

use super::{
  CONCATENATION_PLACEHOLDER_PREFIX, ConcatenatedModuleInfo, FasterModuleConcatenationInfo,
  GENERATED_TOP_LEVEL_SYMBOL_PREFIX, GeneratedTopLevelSymbol, GeneratedTopLevelSymbolTarget,
  MODULE_REFERENCE_PLACEHOLDER_PREFIX, MODULE_REFERENCE_SUFFIX, OriginalScopeIdentUpdate,
};
use crate::{
  ConcatenatedModuleIdent, ConcatenationScope, ConcatenationScopeIdentKind, DependencyRange,
  PendingConcatenationScopeInfo, RenderedInitFragments,
};

type SeenIdent = (Atom, SyntaxContext, DependencyRange, bool, bool);

enum SeenIdents {
  Inline(SmallVec<[SeenIdent; 8]>),
  Heap(HashSet<SeenIdent>),
}

impl Default for SeenIdents {
  fn default() -> Self {
    Self::Inline(SmallVec::new())
  }
}

impl SeenIdents {
  fn insert(&mut self, ident: SeenIdent) -> bool {
    match self {
      Self::Inline(idents) => {
        if idents.contains(&ident) {
          return false;
        }
        if idents.len() < idents.inline_size() {
          idents.push(ident);
          return true;
        }

        let mut ident_set = HashSet::with_capacity_and_hasher(idents.len() * 2, Default::default());
        ident_set.extend(idents.drain(..));
        ident_set.insert(ident);
        *self = Self::Heap(ident_set);
        true
      }
      Self::Heap(idents) => idents.insert(ident),
    }
  }
}

struct PlaceholderReplacements<'a> {
  module_references: &'a [(String, String)],
  generated_symbols: &'a [GeneratedTopLevelSymbol],
  internal_names: &'a rustc_hash::FxHashMap<Atom, Atom>,
}

impl PlaceholderReplacements<'_> {
  fn is_empty(&self) -> bool {
    self.module_references.is_empty() && self.generated_symbols.is_empty()
  }

  fn get<'a>(&'a self, placeholder: &str) -> Option<&'a str> {
    if let Some(index) = placeholder
      .strip_prefix(MODULE_REFERENCE_PLACEHOLDER_PREFIX)
      .and_then(|value| value.strip_suffix(MODULE_REFERENCE_SUFFIX))
      .and_then(|value| value.parse().ok())
    {
      return self.get_module_reference(index, placeholder);
    }
    if let Some(index) = placeholder
      .strip_prefix(GENERATED_TOP_LEVEL_SYMBOL_PREFIX)
      .and_then(|value| value.strip_suffix("__"))
      .and_then(|value| value.parse().ok())
    {
      return self.get_generated_symbol(index, placeholder);
    }
    None
  }

  fn get_module_reference<'a>(&'a self, index: usize, placeholder: &str) -> Option<&'a str> {
    self
      .module_references
      .get(index)
      .filter(|(candidate, _)| candidate == placeholder)
      .or_else(|| {
        self
          .module_references
          .iter()
          .find(|(candidate, _)| candidate == placeholder)
      })
      .map(|(_, name)| name.as_str())
  }

  fn get_generated_symbol<'a>(&'a self, index: usize, placeholder: &str) -> Option<&'a str> {
    let symbol = self
      .generated_symbols
      .get(index)
      .filter(|symbol| symbol.placeholder == placeholder)?;
    let binding = match symbol.target {
      GeneratedTopLevelSymbolTarget::New => &symbol.placeholder,
      GeneratedTopLevelSymbolTarget::Rebind { .. } => symbol.resolved_binding.as_ref()?,
    };
    self.internal_names.get(binding).map(Atom::as_str)
  }
}

fn parse_decimal_prefix(value: &[u8]) -> Option<(usize, usize)> {
  let mut index = 0usize;
  let mut len = 0;
  for &byte in value {
    if !byte.is_ascii_digit() {
      break;
    }
    index = index.checked_mul(10)?.checked_add((byte - b'0') as usize)?;
    len += 1;
  }
  (len != 0).then_some((index, len))
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

fn add_ident_to_module_info(
  module_info: &mut ConcatenatedModuleInfo,
  ident: ConcatenatedModuleIdent,
  seen: &mut SeenIdents,
) {
  let span = ident.id.span();
  if !seen.insert((
    ident.id.sym.clone(),
    ident.id.ctxt,
    DependencyRange::new(span.real_lo(), span.real_hi()),
    ident.shorthand,
    ident.is_class_expr_with_ident,
  )) {
    return;
  }

  let is_global = ident.id.ctxt == module_info.global_ctxt;
  if is_global {
    module_info.global_scope_ident.push(ident.clone());
    module_info.all_used_names.insert(ident.id.sym.clone());
  }
  if ident.is_class_expr_with_ident {
    module_info.all_used_names.insert(ident.id.sym);
    return;
  }
  if !is_global && ident.id.ctxt != module_info.module_ctxt {
    module_info.all_used_names.insert(ident.id.sym.clone());
  }
  module_info
    .binding_to_ref
    .entry((ident.id.sym.clone(), ident.id.ctxt))
    .or_default()
    .push(ident.clone());
  module_info.idents.push(ident);
}

pub(crate) fn populate_info_from_pending(
  pending: &PendingConcatenationScopeInfo,
  original_source: &str,
  module_info: &mut ConcatenatedModuleInfo,
  faster_info: &mut FasterModuleConcatenationInfo,
) {
  let (module_ctxt, global_ctxt, pending_idents, canonical_names) = match pending {
    PendingConcatenationScopeInfo::Analyzed(info) => (
      info.module_ctxt,
      info.global_ctxt,
      info.idents.as_slice(),
      info.canonical_names.as_slice(),
    ),
    PendingConcatenationScopeInfo::Generated => (0, 0, &[][..], &[][..]),
  };
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
  module_info.module_ctxt = SyntaxContext::from_u32(module_ctxt);
  module_info.global_ctxt = SyntaxContext::from_u32(global_ctxt);
  module_info.idents.clear();
  module_info.global_scope_ident.clear();
  module_info.binding_to_ref.clear();
  module_info.all_used_names.clear();
  module_info.idents.reserve(
    pending_idents
      .len()
      .saturating_add(faster_info.added_scope_idents.len()),
  );
  module_info.global_scope_ident.reserve(pending_idents.len());
  module_info.binding_to_ref.reserve(
    pending_idents
      .len()
      .saturating_add(faster_info.added_scope_idents.len()),
  );
  module_info.all_used_names.reserve(
    pending_idents
      .len()
      .saturating_add(faster_info.added_used_names.len()),
  );
  module_info
    .all_used_names
    .extend(faster_info.added_used_names.drain(..));

  let mut removed_ranges = SmallVec::<[DependencyRange; 4]>::new();
  let mut non_shorthand_ranges = SmallVec::<[DependencyRange; 4]>::new();
  for update in &faster_info.original_scope_ident_updates {
    match update {
      OriginalScopeIdentUpdate::Remove(range) => removed_ranges.push(*range),
      OriginalScopeIdentUpdate::NonShorthand(range) => non_shorthand_ranges.push(*range),
    }
  }

  let mut seen = SeenIdents::default();
  for pending_ident in pending_idents {
    let symbol = canonical_names
      .iter()
      .find(|canonical_name| canonical_name.range == pending_ident.range)
      .map_or_else(
        || symbol_from_range(pending_ident.range),
        |canonical_name| canonical_name.name.clone(),
      );
    if pending_ident.kind == ConcatenationScopeIdentKind::UsedName {
      module_info.all_used_names.insert(symbol);
      continue;
    }

    if removed_ranges
      .iter()
      .any(|range| range.start <= pending_ident.range.start && pending_ident.range.end <= range.end)
    {
      if pending_ident.kind == ConcatenationScopeIdentKind::Global {
        module_info.all_used_names.insert(symbol);
      }
      continue;
    }

    let ctxt = match pending_ident.kind {
      ConcatenationScopeIdentKind::TopLevel => module_info.module_ctxt,
      ConcatenationScopeIdentKind::Global => module_info.global_ctxt,
      ConcatenationScopeIdentKind::UsedName => unreachable!(),
    };
    add_ident_to_module_info(
      module_info,
      ConcatenatedModuleIdent {
        id: swc_ecma_ast::Ident::new(
          symbol,
          Span::new(
            BytePos(pending_ident.range.start.saturating_add(1)),
            BytePos(pending_ident.range.end.saturating_add(1)),
          ),
          ctxt,
        ),
        shorthand: pending_ident.shorthand && !non_shorthand_ranges.contains(&pending_ident.range),
        is_class_expr_with_ident: false,
      },
      &mut seen,
    );
  }

  for ident in faster_info.added_scope_idents.drain(..) {
    add_ident_to_module_info(
      module_info,
      ConcatenatedModuleIdent {
        id: swc_ecma_ast::Ident::new(
          ident.symbol,
          Span::new(
            BytePos(ident.range.start.saturating_add(1)),
            BytePos(ident.range.end.saturating_add(1)),
          ),
          module_info.module_ctxt,
        ),
        shorthand: ident.shorthand,
        is_class_expr_with_ident: ident.is_class_expr_with_ident,
      },
      &mut seen,
    );
  }

  for symbol in &mut module_info.generated_top_level_symbols {
    let binding = match symbol.target {
      GeneratedTopLevelSymbolTarget::New => symbol.placeholder.clone(),
      GeneratedTopLevelSymbolTarget::Rebind { original_range } => {
        let original_ident = pending_idents
          .iter()
          .find(|ident| {
            ident.kind == ConcatenationScopeIdentKind::TopLevel && ident.range == original_range
          })
          .unwrap_or_else(|| {
            panic!(
              "rebound concatenation symbol range {}..{} should refer to a make-time top-level identifier",
              original_range.start, original_range.end
            )
          });
        let binding = canonical_names
          .iter()
          .find(|canonical_name| canonical_name.range == original_ident.range)
          .map_or_else(
            || symbol_from_range(original_range),
            |canonical_name| canonical_name.name.clone(),
          );
        symbol.resolved_binding = Some(binding.clone());
        binding
      }
    };
    module_info
      .binding_to_ref
      .entry((binding, module_info.module_ctxt))
      .or_default();
  }
  if let Some(export_map) = &module_info.export_map {
    for export in export_map.values() {
      let export = Atom::from(export.as_str());
      if is_plain_identifier_name(export.as_ref())
        && !module_info.module_references.contains_key(export.as_ref())
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

fn find_placeholder_replacement<'a>(
  source: &str,
  mut cursor: usize,
  replacements: &'a PlaceholderReplacements<'_>,
) -> Option<(usize, usize, &'a str)> {
  let source_bytes = source.as_bytes();
  while let Some(offset) = memchr::memchr(b'_', &source_bytes[cursor..]) {
    let start = cursor + offset;
    let candidate = &source[start..];
    if let Some(value) = candidate.strip_prefix(MODULE_REFERENCE_PLACEHOLDER_PREFIX)
      && let Some((index, index_len)) = parse_decimal_prefix(value.as_bytes())
      && value[index_len..].starts_with(MODULE_REFERENCE_SUFFIX)
    {
      let end = start
        + MODULE_REFERENCE_PLACEHOLDER_PREFIX.len()
        + index_len
        + MODULE_REFERENCE_SUFFIX.len();
      if let Some(value) = replacements.get_module_reference(index, &source[start..end]) {
        return Some((start, end, value));
      }
    } else if let Some(value) = candidate.strip_prefix(GENERATED_TOP_LEVEL_SYMBOL_PREFIX)
      && let Some((index, index_len)) = parse_decimal_prefix(value.as_bytes())
      && value[index_len..].starts_with("__")
    {
      let end = start + GENERATED_TOP_LEVEL_SYMBOL_PREFIX.len() + index_len + 2;
      if let Some(value) = replacements.get_generated_symbol(index, &source[start..end]) {
        return Some((start, end, value));
      }
    }
    cursor = start + 1;
  }
  None
}

fn replace_placeholders(
  source: &str,
  replacements: &PlaceholderReplacements<'_>,
) -> Option<String> {
  if replacements.is_empty() {
    return None;
  }

  let (mut start, mut end, mut value) = find_placeholder_replacement(source, 0, replacements)?;

  let mut output = String::with_capacity(source.len());
  let mut cursor = 0;
  loop {
    output.push_str(&source[cursor..start]);
    output.push_str(value);
    cursor = end;
    let Some(next) = find_placeholder_replacement(source, cursor, replacements) else {
      break;
    };
    (start, end, value) = next;
  }
  output.push_str(&source[cursor..]);
  Some(output)
}

fn apply_placeholder_replacements(source: &mut String, replacements: &PlaceholderReplacements<'_>) {
  if replacements.is_empty() {
    return;
  }
  if source.starts_with(CONCATENATION_PLACEHOLDER_PREFIX)
    && let Some(value) = replacements.get(source)
  {
    source.clear();
    source.push_str(value);
    return;
  }
  if let Some(output) = replace_placeholders(source, replacements) {
    *source = output;
  }
}

fn apply_placeholder_replacements_to_source(
  mut source: ReplaceSource,
  replacements: &PlaceholderReplacements<'_>,
) -> ReplaceSource {
  if replacements.is_empty() {
    return source;
  }
  // Concatenation placeholders are generated dynamically, so dependency
  // templates and generators store them in owned replacement contents.
  source.rewrite_replacement_contents(|content| match content {
    Cow::Owned(content) => apply_placeholder_replacements(content, replacements),
    Cow::Borrowed(content) => debug_assert!(
      find_placeholder_replacement(content, 0, replacements).is_none(),
      "concatenation placeholders should be stored in owned replacement contents"
    ),
  });
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
