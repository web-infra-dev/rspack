#![feature(let_chains)]
#![feature(round_char_boundary)]

mod content;
mod loader;
mod plugin;
mod runner;
mod scheme;

pub use content::Content;
pub use loader::{DisplayWithSuffix, Loader};
pub use plugin::LoaderRunnerPlugin;
pub use rspack_identifier::{Identifiable, Identifier};
pub use runner::{run_loaders, AdditionalData, DescriptionData, LoaderContext, ResourceData};
pub use scheme::{get_scheme, Scheme};

pub const BUILTIN_LOADER_PREFIX: &str = "builtin:";

#[doc(hidden)]
pub mod __private {
  pub mod loader {
    pub use crate::loader::{LoaderItem, LoaderItemList};
  }
}
