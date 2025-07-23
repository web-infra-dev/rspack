//! This crate contains everything that should only be used for `@rspack/browser`,
//! especially some api that avoiding requesting locks in the main thread of browser.
//!
//! Always make it optional and use a "browser" feature to control it.
//! ```toml
//! [features]
//! browser = ["dep:rspack_browser"]
//!
//! [dependencies]
//! rspack_browser = { workspace = true, optional = true }
//! ```
pub mod oneshot;
pub mod panic;
