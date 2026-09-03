#![feature(string_from_utf8_lossy_owned)]

mod cache;
mod chain;
mod content;
mod context;
mod loader;
mod plugin;
mod runner;
mod scheme;

pub use cache::LoaderRunnerOptions;
pub use chain::{LoaderChain, LoaderExecutionKind};
pub use content::{
  AdditionalData, Content, DescriptionData, ParseMeta, ParseMetaValue, ResourceData,
};
pub use context::{LoaderContext, LoaderDependencies, LoaderRunnerContext, State};
pub use loader::{
  DisplayWithSuffix, Loader, LoaderItem, LoaderItemState, ResourceParsedData, parse_resource,
};
pub use plugin::LoaderRunnerPlugin;
pub use rspack_collections::{Identifiable, Identifier};
pub use runner::{LoaderResult, Loaders, ResolvedLoader, run_loaders};
pub use scheme::{Scheme, get_scheme};

pub const BUILTIN_LOADER_PREFIX: &str = "builtin:";
