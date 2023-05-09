#[derive(Debug, Clone, Copy)]
pub enum SideEffectOption {
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

#[derive(Debug)]
pub struct Optimization {
  pub remove_available_modules: bool,
  pub remove_empty_chunks: bool,
  pub side_effects: SideEffectOption,
}
