// TODO: move to rspack_plugin_mf once we remove the hardcoded DependencyType => ModuleFactory
mod container;
mod sharing;
pub use container::*;
pub use sharing::*;

mod utils {
  use std::fmt;

  use serde::Serialize;

  pub fn json_stringify<T: ?Sized + Serialize + fmt::Debug>(v: &T) -> String {
    serde_json::to_string(v).unwrap_or_else(|e| panic!("{e}: {v:?} should able to json stringify"))
  }
}
