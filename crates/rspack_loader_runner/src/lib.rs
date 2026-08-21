#![feature(string_from_utf8_lossy_owned)]

mod cache;
mod content;
mod context;
mod loader;
mod plugin;
mod runner;
mod scheme;

pub use content::{AdditionalData, Content, DescriptionData, ParseMeta, ResourceData};
#[doc(hidden)]
pub use content::{ParseMetaValue, ParseMetaValueRegistration};
pub use context::{LoaderContext, State};
pub use loader::{DisplayWithSuffix, Loader, LoaderItem, ResourceParsedData, parse_resource};
pub use plugin::LoaderRunnerPlugin;
pub use rspack_collections::{Identifiable, Identifier};
pub use runner::{LoaderResult, run_loaders};
pub use scheme::{Scheme, get_scheme};

pub const BUILTIN_LOADER_PREFIX: &str = "builtin:";
pub use cache::LoaderRunnerOptions;
