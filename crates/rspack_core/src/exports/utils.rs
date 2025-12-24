use std::{borrow::Cow, hash::Hash, sync::atomic::AtomicU32};

use either::Either;
use rspack_cacheable::{
  cacheable,
  with::{AsPreset, AsVec},
};
use rspack_util::{atom::Atom, json_stringify, ryu_js};
use rustc_hash::FxHashSet as HashSet;

use crate::{DependencyId, property_access};

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
#[derive(Debug, Clone)]
pub enum EvaluatedInlinableValue {
  Null,
  Undefined,
  Boolean(bool),
  Number(f64),
  String(#[cacheable(with=AsPreset)] Atom),
}

impl Hash for EvaluatedInlinableValue {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    std::mem::discriminant(self).hash(state);
    match self {
      EvaluatedInlinableValue::Boolean(v) => {
        v.hash(state);
      }
      EvaluatedInlinableValue::Number(v) => {
        v.to_bits().hash(state);
      }
      EvaluatedInlinableValue::String(atom) => {
        atom.hash(state);
      }
      _ => {}
    }
  }
}

impl EvaluatedInlinableValue {
  pub const SHORT_SIZE: usize = 6;

  pub fn new_null() -> Self {
    Self::Null
  }

  pub fn new_undefined() -> Self {
    Self::Undefined
  }

  pub fn new_boolean(v: bool) -> Self {
    Self::Boolean(v)
  }

  pub fn new_number(v: f64) -> Self {
    Self::Number(v)
  }

  pub fn new_string(v: Atom) -> Self {
    Self::String(v)
  }

  pub fn render(&self) -> String {
    let s: Cow<str> = match self {
      Self::Null => "null".into(),
      Self::Undefined => "undefined".into(),
      Self::Boolean(v) => if *v { "true" } else { "false" }.into(),
      Self::Number(v) => {
        let mut buf = ryu_js::Buffer::new();
        buf.format(*v).to_string().into()
      }
      Self::String(v) => json_stringify(v.as_str()).into(),
    };
    format!("({s})")
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CanInlineUse {
  // Must have this initial state to get the correct dependency condition of inline value
  // at flag_dependency_usage_plugin. If it's just bool and the initial state is true like
  // mangleExports, then the dependency condition of inline value will be false and the
  // flag_dependency_usage_plugin will not collect the usage of these dependency.
  HasInfo,
  Yes,
  No,
}

#[derive(Debug, Clone, Hash)]
pub enum UsedNameItem {
  Str(Atom),
  Inlined(EvaluatedInlinableValue),
}

#[derive(Debug, Clone)]
pub struct InlinedUsedName {
  value: EvaluatedInlinableValue,
  suffix: Vec<Atom>,
}

impl InlinedUsedName {
  pub fn new(value: EvaluatedInlinableValue) -> Self {
    Self {
      value,
      suffix: Vec::new(),
    }
  }

  pub fn render(&self) -> String {
    let mut inlined = self.value.render();
    inlined.push_str(&property_access(&self.suffix, 0));
    inlined
  }

  pub fn inlined_value(&self) -> &EvaluatedInlinableValue {
    &self.value
  }

  pub fn suffix_ids(&self) -> &[Atom] {
    &self.suffix
  }
}

#[derive(Debug, Clone)]
pub enum UsedName {
  Normal(Vec<Atom>),
  Inlined(InlinedUsedName),
}

impl UsedName {
  pub fn append(&mut self, item: impl IntoIterator<Item = Atom>) {
    match self {
      UsedName::Normal(vec) => vec.extend(item),
      UsedName::Inlined(inlined) => inlined.suffix.extend(item),
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

#[derive(Debug, Hash, PartialEq, Eq, Default, Clone)]
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
