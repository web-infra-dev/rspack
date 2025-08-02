use std::{borrow::Cow, hash::Hash, sync::atomic::AtomicU32};

use either::Either;
use rspack_cacheable::{
  cacheable,
  with::{AsPreset, AsVec},
};
use rspack_util::{atom::Atom, json_stringify};
use rustc_hash::FxHashSet as HashSet;

use crate::DependencyId;

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

#[cacheable]
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
enum EvaluatedInlinableValueInner {
  Null,
  Undefined,
  Boolean(bool),
  Number(#[cacheable(with=AsPreset)] Atom),
  String(#[cacheable(with=AsPreset)] Atom),
}

#[cacheable]
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
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

  pub fn new_number(v: Atom) -> Self {
    Self(EvaluatedInlinableValueInner::Number(v))
  }

  pub fn new_string(v: Atom) -> Self {
    Self(EvaluatedInlinableValueInner::String(v))
  }

  pub fn render(&self) -> Cow<'_, str> {
    match &self.0 {
      EvaluatedInlinableValueInner::Null => "null".into(),
      EvaluatedInlinableValueInner::Undefined => "undefined".into(),
      EvaluatedInlinableValueInner::Boolean(v) => if *v { "true" } else { "false" }.into(),
      EvaluatedInlinableValueInner::Number(v) => v.as_str().into(),
      EvaluatedInlinableValueInner::String(v) => json_stringify(v.as_str()).into(),
    }
  }
}

#[cacheable]
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
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

#[derive(Debug, Hash, PartialEq, Eq, Default)]
pub struct UsageKey(pub Vec<Either<Box<UsageKey>, UsageState>>);

impl UsageKey {
  pub fn add(&mut self, value: Either<Box<UsageKey>, UsageState>) {
    self.0.push(value);
  }
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
