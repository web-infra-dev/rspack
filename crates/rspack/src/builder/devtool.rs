use bitflags::bitflags;

/// Devtool options used to control the behavior of Source Map generation.
#[derive(Debug, PartialEq)]
pub enum Devtool {
  /// Disable source map.
  False,
  /// Use eval to generate source map.
  Eval,
  /// Use cheap-source-map to generate source map.
  CheapSourceMap,
  /// Use cheap-module-source-map to generate source map.
  CheapModuleSourceMap,
  /// Use source-map to generate source map.
  SourceMap,
  /// Use inline-cheap-source-map to generate source map.
  InlineCheapSourceMap,
  /// Use inline-cheap-module-source-map to generate source map.
  InlineCheapModuleSourceMap,
  /// Use inline-source-map to generate source map.
  InlineSourceMap,
  /// Use inline-nosources-cheap-source-map to generate source map.
  InlineNosourcesCheapSourceMap,
  /// Use inline-nosources-cheap-module-source-map to generate source map.
  InlineNosourcesCheapModuleSourceMap,
  /// Use inline-nosources-source-map to generate source map.
  InlineNosourcesSourceMap,
  /// Use nosources-cheap-source-map to generate source map.
  NosourcesCheapSourceMap,
  /// Use nosources-cheap-module-source-map to generate source map.
  NosourcesCheapModuleSourceMap,
  /// Use nosources-source-map to generate source map.
  NosourcesSourceMap,
  /// Use hidden-nosources-cheap-source-map to generate source map.
  HiddenNosourcesCheapSourceMap,
  /// Use hidden-nosources-cheap-module-source-map to generate source map.
  HiddenNosourcesCheapModuleSourceMap,
  /// Use hidden-nosources-source-map to generate source map.
  HiddenNosourcesSourceMap,
  /// Use hidden-cheap-source-map to generate source map.
  HiddenCheapSourceMap,
  /// Use hidden-cheap-module-source-map to generate source map.
  HiddenCheapModuleSourceMap,
  /// Use hidden-source-map to generate source map.
  HiddenSourceMap,
  /// Use eval-cheap-source-map to generate source map.
  EvalCheapSourceMap,
  /// Use eval-cheap-module-source-map to generate source map.
  EvalCheapModuleSourceMap,
  /// Use eval-source-map to generate source map.
  EvalSourceMap,
  /// Use eval-nosources-cheap-source-map to generate source map.
  EvalNosourcesCheapSourceMap,
  /// Use eval-nosources-cheap-module-source-map to generate source map.
  EvalNosourcesCheapModuleSourceMap,
  /// Use eval-nosources-source-map to generate source map.
  EvalNosourcesSourceMap,
}

// Implement FromStr trait to parse strings into Devtool enum
impl std::str::FromStr for Devtool {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "false" => Ok(Devtool::False),
      "eval" => Ok(Devtool::Eval),
      "cheap-source-map" => Ok(Devtool::CheapSourceMap),
      "cheap-module-source-map" => Ok(Devtool::CheapModuleSourceMap),
      "source-map" => Ok(Devtool::SourceMap),
      "inline-cheap-source-map" => Ok(Devtool::InlineCheapSourceMap),
      "inline-cheap-module-source-map" => Ok(Devtool::InlineCheapModuleSourceMap),
      "inline-source-map" => Ok(Devtool::InlineSourceMap),
      "inline-nosources-cheap-source-map" => Ok(Devtool::InlineNosourcesCheapSourceMap),
      "inline-nosources-cheap-module-source-map" => {
        Ok(Devtool::InlineNosourcesCheapModuleSourceMap)
      }
      "inline-nosources-source-map" => Ok(Devtool::InlineNosourcesSourceMap),
      "nosources-cheap-source-map" => Ok(Devtool::NosourcesCheapSourceMap),
      "nosources-cheap-module-source-map" => Ok(Devtool::NosourcesCheapModuleSourceMap),
      "nosources-source-map" => Ok(Devtool::NosourcesSourceMap),
      "hidden-nosources-cheap-source-map" => Ok(Devtool::HiddenNosourcesCheapSourceMap),
      "hidden-nosources-cheap-module-source-map" => {
        Ok(Devtool::HiddenNosourcesCheapModuleSourceMap)
      }
      "hidden-nosources-source-map" => Ok(Devtool::HiddenNosourcesSourceMap),
      "hidden-cheap-source-map" => Ok(Devtool::HiddenCheapSourceMap),
      "hidden-cheap-module-source-map" => Ok(Devtool::HiddenCheapModuleSourceMap),
      "hidden-source-map" => Ok(Devtool::HiddenSourceMap),
      "eval-cheap-source-map" => Ok(Devtool::EvalCheapSourceMap),
      "eval-cheap-module-source-map" => Ok(Devtool::EvalCheapModuleSourceMap),
      "eval-source-map" => Ok(Devtool::EvalSourceMap),
      "eval-nosources-cheap-source-map" => Ok(Devtool::EvalNosourcesCheapSourceMap),
      "eval-nosources-cheap-module-source-map" => Ok(Devtool::EvalNosourcesCheapModuleSourceMap),
      "eval-nosources-source-map" => Ok(Devtool::EvalNosourcesSourceMap),
      _ => Err(format!("Invalid devtool value: {s}")),
    }
  }
}

// Implement Display trait to convert enum variants back to strings
impl std::fmt::Display for Devtool {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    let s = match self {
      Devtool::False => "false",
      Devtool::Eval => "eval",
      Devtool::CheapSourceMap => "cheap-source-map",
      Devtool::CheapModuleSourceMap => "cheap-module-source-map",
      Devtool::SourceMap => "source-map",
      Devtool::InlineCheapSourceMap => "inline-cheap-source-map",
      Devtool::InlineCheapModuleSourceMap => "inline-cheap-module-source-map",
      Devtool::InlineSourceMap => "inline-source-map",
      Devtool::InlineNosourcesCheapSourceMap => "inline-nosources-cheap-source-map",
      Devtool::InlineNosourcesCheapModuleSourceMap => "inline-nosources-cheap-module-source-map",
      Devtool::InlineNosourcesSourceMap => "inline-nosources-source-map",
      Devtool::NosourcesCheapSourceMap => "nosources-cheap-source-map",
      Devtool::NosourcesCheapModuleSourceMap => "nosources-cheap-module-source-map",
      Devtool::NosourcesSourceMap => "nosources-source-map",
      Devtool::HiddenNosourcesCheapSourceMap => "hidden-nosources-cheap-source-map",
      Devtool::HiddenNosourcesCheapModuleSourceMap => "hidden-nosources-cheap-module-source-map",
      Devtool::HiddenNosourcesSourceMap => "hidden-nosources-source-map",
      Devtool::HiddenCheapSourceMap => "hidden-cheap-source-map",
      Devtool::HiddenCheapModuleSourceMap => "hidden-cheap-module-source-map",
      Devtool::HiddenSourceMap => "hidden-source-map",
      Devtool::EvalCheapSourceMap => "eval-cheap-source-map",
      Devtool::EvalCheapModuleSourceMap => "eval-cheap-module-source-map",
      Devtool::EvalSourceMap => "eval-source-map",
      Devtool::EvalNosourcesCheapSourceMap => "eval-nosources-cheap-source-map",
      Devtool::EvalNosourcesCheapModuleSourceMap => "eval-nosources-cheap-module-source-map",
      Devtool::EvalNosourcesSourceMap => "eval-nosources-source-map",
    };
    write!(f, "{s}")
  }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct DevtoolFlags: u32 {
        const SOURCE_MAP = 1 << 0;
        const HIDDEN    = 1 << 1;
        const INLINE    = 1 << 2;
        const EVAL      = 1 << 3;
        const CHEAP     = 1 << 4;
        const MODULE    = 1 << 5;
        const NOSOURCES = 1 << 6;
    }
}

impl DevtoolFlags {
  pub(crate) fn source_map(self) -> bool {
    self.contains(DevtoolFlags::SOURCE_MAP)
  }

  pub(crate) fn hidden(self) -> bool {
    self.contains(DevtoolFlags::HIDDEN)
  }

  pub(crate) fn inline(self) -> bool {
    self.contains(DevtoolFlags::INLINE)
  }

  pub(crate) fn eval(self) -> bool {
    self.contains(DevtoolFlags::EVAL)
  }

  pub(crate) fn cheap(self) -> bool {
    self.contains(DevtoolFlags::CHEAP)
  }

  pub(crate) fn module(self) -> bool {
    self.contains(DevtoolFlags::MODULE)
  }

  pub(crate) fn nosources(self) -> bool {
    self.contains(DevtoolFlags::NOSOURCES)
  }
}

impl From<Devtool> for DevtoolFlags {
  fn from(devtool: Devtool) -> Self {
    match devtool {
      Devtool::False => DevtoolFlags::empty(),
      Devtool::Eval => DevtoolFlags::EVAL,
      _ => {
        // Start with SOURCE_MAP since all other variants contain it
        let mut flags = DevtoolFlags::SOURCE_MAP;

        match devtool {
          Devtool::HiddenSourceMap
          | Devtool::HiddenCheapSourceMap
          | Devtool::HiddenCheapModuleSourceMap
          | Devtool::HiddenNosourcesSourceMap
          | Devtool::HiddenNosourcesCheapSourceMap
          | Devtool::HiddenNosourcesCheapModuleSourceMap => {
            flags |= DevtoolFlags::HIDDEN;
          }
          _ => {}
        }

        match devtool {
          Devtool::InlineSourceMap
          | Devtool::InlineCheapSourceMap
          | Devtool::InlineCheapModuleSourceMap
          | Devtool::InlineNosourcesSourceMap
          | Devtool::InlineNosourcesCheapSourceMap
          | Devtool::InlineNosourcesCheapModuleSourceMap => {
            flags |= DevtoolFlags::INLINE;
          }
          _ => {}
        }

        match devtool {
          Devtool::EvalSourceMap
          | Devtool::EvalCheapSourceMap
          | Devtool::EvalCheapModuleSourceMap
          | Devtool::EvalNosourcesSourceMap
          | Devtool::EvalNosourcesCheapSourceMap
          | Devtool::EvalNosourcesCheapModuleSourceMap => {
            flags |= DevtoolFlags::EVAL;
          }
          _ => {}
        }

        match devtool {
          Devtool::CheapSourceMap
          | Devtool::CheapModuleSourceMap
          | Devtool::InlineCheapSourceMap
          | Devtool::InlineCheapModuleSourceMap
          | Devtool::EvalCheapSourceMap
          | Devtool::EvalCheapModuleSourceMap
          | Devtool::NosourcesCheapSourceMap
          | Devtool::NosourcesCheapModuleSourceMap
          | Devtool::HiddenCheapSourceMap
          | Devtool::HiddenCheapModuleSourceMap
          | Devtool::InlineNosourcesCheapSourceMap
          | Devtool::InlineNosourcesCheapModuleSourceMap
          | Devtool::EvalNosourcesCheapSourceMap
          | Devtool::EvalNosourcesCheapModuleSourceMap
          | Devtool::HiddenNosourcesCheapSourceMap
          | Devtool::HiddenNosourcesCheapModuleSourceMap => {
            flags |= DevtoolFlags::CHEAP;
          }
          _ => {}
        }

        match devtool {
          Devtool::CheapModuleSourceMap
          | Devtool::InlineCheapModuleSourceMap
          | Devtool::EvalCheapModuleSourceMap
          | Devtool::NosourcesCheapModuleSourceMap
          | Devtool::HiddenCheapModuleSourceMap
          | Devtool::InlineNosourcesCheapModuleSourceMap
          | Devtool::EvalNosourcesCheapModuleSourceMap
          | Devtool::HiddenNosourcesCheapModuleSourceMap => {
            flags |= DevtoolFlags::MODULE;
          }
          _ => {}
        }

        match devtool {
          Devtool::NosourcesSourceMap
          | Devtool::NosourcesCheapSourceMap
          | Devtool::NosourcesCheapModuleSourceMap
          | Devtool::InlineNosourcesSourceMap
          | Devtool::InlineNosourcesCheapSourceMap
          | Devtool::InlineNosourcesCheapModuleSourceMap
          | Devtool::EvalNosourcesSourceMap
          | Devtool::EvalNosourcesCheapSourceMap
          | Devtool::EvalNosourcesCheapModuleSourceMap
          | Devtool::HiddenNosourcesSourceMap
          | Devtool::HiddenNosourcesCheapSourceMap
          | Devtool::HiddenNosourcesCheapModuleSourceMap => {
            flags |= DevtoolFlags::NOSOURCES;
          }
          _ => {}
        }

        flags
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_devtool_flags_conversion() {
    // Test eval-cheap-module-source-map
    let devtool = Devtool::EvalCheapModuleSourceMap;
    let flags = DevtoolFlags::from(devtool);

    assert!(flags.contains(DevtoolFlags::SOURCE_MAP));
    assert!(flags.contains(DevtoolFlags::EVAL));
    assert!(flags.contains(DevtoolFlags::CHEAP));
    assert!(flags.contains(DevtoolFlags::MODULE));
    assert!(!flags.contains(DevtoolFlags::HIDDEN));
    assert!(!flags.contains(DevtoolFlags::INLINE));
    assert!(!flags.contains(DevtoolFlags::NOSOURCES));
  }

  #[test]
  fn test_eval_only() {
    let devtool = Devtool::Eval;
    let flags = DevtoolFlags::from(devtool);

    assert!(flags.contains(DevtoolFlags::EVAL));
    assert!(!flags.contains(DevtoolFlags::SOURCE_MAP));
  }
}
