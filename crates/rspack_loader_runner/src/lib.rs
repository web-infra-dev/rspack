#![feature(let_chains)]
#![feature(round_char_boundary)]

mod content;
mod context;
mod loader;
mod plugin;
mod runner;
mod scheme;

pub use content::{AdditionalData, Content, DescriptionData, ResourceData};
pub use context::{LoaderContext, State};
pub use loader::{parse_resource, DisplayWithSuffix, Loader, LoaderItem, ResourceParsedData};
pub use plugin::LoaderRunnerPlugin;
pub use rspack_collections::{Identifiable, Identifier};
pub use runner::run_loaders;
pub use scheme::{get_scheme, Scheme};

pub const BUILTIN_LOADER_PREFIX: &str = "builtin:";
