//! Certified project-path literals.
//!
//! A loader chain can certify that some absolute path values in its output are visible project
//! paths and that nothing else in the output is derived from the checkout location. Each whole
//! string literal whose cooked value equals one of those paths, under `compiler.context`, is a
//! site. This module owns the site rules, the canonical spelling and the rendering:
//!
//! - The stored source, its map, dependency ranges and the build hash use canonical text, where
//!   every site is a root-free literal. A cold build and a cache restore in another checkout
//!   therefore store the same bytes.
//! - Every reader of the parsed AST sees the real value and raw text of each site, because the
//!   canonical AST is patched before any analysis runs. Direct readers may copy a site's whole
//!   value or compare it with a root-independent constant; derivations must use the evaluator
//!   so they bind the module. Source-copying createRequire handling is excluded. Diagnostics
//!   collected during the scan, including critical context-dependency warnings, force a real-text
//!   parse.
//! - Code generation and the public source view render every site as the current root's path.
//!   Canonical text is never emitted: a site under a replacement that already removed its range
//!   renders nothing, and any other site range is fully rewritten. A site that SWC wrote keeps
//!   SWC's spelling, so the output has the bytes of a build without this module.
//!
//! Anything this module cannot relocate keeps today's behavior: the real text is parsed, and the
//! module is bound to the root when a certified value occurs in it.

use std::hash::Hash;

use rspack_cacheable::{cacheable, cacheable_dyn, utils::PortablePath, with::As};
use rspack_core::{
  Compilation, DependencyCodeGeneration, DependencyCodeGenerationRef, DependencyRange,
  DependencyTemplate, DependencyTemplateType, PortableLocationShift, RuntimeSpec, SourcePosition,
  SourceView, TemplateContext, TemplateReplaceSource,
  rspack_sources::{
    BoxSource, MapOptions, RawStringSource, ReplaceSource, Source, SourceExt, SourceMap,
    SourceMapSource, WithoutOriginalOptions,
  },
};
use rspack_hash::RspackHasher;
use rspack_location::portable::{PathSpelling, quote_ascii};
use rspack_paths::{Utf8Component, Utf8Path, Utf8PathBuf};
use rustc_hash::{FxHashMap, FxHashSet};
use simd_utf16_len::utf16_len;
use swc_next_allocator::{Allocator, wtf8::Wtf8};
use swc_next_ecma_ast::{Ast, GetSpan, Lang, NodeId, NodeKindData, SourceType as SwcSourceType};
use swc_next_ecma_parser::{CommentMode, NoTokenParserConfig, Options, Parser};

/// Root spelling of canonical literals. Canonical literals are only parsed, stored and hashed.
const CANONICAL_ROOT: &str = "[portable-root]";

/// One certified literal.
struct Site {
  /// Byte range of the canonical literal in the canonical text.
  range: DependencyRange,
  /// The certified path, absolute in the current root.
  path: Utf8PathBuf,
  /// The real literal as the producer spelled it, quotes included.
  raw: String,
  /// How to write the path under any root.
  spelling: PathSpelling,
}

/// Canonical text for one parse, and the sites it holds.
pub(crate) struct Canonical {
  pub(crate) source: BoxSource,
  /// Sorted by range.
  sites: Vec<Site>,
}

pub(crate) enum Prepared {
  /// No certified literal: parse the real text as before.
  Unchanged,
  /// A certified value occurs where it cannot be relocated, or the source representation cannot
  /// be normalized. Parse the real text and bind the module to the root.
  Unsupported,
  Canonical(Canonical),
}

fn parser_options(source_type: SwcSourceType, lang: Lang) -> Options {
  Options {
    source_type,
    lang,
    preserve_parens: false,
    comments: CommentMode::Flat,
  }
}

/// Finds the sites in the real text with the JS parser and builds the canonical text.
///
/// `paths` are the deduplicated absolute values the loader chain certified. Paths outside
/// `context` are not sites; they stay fixed in the real text as today. `es5_strings` says that
/// a loader of the chain writes its literals by SWC's rule for ES5 and below.
pub(crate) fn prepare(
  source: &BoxSource,
  text: &str,
  paths: &[Utf8PathBuf],
  es5_strings: bool,
  context: &Utf8Path,
  source_type: SwcSourceType,
  lang: Lang,
) -> Prepared {
  let declared: FxHashMap<&str, (&Utf8PathBuf, String)> = paths
    .iter()
    .filter_map(|path| {
      let relative = path.strip_prefix(context).ok()?;
      let mut canonical = String::from(CANONICAL_ROOT);
      for component in relative.components() {
        let Utf8Component::Normal(name) = component else {
          return None;
        };
        canonical.push('/');
        canonical.push_str(name);
      }
      Some((path.as_str(), (path, quote_ascii(&canonical))))
    })
    .collect();
  // A literal can only cook to a declared value if the value occurs verbatim or an escape occurs.
  if declared.is_empty()
    || (memchr::memchr(b'\\', text.as_bytes()).is_none()
      && !declared
        .keys()
        .any(|value| memchr::memmem::find(text.as_bytes(), value.as_bytes()).is_some()))
  {
    return Prepared::Unchanged;
  }

  // The real text is parsed once to locate whole literals by their cooked value, whatever
  // quotes or escapes the producer chose.
  let allocator = Allocator::new();
  let parsed = Parser::init(
    &allocator,
    text,
    parser_options(source_type, lang),
    NoTokenParserConfig,
  )
  .parse();
  if !parsed.diagnostics.is_empty() {
    // The ordinary parse reports the diagnostics.
    return Prepared::Unchanged;
  }
  // SAFETY: This AST is only read, and every handle below was taken from it.
  let ast = unsafe { parsed.ast.ast() };
  let mut not_relocatable = FxHashSet::default();
  let mut candidates = Vec::new();
  let mut unsupported = false;
  for index in 0..ast.node_count() {
    let node = NodeId::from_raw_unchecked(index as u32);
    match ast.kind_data(node) {
      NodeKindData::StringLiteral(literal) => {
        if let Some(value) = ast.get_wtf8(literal.value(ast)).as_str()
          && let Some(entry) = declared.get(value)
        {
          candidates.push((
            node,
            literal.span(ast),
            entry,
            ast.get_utf8(literal.raw(ast)),
          ));
        }
      }
      // A template quasi has no literal of its own to replace.
      NodeKindData::TemplateElement(element)
        if !element.is_cooked_undefined(ast)
          && ast
            .get_wtf8(element.cooked(ast))
            .as_str()
            .is_some_and(|value| declared.contains_key(value)) =>
      {
        unsupported = true;
      }
      // JSX attribute strings have no escapes, so the JSON spelling would change their value.
      NodeKindData::JsxAttribute(attribute) => {
        if let Some(value) = attribute.value(ast) {
          not_relocatable.insert(value.node_id());
        }
      }
      // A directive's text is read separately from its literal.
      NodeKindData::Directive(directive) => {
        not_relocatable.insert(directive.expression(ast).node_id());
      }
      _ => {}
    }
  }
  if unsupported
    || candidates
      .iter()
      .any(|(node, ..)| not_relocatable.contains(node))
  {
    return Prepared::Unsupported;
  }
  if candidates.is_empty() {
    return Prepared::Unchanged;
  }
  candidates.sort_unstable_by_key(|(_, span, ..)| span.start);

  let leaf = source.as_ref().as_any();
  let original_map_source = leaf.downcast_ref::<SourceMapSource>();
  if original_map_source.is_none() && !leaf.is::<RawStringSource>() {
    // `OriginalSource` maps to its own text, so its map would hold canonical text.
    return Prepared::Unsupported;
  }

  let mut replaced = ReplaceSource::new(source.clone());
  let mut sites = Vec::with_capacity(candidates.len());
  let mut delta: i64 = 0;
  for (_, span, (path, canonical), raw) in candidates {
    replaced.replace(span.start, span.end, canonical.clone(), None);
    let start = (i64::from(span.start) + delta) as u32;
    let end = start + canonical.len() as u32;
    delta += canonical.len() as i64 - i64::from(span.end - span.start);
    sites.push(Site {
      range: DependencyRange::new(start, end),
      path: (*path).clone(),
      raw: raw.to_owned(),
      spelling: PathSpelling::of(raw, path.as_str(), es5_strings),
    });
  }
  let code = replaced.source().into_string_lossy().into_owned();
  // Flatten into owned leaves so no root-bearing source stays behind the canonical one.
  let canonical = if let Some(original) = original_map_source {
    match replaced.map(&MapOptions::default()) {
      Some(map) => match SourceMap::from_json(map.to_json()) {
        Ok(source_map) => SourceMapSource::new(WithoutOriginalOptions {
          value: code,
          name: original.name().to_owned(),
          source_map,
        })
        .boxed(),
        Err(_) => return Prepared::Unsupported,
      },
      None => RawStringSource::from(code).boxed(),
    }
  } else {
    RawStringSource::from(code).boxed()
  };
  Prepared::Canonical(Canonical {
    source: canonical,
    sites,
  })
}

impl Canonical {
  /// Gives each site literal in the canonical AST the real value and raw text, so every AST
  /// reader, raw or through the evaluator, sees what the loader produced. Returns `false` when
  /// the canonical text did not parse with exactly one string literal at each site.
  pub(crate) fn restore_real_literals(&self, ast: &mut Ast<'_>) -> bool {
    let mut restored = vec![false; self.sites.len()];
    for node_index in 0..ast.node_count() {
      let node = NodeId::from_raw_unchecked(node_index as u32);
      let NodeKindData::StringLiteral(literal) = ast.kind_data(node) else {
        continue;
      };
      let span = literal.span(ast);
      let Ok(index) = self
        .sites
        .binary_search_by_key(&span.start, |site| site.range.start)
      else {
        continue;
      };
      let site = &self.sites[index];
      if span.end != site.range.end || std::mem::replace(&mut restored[index], true) {
        return false;
      }
      let value = ast.add_wtf8(Wtf8::from_str(site.path.as_str()));
      let raw = ast.add_utf8(&site.raw);
      literal.set_value(ast, value);
      literal.set_raw(ast, raw);
    }
    restored.into_iter().all(|restored| restored)
  }

  /// Sorted canonical site ranges.
  pub(crate) fn site_ranges(&self) -> Vec<DependencyRange> {
    self.sites.iter().map(|site| site.range).collect()
  }

  /// Compute canonical positions once, without rendering another full source for locations.
  pub(crate) fn location_shifts(&self, source: &str) -> Vec<PortableLocationShift> {
    let mut offset = 0;
    let mut position = SourcePosition { line: 1, column: 1 };
    self
      .sites
      .iter()
      .map(|site| {
        let segment = &source[offset..site.range.start as usize];
        let mut newlines = memchr::memchr_iter(b'\n', segment.as_bytes());
        if let Some(mut last) = newlines.next() {
          position.line += 1;
          for newline in newlines {
            position.line += 1;
            last = newline;
          }
          position.column = (utf16_len(&segment[last + 1..]) + 1) as u32;
        } else {
          position.column += utf16_len(segment) as u32;
        }
        offset = site.range.start as usize;
        PortableLocationShift {
          start: position,
          canonical_width: site.range.end - site.range.start,
          path: site.path.clone(),
          spelling: site.spelling,
        }
      })
      .collect()
  }

  pub(crate) fn into_dependencies(self) -> impl Iterator<Item = DependencyCodeGenerationRef> {
    self.sites.into_iter().map(|site| {
      std::sync::Arc::new(PortablePathDependency {
        range: site.range,
        path: site.path,
        spelling: site.spelling,
      }) as DependencyCodeGenerationRef
    })
  }
}

/// Whether a dependency range cuts into a site instead of enclosing it or staying apart. Such a
/// range was computed inside the canonical literal, so the canonical scan is not trusted.
pub(crate) fn cuts_site(sites: &[DependencyRange], range: DependencyRange) -> bool {
  let first = sites.partition_point(|site| site.end <= range.start);
  sites[first..]
    .iter()
    .take_while(|site| site.start < range.end)
    .any(|site| {
      if range.start == range.end {
        // An insertion strictly inside the literal.
        site.start < range.start
      } else {
        // Overlapping, so it must enclose the whole literal.
        !(range.start <= site.start && range.end >= site.end)
      }
    })
}

/// A certified literal, stored at its canonical range and rendered for the current root.
#[cacheable]
#[derive(Debug)]
pub struct PortablePathDependency {
  range: DependencyRange,
  /// Stored relative to the portable root, restored under the reader's root.
  #[cacheable(with=As<PortablePath>)]
  path: Utf8PathBuf,
  spelling: PathSpelling,
}

impl PortablePathDependency {
  /// Renders the real literal unless a replacement already removed the whole canonical one.
  pub(crate) fn render(&self, source: &mut TemplateReplaceSource) {
    let DependencyRange { start, end } = self.range;
    // A replacement that already removed the whole literal was computed from the real value.
    let covered = source
      .replacements()
      .iter()
      .take_while(|replacement| replacement.start() <= start)
      .any(|replacement| replacement.start() < replacement.end() && replacement.end() >= end);
    if !covered {
      source.replace(start, end, self.spelling.quote(self.path.as_str()), None);
    }
  }
}

#[cacheable_dyn]
impl DependencyCodeGeneration for PortablePathDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(PortablePathDependencyTemplate::template_type())
  }

  fn update_hash(
    &self,
    hasher: &mut RspackHasher,
    _compilation: &Compilation,
    _runtime: Option<&RuntimeSpec>,
  ) {
    // The module hash follows the rendered bytes; the build hash covers the canonical source.
    self.range.hash(hasher);
    self.path.as_str().hash(hasher);
    self.spelling.hash(hasher);
  }
}

#[cacheable]
#[derive(Debug, Default)]
pub struct PortablePathDependencyTemplate;

impl PortablePathDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Custom("PortablePathDependency")
  }
}

impl DependencyTemplate for PortablePathDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    _code_generatable_context: &mut TemplateContext,
  ) {
    dep
      .as_any()
      .downcast_ref::<PortablePathDependency>()
      .expect("PortablePathDependencyTemplate should be used for PortablePathDependency")
      .render(source);
  }
}

/// Splits sites out of the presentational dependencies, so the JS generator renders them after
/// every other replacement, blocks included.
pub(crate) fn as_site(dependency: &DependencyCodeGenerationRef) -> Option<&PortablePathDependency> {
  dependency.as_any().downcast_ref::<PortablePathDependency>()
}

/// The public view of a stored canonical source: every site rendered for the current root.
pub(crate) fn render_view(
  source: &BoxSource,
  dependencies: &[DependencyCodeGenerationRef],
) -> Option<SourceView> {
  let mut sites = dependencies.iter().filter_map(as_site).peekable();
  sites.peek()?;
  Some(render_sites(
    source,
    sites.map(|site| (site.range, site.spelling.quote(site.path.as_str()))),
  ))
}

fn render_sites(
  source: &BoxSource,
  sites: impl Iterator<Item = (DependencyRange, String)>,
) -> SourceView {
  let mut view = ReplaceSource::new(source.clone());
  let mut replacements = Vec::new();
  for (range, rendered) in sites {
    replacements.push((range, rendered.len() as u32));
    view.replace(range.start, range.end, rendered, None);
  }
  SourceView::new(view.boxed(), replacements)
}
