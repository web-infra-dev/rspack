#![feature(let_chains)]

mod ast;
mod dependency;
mod loading_plugin;
mod parser_and_generator;
mod runtime;
mod wasm_plugin;

pub use ast::*;
pub use loading_plugin::*;
pub use parser_and_generator::*;
use rspack_core::AssetInfo;
pub use runtime::*;
pub use wasm_plugin::*;

// TODO(ahabhgk): remove this
pub type ModuleIdToFileName = std::sync::Arc<
  dashmap::DashMap<
    rspack_core::ModuleIdentifier,
    (String, AssetInfo),
    std::hash::BuildHasherDefault<rspack_identifier::IdentifierHasher>,
  >,
>;
