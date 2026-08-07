use std::cmp::Reverse;

use rspack_hash::{RspackHash, RspackHasher};
use rspack_sources::{BoxSource, ConcatSource, RawStringSource, ReplaceSource, Source, SourceExt};
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
  OriginalScopeIdentUpdate,
};
use crate::{
  ConcatenatedModuleIdent, ConcatenationScope, ConcatenationScopeIdentKind, DependencyRange,
  PendingConcatenationScopeInfo, RenderedInitFragments,
};

type PlaceholderReplacement<'a> = (&'a str, &'a str);

fn collect_placeholder_replacements<'a>(
  info: &'a ConcatenatedModuleInfo,
  module_references: &'a [(String, String)],
) -> Vec<PlaceholderReplacement<'a>> {
  let mut replacements =
    Vec::with_capacity(module_references.len() + info.generated_top_level_symbols.len());
  replacements.extend(
    module_references
      .iter()
      .map(|(placeholder, name)| (placeholder.as_str(), name.as_str())),
  );
  replacements.extend(
    info
      .generated_top_level_symbols
      .iter()
      .filter_map(|symbol| {
        info
          .internal_names
          .get(&symbol.placeholder)
          .map(|name| (symbol.placeholder.as_ref(), name.as_ref()))
      }),
  );
  replacements.sort_unstable_by_key(|(placeholder, _)| Reverse(placeholder.len()));
  replacements
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

fn scan_placeholder_replacements<'a>(
  source: &str,
  replacements: &[PlaceholderReplacement<'a>],
  mut on_match: impl FnMut(usize, usize, &'a str),
) {
  if replacements.is_empty() {
    return;
  }
  let mut cursor = 0;
  while let Some(offset) = source[cursor..].find(CONCATENATION_PLACEHOLDER_PREFIX) {
    let start = cursor + offset;
    if let Some(&(placeholder, value)) = replacements
      .iter()
      .find(|(placeholder, _)| source[start..].starts_with(*placeholder))
    {
      let end = start + placeholder.len();
      on_match(start, end, value);
      cursor = end;
    } else {
      cursor = start + CONCATENATION_PLACEHOLDER_PREFIX.len();
    }
  }
}

fn apply_placeholder_replacements(
  source: &mut String,
  replacements: &[PlaceholderReplacement<'_>],
) {
  let mut matches = Vec::new();
  scan_placeholder_replacements(source, replacements, |start, end, value| {
    matches.push((start, end, value));
  });
  if matches.is_empty() {
    return;
  }

  let mut output = String::with_capacity(source.len());
  let mut cursor = 0;
  for (start, end, value) in matches {
    output.push_str(&source[cursor..start]);
    output.push_str(value);
    cursor = end;
  }
  output.push_str(&source[cursor..]);
  *source = output;
}

fn apply_placeholder_replacements_to_source(
  source: ReplaceSource,
  replacements: &[PlaceholderReplacement<'_>],
) -> ReplaceSource {
  // Placeholders are emitted atomically by generators and dependency
  // templates, so each token is contained in one rope chunk. Apply all matches
  // in rendered coordinates through one outer ReplaceSource.
  let mut rendered_offset = 0usize;
  let mut rendered_replacements = Vec::new();
  source.rope(&mut |chunk| {
    scan_placeholder_replacements(chunk, replacements, |start, end, value| {
      rendered_replacements.push((
        (rendered_offset + start) as u32,
        (rendered_offset + end) as u32,
        value,
      ));
    });
    rendered_offset += chunk.len();
  });

  if rendered_replacements.is_empty() {
    return source;
  }
  let mut rendered_source = ReplaceSource::new(source);
  for (start, end, value) in rendered_replacements {
    rendered_source.replace(start, end, value.to_string(), None);
  }
  rendered_source
}

pub(super) fn render_concatenated_module_source(
  info: &mut ConcatenatedModuleInfo,
  module_reference_replacements: &[(String, String)],
  rendered_init_fragments_hasher: Option<&mut RspackHasher>,
) -> BoxSource {
  let source = info.source.take().expect("should have source");
  let fragments = info.rendered_init_fragments.take();
  let replacements = collect_placeholder_replacements(info, module_reference_replacements);
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
