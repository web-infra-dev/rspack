#![feature(let_chains)]

mod content;
mod loader;
mod plugin;
mod runner;
mod scheme;

pub use content::Content;
pub use loader::{DisplayWithSuffix, Loader};
pub use plugin::LoaderRunnerPlugin;
pub use rspack_identifier::{Identifiable, Identifier};
pub use runner::{run_loaders, LoaderContext, ResourceData};
pub use scheme::{get_scheme, Scheme};
