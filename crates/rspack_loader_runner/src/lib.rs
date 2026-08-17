#![feature(string_from_utf8_lossy_owned)]

mod chain;
mod content;
mod context;
mod loader;
mod plugin;
mod runner;
mod scheme;

pub use chain::{LoaderChain, LoaderExecutionKind, plan_loader_chains};
pub use content::{AdditionalData, Content, DescriptionData, ParseMeta, ResourceData};
pub use context::{LoaderContext, State};
pub use loader::{
  DisplayWithSuffix, Loader, LoaderItem, LoaderItemState, ResourceParsedData, parse_resource,
};
pub use plugin::LoaderRunnerPlugin;
pub use rspack_collections::{Identifiable, Identifier};
pub use runner::{LoaderResult, Loaders, run_loaders};
pub use scheme::{Scheme, get_scheme};

pub const BUILTIN_LOADER_PREFIX: &str = "builtin:";
