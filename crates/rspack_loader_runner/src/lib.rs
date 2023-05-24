#![feature(let_chains)]

mod content;
mod loader;
mod plugin;
mod runner;

pub use content::Content;
pub use loader::{DisplayWithSuffix, Loader};
pub use plugin::LoaderRunnerPlugin;
pub use rspack_identifier::{Identifiable, Identifier};
pub use runner::{get_scheme, run_loaders, LoaderContext, ResourceData, Scheme};
