#![forbid(missing_docs)]

//! Rspack is a high performance JavaScript bundler written in Rust. It offers strong compatibility with the webpack ecosystem, allowing for seamless replacement of webpack, and provides lightning fast build speeds.
//!
//! ## Why rspack?
//!
//! Rspack was initially created to solve performance problems encountered at ByteDance, a tech company that maintains many large monolithic app projects with complex bundling requirements. Production build times had grown to ten minutes or even half an hour in some cases, and cold start times could exceed several minutes. After experimenting with many bundlers and optimization ideas, a common set of requirements emerged:
//! - **Dev mode startup performance.** npm run dev is a command that developers may invoke many times per hour. Engineering productivity suffers if startup time exceeds 10-15 seconds.
//! - **Fast builds.** npm run build is used in CI/CD pipelines and directly impacts merging productivity and application delivery time. Large applications may spend 20-30 minutes running these pipelines, and bundling time is often a major contributor.
//! - **Flexible configuration.** From experimenting with various popular bundlers, we found that one-size-fits-all configurations encountered many problems when trying to accommodate real world projects. A major advantage of webpack is its flexibility and ease of accommodating customized requirements for each project. This in turn may pose steep migration costs for legacy projects that try to migrate away from webpack.
//! - **Production optimization capabilities.** All of the existing bundling solutions also had various limitations when optimizing for a production environment, such as insufficiently fine-grained code splitting, etc. Rspack has an opportunity to rethink these optimizations from the ground up, leveraging Rust-specific features such as multithreading.
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
//! ```
//! use std::path::PathBuf;
//! use rspack::builder::{Builder, CompilerBuilder};
//! use rspack_core::Compiler;
//!
//! #[tokio::main]
//! async fn main() {
//! # let context = PathBuf::from(env!("CARGO_MANIFEST_DIR").to_string()).join("tests/fixtures/basic");
//!   let compiler = Compiler::builder()
//!     .context(context)
//!     .entry("main", "./src/index.js")
//!     .build()
//!     .unwrap();
//!
//!   let errors: Vec<_> = compiler.compilation.get_errors().collect();
//!   assert!(errors.is_empty());
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
//! - [x] Basic `CompilerBuilder` API
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
