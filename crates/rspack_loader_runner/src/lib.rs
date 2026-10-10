#![feature(string_from_utf8_lossy_owned)]

mod cache;
mod chain;
mod content;
mod context;
mod loader;
mod plugin;
mod registry;
mod runner;
mod scheme;

pub use content::{
  AdditionalData, Content, DescriptionData, ParseMeta, ParseMetaValue, ResourceData,
};
pub use context::{LoaderContext, LoaderContextHandle, LoaderDependencies, State};
pub use loader::{
  DisplayWithSuffix, Loader, LoaderExecutionKind, LoaderItem, ResourceParsedData, parse_resource,
};
pub use plugin::LoaderRunnerPlugin;
pub use registry::{
  LoaderContextDropRegistration, LoaderContextId, notify_loader_context_drop,
  register_loader_context_drop_listener,
};
pub use rspack_collections::{Identifiable, Identifier};
pub use runner::{LoaderResult, run_loaders};
pub use scheme::{Scheme, get_scheme};

pub const BUILTIN_LOADER_PREFIX: &str = "builtin:";
pub use cache::LoaderRunnerOptions;
pub use chain::LoaderChain;
