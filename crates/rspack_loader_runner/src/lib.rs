#![feature(string_from_utf8_lossy_owned)]

mod chain;
mod content;
mod context;
mod loader;
mod plugin;
mod runner;
mod scheme;

pub use chain::{
  LoaderChain, LoaderChainCacheAction, LoaderChainCacheState, LoaderChainMergeReason,
  LoaderChainStrategy, LoaderExecutionKind, LoaderRunnerOptions, plan_loader_chains,
};
pub use content::{AdditionalData, Content, DescriptionData, ParseMeta, ResourceData};
pub use context::{LoaderContext, State};
pub use loader::{DisplayWithSuffix, Loader, LoaderItem, ResourceParsedData, parse_resource};
pub use plugin::LoaderRunnerPlugin;
pub use rspack_collections::{Identifiable, Identifier};
pub use runner::{
  LoaderResult, run_loaders, run_loaders_with_options, run_loaders_with_options_and_strategy,
};
pub use scheme::{Scheme, get_scheme};

pub const BUILTIN_LOADER_PREFIX: &str = "builtin:";
