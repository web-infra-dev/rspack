//! # Tests for HoistContainerReferencesPlugin
//!
//! Unit tests validating the plugin's behavior for Module Federation scenarios.
//!
//! Note: Full test implementation requires integration with rspack's test framework
//! and proper mock compilation setup. This serves as a placeholder for comprehensive
//! test coverage similar to the external PR.

#[cfg(test)]
mod tests {
  use rspack_core::Plugin;

  use super::super::hoist_container_references_plugin::HoistContainerReferencesPlugin;

  #[test]
  fn test_plugin_creation() {
    let plugin = HoistContainerReferencesPlugin::default();
    assert_eq!(plugin.name(), "HoistContainerReferencesPlugin");
  }
}
