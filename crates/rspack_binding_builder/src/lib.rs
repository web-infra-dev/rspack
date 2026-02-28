//! A Rust crate that provides the foundation for building custom Rspack bindings and plugins.
//! This crate serves as a bridge between the Rspack core functionality and custom plugin implementations.
//!
//! ## Overview
//!
//! `rspack_binding_builder` is a lightweight wrapper around `rspack_binding_api` that provides
//! essential utilities for creating custom Rspack plugins and bindings. It re-exports key components
//! from the binding API to simplify the development of custom Rspack integrations.
//!
//! ## Features
//!
//! - `plugin`: Enable SWC plugin support
//!
//! # Guide
//! [Rspack Custom binding](https://rstackjs.github.io/rspack-rust-book/custom-binding/getting-started/index.html)
// Re-export rspack_binding_api to make it available to the N-API binding
extern crate rspack_binding_api;

pub use rspack_binding_api::{CustomPluginBuilder, register_custom_plugin};
