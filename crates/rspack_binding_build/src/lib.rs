//! Build script for rspack_binding_builder
//!
//! In `build.rs`, you can use `rspack_binding_build::setup` to setup the build script.
//! ```rust,ignore
//! fn main() {
//!   rspack_binding_build::setup();
//! }
//! ```
//!
//! # Guide
//! [Rspack Custom binding](https://rstackjs.github.io/rspack-rust-book/custom-binding/getting-started/index.html)

pub fn setup() {
  napi_build::setup()
}
