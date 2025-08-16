// Re-export rspack_binding_api to make it available to the N-API binding
extern crate rspack_binding_api;

pub use rspack_binding_api::{CustomPluginBuilder, register_custom_plugin};
