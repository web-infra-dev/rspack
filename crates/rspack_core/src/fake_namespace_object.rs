use std::fmt;

use bitflags::bitflags;
use serde::Serialize;

bitflags! {
  #[derive(Debug, Clone, Copy, Eq, PartialEq)]
    pub struct FakeNamespaceObjectMode: u8 {
        const MODULE_ID = 1 << 0; // value is a module id, require it
        const MERGE_PROPERTIES = 1 << 1; // merge all properties of value into the ns
        const RETURN_VALUE = 1 << 2; // return value when already ns object
        const REQUIRE =  1 << 3;
        const PROMISE_LIKE = 1 << 4; // return value when it's Promise-like
        const NAMESPACE = Self::MODULE_ID.bits() | Self::REQUIRE.bits();
        const DYNAMIC = Self::MODULE_ID.bits() | Self::MERGE_PROPERTIES.bits() | Self::RETURN_VALUE.bits();
        const DEFAULT_WITH_NAMED = Self::MODULE_ID.bits() | Self::MERGE_PROPERTIES.bits();
    }
}

impl fmt::Display for FakeNamespaceObjectMode {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_fmt(format_args!("{}", self.bits()))
  }
}

impl Serialize for FakeNamespaceObjectMode {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    serializer.serialize_u8(self.bits())
  }
}
