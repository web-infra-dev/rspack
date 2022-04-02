#![feature(async_closure)]

use std::{
    fmt::Debug,
    sync::atomic::{AtomicUsize, Ordering},
};

use crossbeam::channel::{self};

use js_ext_module::JsExtModule;
use js_module::JsModule;
use plugin_driver::PluginDriver;
use smol_str::SmolStr;
pub mod bundle_context;
pub mod graph_container;
pub mod js_ext_module;
pub mod plugin_driver;
pub mod worker;

pub mod utils;

pub mod bundler;
pub mod js_module;
pub mod plugin;
pub mod plugins;
pub mod visitors;

#[derive(Debug, Clone, Copy)]
pub enum Relation {
    AsyncImport,
    StaticImport,
}

type DepGraph = petgraph::Graph<SmolStr, Relation>;

pub enum Msg {
    DependencyReference(SmolStr, SmolStr, Relation),
    NewMod(JsModule),
    NewExtMod(JsExtModule),
}
