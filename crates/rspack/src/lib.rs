#![forbid(missing_docs)]

//! Rspack is a high performance JavaScript bundler written in Rust. It offers strong compatibility with the webpack ecosystem, allowing for seamless replacement of webpack, and provides lightning fast build.
//!
//! For guide level documentation, please refer to the [Rspack Guide](https://rspack.rs/guide/start/introduction).
//!
//! ## Example
//!
//! Rspack uses [`tokio`](https://docs.rs/tokio) as the runtime for async operations. Here's an example of how to use with `Compiler`:
//!
//! ```toml
//! [dependencies]
//! tokio = { version = "1", features = ["full"] }
//! rspack = "0.3"
//! rspack_core = "0.3"
//! ```
//!
//! ```ignore
//! use std::path::PathBuf;
//!
//! use rspack::builder::{Builder, CompilerBuilder};
//! use rspack_core::Compiler;
//!
//! #[tokio::main]
//! async fn main() {
//!   use rspack_tasks::within_compiler_context_for_testing_sync;
//!   within_compiler_context_for_testing_sync(|| {
//!     let context =
//!       PathBuf::from(env!("CARGO_MANIFEST_DIR").to_string()).join("tests/fixtures/basic");
//!     let compiler = Compiler::builder()
//!       .context(context)
//!       .entry("main", "./src/index.js")
//!       .build()
//!       .unwrap();
//!
//!     let errors: Vec<_> = compiler.compilation.get_errors().collect();
//!     assert!(errors.is_empty());
//!   })
//! }
//! ```
//!
//! ## Stability
//!
//! This crate and the dependencies that this crate are relying on are not stable yet. The API may change at any time.
//! We do not guarantee any backward compatibility at the moment.
//!
//! ## Features
//!
//! Currently, there's still alot of features that are not implemented yet. Here's a list of features that are not implemented yet:
//!
//! - [x] `CompilerBuilder` API
//! - [ ] `SplitChunksPlugin` API
//! - [ ] `BundlerInfoPlugin` API
//! - [ ] `StatsPrinter` API
//! - [ ] Stable `Compiler` API
//! - [ ] Stable `Compilation` API
//! - [ ] Rust Plugin for Rspack
//! - [ ] ...
//!
//!
//!
//! To track the current stats for API, please refer to [this](https://github.com/web-infra-dev/rspack/issues/9378) GitHub issue.
pub mod builder;
