//! Public dependency result types and their typed side-table storage.

use std::{
  fmt::Display,
  hash::{Hash, Hasher},
  marker::PhantomData,
  ops::Index,
};

use crate::lexer::Pos;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Range {
  pub start: Pos,
  pub end: Pos,
}

impl Range {
  pub fn new(start: Pos, end: Pos) -> Self {
    Self { start, end }
  }
}

/// A half-open index range into one of [`DependencyContext`]'s flat payload
/// vectors. Keeping list payloads out of [`Dependency`] prevents rare, large
/// variants from determining the size of every dependency value. `T` binds the
/// range to its payload element type without adding runtime storage.
#[derive(Debug, Hash, PartialEq, Eq)]
pub struct DependencyListRange<T> {
  start: u32,
  end: u32,
  marker: PhantomData<fn() -> T>,
}

impl<T> Copy for DependencyListRange<T> {}

impl<T> Clone for DependencyListRange<T> {
  fn clone(&self) -> Self {
    *self
  }
}

impl<T> DependencyListRange<T> {
  pub(crate) fn from_bounds(start: usize, end: usize) -> Self {
    assert!(start <= end, "dependency list range is reversed");
    assert!(
      u32::try_from(end).is_ok(),
      "dependency list storage is too large"
    );
    Self {
      start: start as u32,
      end: end as u32,
      marker: PhantomData,
    }
  }

  pub(crate) fn as_usize_range(self) -> std::ops::Range<usize> {
    self.start as usize..self.end as usize
  }

  pub fn start(self) -> u32 {
    self.start
  }

  pub fn end(self) -> u32 {
    self.end
  }

  pub fn len(self) -> usize {
    (self.end - self.start) as usize
  }

  pub fn is_empty(self) -> bool {
    self.start == self.end
  }
}

/// A typed index into one of [`DependencyContext`]'s side tables.
///
/// `T` identifies the target table without adding runtime storage. Keeping
/// rare payloads behind an index prevents them from determining the size of
/// every [`Dependency`] value.
#[derive(Debug, Hash, PartialEq, Eq)]
pub struct DependencyIndex<T> {
  index: u32,
  marker: PhantomData<fn() -> T>,
}

impl<T> Copy for DependencyIndex<T> {}

impl<T> Clone for DependencyIndex<T> {
  fn clone(&self) -> Self {
    *self
  }
}

impl<T> DependencyIndex<T> {
  pub(crate) fn from_index(index: usize) -> Self {
    assert!(
      u32::try_from(index).is_ok(),
      "dependency side table is too large"
    );
    Self {
      index: index as u32,
      marker: PhantomData,
    }
  }

  pub(crate) fn as_usize(self) -> usize {
    self.index as usize
  }

  pub fn index(self) -> u32 {
    self.index
  }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Mode {
  Local,
  Global,
  Pure,
  Css,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct ValueAtRuleImportItem<'s> {
  local_name: &'s str,
  import_name: &'s str,
}

impl<'s> ValueAtRuleImportItem<'s> {
  pub(crate) fn new(local_name: &'s str, import_name: &'s str) -> Self {
    Self {
      local_name,
      import_name,
    }
  }

  pub fn local_name(&self) -> &'s str {
    self.local_name
  }

  pub fn import_name(&self) -> &'s str {
    self.import_name
  }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct ImportAttributes<'s> {
  layer: Option<&'s str>,
  supports: Option<&'s str>,
  media: Option<&'s str>,
}

impl<'s> ImportAttributes<'s> {
  pub(crate) fn new(
    layer: Option<&'s str>,
    supports: Option<&'s str>,
    media: Option<&'s str>,
  ) -> Self {
    Self {
      layer,
      supports,
      media,
    }
  }

  pub fn layer(&self) -> Option<&'s str> {
    self.layer
  }

  pub fn supports(&self) -> Option<&'s str> {
    self.supports
  }

  pub fn media(&self) -> Option<&'s str> {
    self.media
  }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct MagicComments<'s> {
  value: &'s str,
  range: Range,
}

impl<'s> MagicComments<'s> {
  pub(crate) fn new(value: &'s str, range: Range) -> Self {
    Self { value, range }
  }

  pub fn value(&self) -> &'s str {
    self.value
  }

  pub fn range(&self) -> Range {
    self.range
  }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Dependency<'s> {
  Url {
    request: &'s str,
    range: Range,
    kind: UrlRangeKind,
    magic_comments: Option<MagicComments<'s>>,
  },
  Import {
    request: &'s str,
    range: Range,
    attributes: DependencyIndex<ImportAttributes<'s>>,
    magic_comments: Option<MagicComments<'s>>,
  },
  ICSSImportUrl {
    name: &'s str,
    range: Range,
    name_range: Range,
  },
  Replace {
    content: &'s str,
    range: Range,
  },
  Charset {
    value: &'s str,
    range: Range,
  },
  LocalClass {
    name: &'s str,
    range: Range,
    explicit: bool,
  },
  LocalId {
    name: &'s str,
    range: Range,
    explicit: bool,
  },
  LocalVar {
    name: &'s str,
    range: Range,
    from: Option<&'s str>,
    /// Whether `from` is the unquoted `global` keyword rather than a request.
    from_is_global: bool,
  },
  LocalVarDecl {
    name: &'s str,
    range: Range,
  },
  LocalPropertyDecl {
    name: &'s str,
    range: Range,
  },
  LocalKeyframes {
    name: &'s str,
    range: Range,
  },
  LocalKeyframesDecl {
    name: &'s str,
    range: Range,
  },
  LocalCounterStyle {
    name: &'s str,
    range: Range,
  },
  LocalCounterStyleDecl {
    name: &'s str,
    range: Range,
  },
  LocalFontPalette {
    name: &'s str,
    range: Range,
  },
  LocalFontPaletteDecl {
    name: &'s str,
    range: Range,
  },
  LocalContainer {
    name: &'s str,
    range: Range,
  },
  LocalContainerDecl {
    name: &'s str,
    range: Range,
  },
  LocalFunction {
    name: &'s str,
    range: Range,
  },
  LocalFunctionDecl {
    name: &'s str,
    range: Range,
  },
  LocalGrid {
    name: &'s str,
    range: Range,
  },
  LocalGridDecl {
    name: &'s str,
    range: Range,
  },
  Composes {
    local_classes: DependencyListRange<&'s str>,
    names: DependencyListRange<&'s str>,
    from: Option<&'s str>,
    /// Whether `from` is the unquoted `global` keyword rather than a request.
    from_is_global: bool,
    range: Range,
  },
  ICSSImportFrom {
    path: &'s str,
  },
  ICSSImportValue {
    prop: &'s str,
    value: &'s str,
  },
  ICSSExportValue {
    prop: &'s str,
    value: &'s str,
  },
  ICSSSymbol {
    name: &'s str,
    range: Range,
  },
}

/// Owns dependencies and the side-table payloads referenced by them.
///
/// `Dependency` values are intentionally cheap to move. Rare or variable-length
/// data is appended to context vectors and referenced through [`DependencyIndex`]
/// or [`DependencyListRange`]. Consumers should resolve those handles through
/// this type rather than retaining raw indices independently.
#[derive(Debug, Clone, Default)]
pub struct DependencyContext<'s> {
  dependencies: Vec<Dependency<'s>>,
  dashed_ident_occurrences: Vec<Range>,
  import_attributes: Vec<ImportAttributes<'s>>,
  composes_local_classes: Vec<&'s str>,
  composes_names: Vec<&'s str>,
  value_at_rule_import_items: Vec<ValueAtRuleImportItem<'s>>,
}

impl<'s> DependencyContext<'s> {
  pub fn new() -> Self {
    Self::default()
  }

  pub(crate) fn reserve_estimated_capacity(&mut self, input_len: usize, mode: Mode) {
    let estimate = DependencyContextCapacity::estimate(input_len, mode);
    reserve_estimated_capacity(&mut self.dependencies, estimate.dependencies);
    reserve_estimated_capacity(
      &mut self.import_attributes,
      estimate.import_attributes as usize,
    );
    reserve_estimated_capacity(
      &mut self.composes_local_classes,
      estimate.composes_local_classes as usize,
    );
    reserve_estimated_capacity(&mut self.composes_names, estimate.composes_names as usize);
    reserve_estimated_capacity(
      &mut self.value_at_rule_import_items,
      estimate.value_at_rule_import_items as usize,
    );
  }

  pub fn len(&self) -> usize {
    self.dependencies.len()
  }

  pub fn is_empty(&self) -> bool {
    self.dependencies.is_empty()
  }

  pub fn get(&self, index: usize) -> Option<&Dependency<'s>> {
    self.dependencies.get(index)
  }

  pub fn iter(&self) -> std::slice::Iter<'_, Dependency<'s>> {
    self.dependencies.iter()
  }

  pub fn dependencies(&self) -> &[Dependency<'s>] {
    &self.dependencies
  }

  pub fn dashed_ident_name_ranges(&self) -> &[Range] {
    &self.dashed_ident_occurrences
  }

  pub(crate) fn set_dashed_ident_occurrences(&mut self, occurrences: Vec<Range>) {
    self.dashed_ident_occurrences = occurrences;
  }

  pub(crate) fn estimated_dashed_ident_capacity(input_len: usize, mode: Mode) -> usize {
    DependencyContextCapacity::estimate(input_len, mode).dashed_ident_occurrences
  }

  pub fn import_attributes(
    &self,
    index: DependencyIndex<ImportAttributes<'s>>,
  ) -> &ImportAttributes<'s> {
    &self.import_attributes[index.as_usize()]
  }

  pub fn composes_local_classes(&self, range: DependencyListRange<&'s str>) -> &[&'s str] {
    &self.composes_local_classes[range.as_usize_range()]
  }

  pub fn composes_names(&self, range: DependencyListRange<&'s str>) -> &[&'s str] {
    &self.composes_names[range.as_usize_range()]
  }

  pub fn value_at_rule_import_items(&self) -> &[ValueAtRuleImportItem<'s>] {
    &self.value_at_rule_import_items
  }

  pub(crate) fn push_dependency(&mut self, dependency: Dependency<'s>) {
    self.dependencies.push(dependency);
  }

  pub(crate) fn push_import(
    &mut self,
    request: &'s str,
    range: Range,
    layer: Option<&'s str>,
    supports: Option<&'s str>,
    media: Option<&'s str>,
    magic_comments: Option<MagicComments<'s>>,
  ) {
    let attributes = DependencyIndex::from_index(self.import_attributes.len());
    self
      .import_attributes
      .push(ImportAttributes::new(layer, supports, media));
    self.dependencies.push(Dependency::Import {
      request,
      range,
      attributes,
      magic_comments,
    });
  }

  pub(crate) fn push_value_at_rule_import_item(&mut self, item: ValueAtRuleImportItem<'s>) {
    self.value_at_rule_import_items.push(item);
  }

  pub(crate) fn value_at_rule_import_item(&self, index: usize) -> ValueAtRuleImportItem<'s> {
    self.value_at_rule_import_items[index]
  }

  pub(crate) fn value_at_rule_import_items_checkpoint(&self) -> usize {
    self.value_at_rule_import_items.len()
  }

  pub(crate) fn truncate_value_at_rule_import_items(&mut self, checkpoint: usize) {
    self.value_at_rule_import_items.truncate(checkpoint);
  }

  pub(crate) fn finish_value_at_rule_import_items(
    &self,
    checkpoint: usize,
  ) -> DependencyListRange<ValueAtRuleImportItem<'s>> {
    debug_assert!(checkpoint <= self.value_at_rule_import_items.len());
    DependencyListRange::from_bounds(checkpoint, self.value_at_rule_import_items.len())
  }

  pub(crate) fn push_composes(
    &mut self,
    local_classes: impl IntoIterator<Item = &'s str>,
    names: impl IntoIterator<Item = &'s str>,
    from: Option<&'s str>,
    from_is_global: bool,
    range: Range,
  ) {
    let local_classes_start = self.composes_local_classes.len();
    self.composes_local_classes.extend(local_classes);
    let local_classes = DependencyListRange::<&'s str>::from_bounds(
      local_classes_start,
      self.composes_local_classes.len(),
    );

    let names_start = self.composes_names.len();
    self.composes_names.extend(names);
    let names = DependencyListRange::<&'s str>::from_bounds(names_start, self.composes_names.len());

    self.dependencies.push(Dependency::Composes {
      local_classes,
      names,
      from,
      from_is_global,
      range,
    });
  }
}

impl PartialEq for DependencyContext<'_> {
  fn eq(&self, other: &Self) -> bool {
    self.dependencies == other.dependencies
      && self.dashed_ident_occurrences == other.dashed_ident_occurrences
      && self.import_attributes == other.import_attributes
      && self.composes_local_classes == other.composes_local_classes
      && self.composes_names == other.composes_names
      && self.value_at_rule_import_items == other.value_at_rule_import_items
  }
}

impl Eq for DependencyContext<'_> {}

impl Hash for DependencyContext<'_> {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.dependencies.hash(state);
    self.dashed_ident_occurrences.hash(state);
    self.import_attributes.hash(state);
    self.composes_local_classes.hash(state);
    self.composes_names.hash(state);
    self.value_at_rule_import_items.hash(state);
  }
}

#[derive(Debug, Clone, Copy, Default)]
struct DependencyContextCapacity {
  dependencies: usize,
  dashed_ident_occurrences: usize,
  import_attributes: u16,
  composes_local_classes: u16,
  composes_names: u16,
  value_at_rule_import_items: u16,
}

impl DependencyContextCapacity {
  fn estimate(input_len: usize, mode: Mode) -> Self {
    let dependency_denominator = match mode {
      Mode::Local | Mode::Pure => 32,
      Mode::Global => 80,
      Mode::Css => 4096,
    };
    let dependencies = estimate_capacity(input_len, dependency_denominator, 4, 8192);
    let dashed_ident_occurrences = if mode != Mode::Css {
      estimate_capacity(input_len, 64, 2, 8192)
    } else {
      0
    };
    let import_attributes = estimate_capacity(input_len, 8192, 2, 1024);
    let (composes_local_classes, composes_names, value_at_rule_import_items) = match mode {
      Mode::Local | Mode::Pure => (
        estimate_capacity(input_len, 640, 2, 4096),
        estimate_capacity(input_len, 576, 2, 4096),
        estimate_capacity(input_len, 1792, 2, 4096),
      ),
      Mode::Global => (
        estimate_capacity(input_len, 3880, 2, 4096),
        estimate_capacity(input_len, 3880, 2, 4096),
        estimate_capacity(input_len, 1774, 2, 4096),
      ),
      Mode::Css => (0, 0, 0),
    };
    Self {
      dependencies,
      dashed_ident_occurrences,
      import_attributes: import_attributes as u16,
      composes_local_classes: composes_local_classes as u16,
      composes_names: composes_names as u16,
      value_at_rule_import_items: value_at_rule_import_items as u16,
    }
  }
}

fn estimate_capacity(
  input_len: usize,
  denominator: usize,
  minimum: usize,
  maximum: usize,
) -> usize {
  input_len
    .checked_div(denominator)
    .unwrap_or(0)
    .clamp(minimum, maximum)
}

fn reserve_estimated_capacity<T>(values: &mut Vec<T>, estimated_capacity: usize) {
  if values.capacity() < estimated_capacity {
    values.reserve(estimated_capacity.saturating_sub(values.len()));
  }
}

impl<'s> Index<usize> for DependencyContext<'s> {
  type Output = Dependency<'s>;

  fn index(&self, index: usize) -> &Self::Output {
    &self.dependencies[index]
  }
}

impl<'context, 's> IntoIterator for &'context DependencyContext<'s> {
  type Item = &'context Dependency<'s>;
  type IntoIter = std::slice::Iter<'context, Dependency<'s>>;

  fn into_iter(self) -> Self::IntoIter {
    self.dependencies.iter()
  }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum UrlRangeKind {
  Function,
  String,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Warning<'s> {
  pub(crate) range: Range,
  pub(crate) kind: WarningKind<'s>,
}

impl<'s> Warning<'s> {
  pub fn new(range: Range, kind: WarningKind<'s>) -> Self {
    Self { range, kind }
  }

  pub fn range(&self) -> &Range {
    &self.range
  }

  pub fn kind(&self) -> &WarningKind<'s> {
    &self.kind
  }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum WarningKind<'s> {
  Unexpected { message: &'s str },
  DuplicateUrl { when: &'s str },
  NamespaceNotSupportedInBundledCss,
  NotPrecededAtImport,
  ExpectedUrl { when: &'s str },
  ExpectedUrlBefore { when: &'s str },
  ExpectedLayerBefore { when: &'s str },
  InconsistentModeResult,
  ExpectedNotInside { pseudo: &'s str },
  NotPure { message: &'s str },
  UnexpectedComposition { message: &'s str },
}

impl Display for Warning<'_> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self.kind {
      WarningKind::Unexpected { message, .. } => write!(f, "{message}"),
      WarningKind::DuplicateUrl { when, .. } => {
        write!(f, "Duplicate of 'url(...)' in '{when}'")
      }
      WarningKind::NamespaceNotSupportedInBundledCss { .. } => {
        write!(f, "'@namespace' is not supported in bundled CSS")
      }
      WarningKind::NotPrecededAtImport { .. } => {
        write!(f, "Any '@import' rules must precede all other rules")
      }
      WarningKind::ExpectedUrl { when, .. } => write!(f, "Expected URL in '{when}'"),
      WarningKind::ExpectedUrlBefore { when, .. } => {
        write!(
          f,
          "An URL in '{when}' should be before 'layer(...)' or 'supports(...)'"
        )
      }
      WarningKind::ExpectedLayerBefore { when, .. } => {
        write!(
          f,
          "The 'layer(...)' in '{when}' should be before 'supports(...)'"
        )
      }
      WarningKind::InconsistentModeResult { .. } => write!(
        f,
        "Inconsistent rule global/local (multiple selectors must result in the same mode for the rule)"
      ),
      WarningKind::ExpectedNotInside { pseudo, .. } => write!(
        f,
        "A '{pseudo}' is not allowed inside of a ':local()' or ':global()'"
      ),
      WarningKind::NotPure { message, .. } => {
        write!(f, "Pure globals is not allowed in pure mode, {message}")
      }
      WarningKind::UnexpectedComposition { message, .. } => {
        write!(f, "Composition is {message}")
      }
    }
  }
}

#[cfg(test)]
#[path = "../tests/dependency_types_tests.rs"]
mod tests;
