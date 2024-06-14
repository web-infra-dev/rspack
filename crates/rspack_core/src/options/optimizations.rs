#[derive(Debug, Clone, Copy, Default)]
pub enum SideEffectOption {
  #[default]
  False,
  True,
  Flag,
}

impl From<&str> for SideEffectOption {
  fn from(value: &str) -> Self {
    match value {
      "true" => Self::True,
      "flag" => Self::Flag,
      _ => Self::False,
    }
  }
}

impl SideEffectOption {
  /// Returns `true` if the side effect option is [`False`].
  ///
  /// [`False`]: SideEffectOption::False
  #[must_use]
  pub fn is_false(&self) -> bool {
    matches!(self, Self::False)
  }

  /// Returns `true` if the side effect option is [`True`].
  ///
  /// [`True`]: SideEffectOption::True
  #[must_use]
  pub fn is_true(&self) -> bool {
    matches!(self, Self::True)
  }

  /// Returns `true` if the side effect option is [`Flag`].
  ///
  /// [`Flag`]: SideEffectOption::Flag
  pub fn is_flag(&self) -> bool {
    matches!(self, Self::Flag)
  }

  pub fn is_enable(&self) -> bool {
    matches!(self, Self::Flag | Self::True)
  }
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub enum UsedExportsOption {
  #[default]
  False,
  True,
  Global,
}

impl From<&str> for UsedExportsOption {
  fn from(value: &str) -> Self {
    match value {
      "true" => Self::True,
      "global" => Self::Global,
      _ => Self::False,
    }
  }
}

impl UsedExportsOption {
  pub fn is_enable(&self) -> bool {
    matches!(self, Self::Global | Self::True)
  }

  /// Returns `true` if the used exports is [`False`].
  ///
  /// [`False`]: UsedExports::False
  #[must_use]
  pub fn is_false(&self) -> bool {
    matches!(self, Self::False)
  }

  /// Returns `true` if the used exports is [`True`].
  ///
  /// [`True`]: UsedExports::True
  #[must_use]
  pub fn is_true(&self) -> bool {
    matches!(self, Self::True)
  }

  /// Returns `true` if the used exports is [`Global`].
  ///
  /// [`Global`]: UsedExports::Global
  #[must_use]
  pub fn is_global(&self) -> bool {
    matches!(self, Self::Global)
  }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum MangleExportsOption {
  #[default]
  False,
  True,
  Deterministic,
  Size,
}

impl MangleExportsOption {
  pub fn is_enable(&self) -> bool {
    !matches!(self, Self::False)
  }
}

impl From<&str> for MangleExportsOption {
  fn from(value: &str) -> Self {
    match value {
      "true" => Self::True,
      "size" => Self::Size,
      "deterministic" => Self::Deterministic,
      _ => Self::False,
    }
  }
}

#[derive(Debug)]
pub struct Optimization {
  pub side_effects: SideEffectOption,
  pub provided_exports: bool,
  pub used_exports: UsedExportsOption,
  pub inner_graph: bool,
  pub mangle_exports: MangleExportsOption,
  pub concatenate_modules: bool,
}

pub static DEFAULT_DELIMITER: &str = "~";
