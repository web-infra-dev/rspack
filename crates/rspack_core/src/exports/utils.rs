use std::{borrow::Cow, hash::Hash, sync::atomic::AtomicU32};

use either::Either;
use rspack_cacheable::{
  cacheable,
  with::{AsPreset, AsVec},
};
use rspack_util::{atom::Atom, json_stringify};
use rustc_hash::FxHashSet as HashSet;

use crate::{DependencyId, ModuleIdentifier};

pub static NEXT_EXPORTS_INFO_UKEY: AtomicU32 = AtomicU32::new(0);
pub static NEXT_EXPORT_INFO_UKEY: AtomicU32 = AtomicU32::new(0);

#[derive(Debug, Clone, Hash)]
pub struct ExportInfoTargetValue {
  pub dependency: Option<DependencyId>,
  pub export: Option<Vec<Atom>>,
  pub priority: u8,
}

pub enum ProvidedExports {
  Unknown,
  ProvidedAll,
  ProvidedNames(Vec<Atom>),
}

pub enum UsedExports {
  Unknown,
  UsedNamespace(bool),
  UsedNames(Vec<Atom>),
}

// refer from: https://github.com/rust-analyzer/smol_str/blob/5ffc90069f545c0444447cd08c2a29c6abb97fbb/src/lib.rs#L481-L484
#[cacheable]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
struct InlineStr<const S: usize> {
  len: u8,
  buf: [u8; S],
}

impl<const S: usize> InlineStr<S> {
  fn new(v: &str) -> Self {
    let len = v.len();
    debug_assert!(len <= S);
    let mut buf = [0; S];
    buf[..len].copy_from_slice(v.as_bytes());
    Self {
      len: len as u8,
      buf,
    }
  }

  fn as_str(&self) -> &str {
    let len: usize = self.len as usize;
    // SAFETY: len is guaranteed to be <= Self::SHORT_SIZE
    let buf = unsafe { self.buf.get_unchecked(..len) };
    // SAFETY: buf is guaranteed to be valid utf8 for ..len bytes
    unsafe { ::core::str::from_utf8_unchecked(buf) }
  }
}

#[cacheable]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
enum EvaluatedInlinableValueInner {
  Null,
  Undefined,
  Boolean(bool),
  ShortNumber(InlineStr<{ EvaluatedInlinableValue::SHORT_SIZE }>),
  ShortString(InlineStr<{ EvaluatedInlinableValue::SHORT_SIZE }>),
}

#[cacheable]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct EvaluatedInlinableValue(EvaluatedInlinableValueInner);

impl EvaluatedInlinableValue {
  pub const SHORT_SIZE: usize = 6;

  pub fn new_null() -> Self {
    Self(EvaluatedInlinableValueInner::Null)
  }

  pub fn new_undefined() -> Self {
    Self(EvaluatedInlinableValueInner::Undefined)
  }

  pub fn new_boolean(v: bool) -> Self {
    Self(EvaluatedInlinableValueInner::Boolean(v))
  }

  pub fn new_short_number(v: &str) -> Self {
    Self(EvaluatedInlinableValueInner::ShortNumber(InlineStr::new(v)))
  }

  pub fn new_short_string(v: &str) -> Self {
    Self(EvaluatedInlinableValueInner::ShortString(InlineStr::new(v)))
  }

  pub fn render(&self) -> Cow<str> {
    match &self.0 {
      EvaluatedInlinableValueInner::Null => "null".into(),
      EvaluatedInlinableValueInner::Undefined => "undefined".into(),
      EvaluatedInlinableValueInner::Boolean(v) => if *v { "true" } else { "false" }.into(),
      EvaluatedInlinableValueInner::ShortNumber(v) => v.as_str().into(),
      EvaluatedInlinableValueInner::ShortString(v) => json_stringify(v.as_str()).into(),
    }
  }
}

#[cacheable]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Inlinable {
  NoByProvide,
  NoByUse,
  Inlined(EvaluatedInlinableValue),
}

impl Inlinable {
  pub fn can_inline(&self) -> bool {
    matches!(self, Inlinable::Inlined(_))
  }
}

#[derive(Debug, Clone)]
pub enum UsedNameItem {
  Str(Atom),
  Inlined(EvaluatedInlinableValue),
}

#[derive(Debug, Clone)]
pub enum UsedName {
  Normal(Vec<Atom>),
  Inlined(EvaluatedInlinableValue),
}

impl UsedName {
  pub fn is_inlined(&self) -> bool {
    matches!(self, UsedName::Inlined(_))
  }

  pub fn inlined(&self) -> Option<&EvaluatedInlinableValue> {
    match self {
      UsedName::Inlined(inlined) => Some(inlined),
      _ => None,
    }
  }
}

#[derive(Debug, Hash, Clone, Copy)]
pub enum ExportProvided {
  /// The export can be statically analyzed, and it is provided
  Provided,
  /// The export can be statically analyzed, and the it is not provided
  NotProvided,
  /// The export is unknown, we don't know if module really has this export, eg. cjs module
  Unknown,
}

#[derive(Clone, Debug)]
pub struct ResolvedExportInfoTarget {
  pub module: ModuleIdentifier,
  pub export: Option<Vec<Atom>>,
  /// using dependency id to retrieve Connection
  pub dependency: DependencyId,
}

#[derive(Clone, Debug)]
pub enum FindTargetRetEnum {
  Undefined,
  False,
  Value(FindTargetRetValue),
}
#[derive(Clone, Debug)]
pub struct FindTargetRetValue {
  pub module: ModuleIdentifier,
  pub export: Option<Vec<Atom>>,
}

#[derive(Debug, Hash, PartialEq, Eq, Default)]
pub struct UsageKey(pub Vec<Either<Box<UsageKey>, UsageState>>);

impl UsageKey {
  pub fn add(&mut self, value: Either<Box<UsageKey>, UsageState>) {
    self.0.push(value);
  }
}

#[derive(Debug, Clone)]
pub struct UnResolvedExportInfoTarget {
  pub dependency: Option<DependencyId>,
  pub export: Option<Vec<Atom>>,
}

#[derive(Debug)]
pub enum ResolvedExportInfoTargetWithCircular {
  Target(ResolvedExportInfoTarget),
  Circular,
}

pub type UsageFilterFnTy<T> = Box<dyn Fn(&T) -> bool>;

#[derive(Debug, PartialEq, Copy, Clone, Default, Hash, PartialOrd, Ord, Eq)]
pub enum UsageState {
  Unused = 0,
  OnlyPropertiesUsed = 1,
  NoInfo = 2,
  #[default]
  Unknown = 3,
  Used = 4,
}

#[cacheable]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UsedByExports {
  Set(#[cacheable(with=AsVec<AsPreset>)] HashSet<Atom>),
  Bool(bool),
}

/// refer https://github.com/webpack/webpack/blob/d15c73469fd71cf98734685225250148b68ddc79/lib/FlagDependencyUsagePlugin.js#L64
#[derive(Clone, Debug)]
pub enum ExtendedReferencedExport {
  Array(Vec<Atom>),
  Export(ReferencedExport),
}

pub fn is_no_exports_referenced(exports: &[ExtendedReferencedExport]) -> bool {
  exports.is_empty()
}

pub fn is_exports_object_referenced(exports: &[ExtendedReferencedExport]) -> bool {
  matches!(exports[..], [ExtendedReferencedExport::Array(ref arr)] if arr.is_empty())
}

pub fn create_no_exports_referenced() -> Vec<ExtendedReferencedExport> {
  vec![]
}

pub fn create_exports_object_referenced() -> Vec<ExtendedReferencedExport> {
  vec![ExtendedReferencedExport::Array(vec![])]
}

impl From<Vec<Atom>> for ExtendedReferencedExport {
  fn from(value: Vec<Atom>) -> Self {
    ExtendedReferencedExport::Array(value)
  }
}
impl From<ReferencedExport> for ExtendedReferencedExport {
  fn from(value: ReferencedExport) -> Self {
    ExtendedReferencedExport::Export(value)
  }
}

#[derive(Clone, Debug)]
pub struct ReferencedExport {
  pub name: Vec<Atom>,
  pub can_mangle: bool,
  pub can_inline: bool,
}

impl ReferencedExport {
  pub fn new(name: Vec<Atom>, can_mangle: bool, can_inline: bool) -> Self {
    Self {
      name,
      can_mangle,
      can_inline,
    }
  }
}

impl Default for ReferencedExport {
  fn default() -> Self {
    Self {
      name: vec![],
      can_mangle: true,
      can_inline: true,
    }
  }
}
