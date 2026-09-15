#![feature(arbitrary_self_types, unique_rc_arc)]

mod backend;
mod dependency;
mod factory;
mod module;
mod plugin;
mod utils;

pub use backend::Backend;
pub use plugin::{LazyCompilationPlugin, LazyCompilationTest, LazyCompilationTestCheck};
