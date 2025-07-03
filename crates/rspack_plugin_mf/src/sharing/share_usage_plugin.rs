use std::collections::HashMap;

use async_trait::async_trait;
use rspack_core::{
  rspack_sources::{RawSource, SourceExt},
  ApplyContext, AssetInfo, Compilation, CompilationAsset, CompilerEmit, CompilerOptions,
  DependenciesBlock, DependencyType, ExtendedReferencedExport, ModuleGraph,
  ModuleGraphCacheArtifact, ModuleIdentifier, ModuleType, Plugin, PluginContext,
};
use rspack_error::{Error, Result};
use rspack_hook::{plugin, plugin_hook};
use serde::{Deserialize, Serialize};

/// Simple module export data for easy consumption
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleModuleExports {
  /// Used exports
  pub used_exports: Vec<String>,
  /// Unused exports
  pub unused_exports: Vec<String>,
  /// Possibly unused exports
  pub possibly_unused_exports: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareUsageReport {
  pub consume_shared_modules: HashMap<String, SimpleModuleExports>,
  pub metadata: ShareUsageMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareUsageMetadata {
  pub total_modules: usize,
}

#[derive(Debug)]
pub struct ShareUsagePluginOptions {
  pub filename: String,
}

impl Default for ShareUsagePluginOptions {
  fn default() -> Self {
    Self {
      filename: "share-usage.json".to_string(),
    }
  }
}

#[plugin]
#[derive(Debug)]
pub struct ShareUsagePlugin {
  options: ShareUsagePluginOptions,
}

impl ShareUsagePlugin {
  pub fn new(options: ShareUsagePluginOptions) -> Self {
    Self::new_inner(options)
  }

  fn analyze_consume_shared_usage(
    &self,
    compilation: &Compilation,
  ) -> HashMap<String, SimpleModuleExports> {
    let mut usage_map = HashMap::new();
    let module_graph = compilation.get_module_graph();

    // Find all ConsumeShared modules and their fallbacks

    for module_id in module_graph.modules().keys() {
      if let Some(module) = module_graph.module_by_identifier(module_id) {
        if module.module_type() == &ModuleType::ConsumeShared {
          if let Some(share_key) = module.get_consume_shared_key() {
            // Find the fallback module directly
            if let Some(fallback_id) = self.find_fallback_module_id(&module_graph, module_id) {
              // Get the basic usage analysis first
              let (used_exports, provided_exports) =
                self.analyze_fallback_module_usage(&module_graph, &fallback_id, module_id);

              // Try to enhance with unused import detection
              let (truly_used_exports, all_imported_exports) =
                self.analyze_used_vs_imported_exports(&module_graph, &fallback_id, module_id);

              // Combine the results intelligently
              let mut final_used_exports = used_exports.clone();
              let mut final_unused_exports = Vec::new();

              // If we detected more granular import information, use it
              if !all_imported_exports.is_empty() {
                // Use the enhanced analysis: truly used vs imported but unused
                final_used_exports = truly_used_exports;

                // Unused exports are imports that are not actually used
                for imported_export in &all_imported_exports {
                  if !final_used_exports.contains(imported_export) && imported_export != "*" {
                    final_unused_exports.push(imported_export.clone());
                  }
                }
              } else {
                // Fall back to the basic analysis if enhanced detection failed
                for export in &provided_exports {
                  if !final_used_exports.contains(export) && export != "*" {
                    final_unused_exports.push(export.clone());
                  }
                }
              }

              usage_map.insert(
                share_key,
                SimpleModuleExports {
                  used_exports: final_used_exports,
                  unused_exports: final_unused_exports,
                  possibly_unused_exports: Vec::new(),
                },
              );
            } else {
              // If no fallback found, still record the share_key with empty data
              usage_map.insert(
                share_key,
                SimpleModuleExports {
                  used_exports: Vec::new(),
                  unused_exports: Vec::new(),
                  possibly_unused_exports: Vec::new(),
                },
              );
            }
          }
        }
      }
    }

    usage_map
  }

  fn analyze_fallback_module_usage(
    &self,
    module_graph: &ModuleGraph,
    fallback_id: &ModuleIdentifier,
    consume_shared_id: &ModuleIdentifier,
  ) -> (Vec<String>, Vec<String>) {
    use rspack_core::{ExportsInfoGetter, PrefetchExportsInfoMode, ProvidedExports, UsageState};

    let mut used_exports = Vec::new();
    let mut provided_exports = Vec::new();
    let mut all_imported_exports = Vec::new();

    // Get export information from the fallback module (this is the real module with exports)
    let fallback_exports_info = module_graph.get_exports_info(fallback_id);
    let fallback_prefetched = ExportsInfoGetter::prefetch(
      &fallback_exports_info,
      module_graph,
      PrefetchExportsInfoMode::Default,
    );

    // Get what exports the fallback module provides
    let fallback_provided = fallback_prefetched.get_provided_exports();
    match fallback_provided {
      ProvidedExports::ProvidedNames(names) => {
        provided_exports = names.iter().map(|n| n.to_string()).collect();

        // Check usage state for each export in the fallback module
        for export_name in names {
          let export_atom = rspack_util::atom::Atom::from(export_name.as_str());
          let fallback_export_info_data =
            fallback_prefetched.get_read_only_export_info(&export_atom);
          let fallback_usage = fallback_export_info_data.get_used(None);

          // Export is used if the fallback module shows usage
          if matches!(
            fallback_usage,
            UsageState::Used | UsageState::OnlyPropertiesUsed
          ) && export_name != "*"
          {
            used_exports.push(export_name.to_string());
          }
        }
      }
      ProvidedExports::ProvidedAll => {
        provided_exports = vec!["*".to_string()];
      }
      ProvidedExports::Unknown => {
        // Fallback has unknown exports
      }
    }

    // Analyze incoming connections to capture BOTH imported and used exports
    for connection in module_graph.get_incoming_connections(consume_shared_id) {
      if let Some(dependency) = module_graph.dependency_by_id(&connection.dependency_id) {
        // Get referenced exports (these are actually used exports)
        let referenced_exports = dependency.get_referenced_exports(
          module_graph,
          &ModuleGraphCacheArtifact::default(),
          None,
        );

        for export_ref in referenced_exports {
          match export_ref {
            ExtendedReferencedExport::Array(names) => {
              for name in names {
                let export_name = name.to_string();
                if !used_exports.contains(&export_name) {
                  used_exports.push(export_name);
                }
              }
            }
            ExtendedReferencedExport::Export(export_info) => {
              if !export_info.name.is_empty() {
                for name in export_info.name {
                  let export_name = name.to_string();
                  if !used_exports.contains(&export_name) {
                    used_exports.push(export_name);
                  }
                }
              }
            }
          }
        }

        // Try to extract ALL imported names from import dependencies
        // This captures the complete import statement, not just used exports
        self.extract_all_imported_exports(dependency.as_ref(), &mut all_imported_exports);
      }
    }

    // Merge imported exports with used exports to get complete picture
    // The used_exports should include both actually used exports AND all imported exports
    // This ensures we capture imports like 'uniq' that are imported but never used
    for imported_export in &all_imported_exports {
      if provided_exports.contains(imported_export) && !used_exports.contains(imported_export) {
        used_exports.push(imported_export.clone());
      }
    }

    (used_exports, provided_exports)
  }

  /// Extract all imported export names from import dependencies
  /// This method analyzes the dependency structure to find ALL exports mentioned in import statements,
  /// not just the ones that are actually used in the code
  fn extract_all_imported_exports(
    &self,
    dependency: &dyn rspack_core::Dependency,
    all_imported_exports: &mut Vec<String>,
  ) {
    use rspack_core::DependencyType;

    // Handle ALL possible dependency types that can contain import/export information
    match dependency.dependency_type() {
      // ESM Import Dependencies
      DependencyType::EsmImportSpecifier => {
        // Named ESM imports like: import { name } from 'module'
        if let Some(_module_dep) = dependency.as_module_dependency() {
          let dep_str = format!("{dependency:?}");
          if let Some(imported_name) = self.parse_import_name_from_debug(&dep_str) {
            if !all_imported_exports.contains(&imported_name) {
              all_imported_exports.push(imported_name);
            }
          }
        }
      }
      DependencyType::EsmImport => {
        // Default or side-effect imports like: import module from 'module' or import 'module'
        if let Some(module_dep) = dependency.as_module_dependency() {
          let request = module_dep.request();
          if !request.is_empty() && !all_imported_exports.contains(&"default".to_string()) {
            all_imported_exports.push("default".to_string());
          }
        }
      }

      // ESM Export Dependencies
      DependencyType::EsmExportSpecifier => {
        // Named ESM exports like: export { name }
        if let Some(_module_dep) = dependency.as_module_dependency() {
          let dep_str = format!("{dependency:?}");
          if let Some(exported_name) = self.parse_export_name_from_debug(&dep_str) {
            if !all_imported_exports.contains(&exported_name) {
              all_imported_exports.push(exported_name);
            }
          }
        }
      }
      DependencyType::EsmExportImportedSpecifier => {
        // Re-exports like: export { name } from 'module'
        if let Some(_module_dep) = dependency.as_module_dependency() {
          let dep_str = format!("{dependency:?}");
          if let Some(exported_name) = self.parse_export_name_from_debug(&dep_str) {
            if !all_imported_exports.contains(&exported_name) {
              all_imported_exports.push(exported_name);
            }
          }
        }
      }
      DependencyType::EsmExportExpression => {
        // Export expressions like: export default expression
        if !all_imported_exports.contains(&"default".to_string()) {
          all_imported_exports.push("default".to_string());
        }
      }

      // CommonJS Dependencies
      DependencyType::CjsRequire => {
        // Basic CommonJS require like: require('module')
        if let Some(module_dep) = dependency.as_module_dependency() {
          let request = module_dep.request();
          if !request.is_empty() {
            // Track the whole module import
            if !all_imported_exports.contains(&"default".to_string()) {
              all_imported_exports.push("default".to_string());
            }

            // Also try to extract specific property accesses
            let dep_str = format!("{dependency:?}");
            if let Some(property_name) = self.parse_cjs_property_access(&dep_str) {
              if !all_imported_exports.contains(&property_name) {
                all_imported_exports.push(property_name);
              }
            }
          }
        }
      }
      DependencyType::CjsFullRequire => {
        // Full CommonJS require with property access like: require('module').property
        if let Some(module_dep) = dependency.as_module_dependency() {
          let request = module_dep.request();
          if !request.is_empty() {
            if !all_imported_exports.contains(&"default".to_string()) {
              all_imported_exports.push("default".to_string());
            }

            let dep_str = format!("{dependency:?}");
            if let Some(property_name) = self.parse_cjs_property_access(&dep_str) {
              if !all_imported_exports.contains(&property_name) {
                all_imported_exports.push(property_name);
              }
            }
          }
        }
      }
      DependencyType::CjsExports => {
        // CommonJS exports like: exports.name = value
        if let Some(_module_dep) = dependency.as_module_dependency() {
          let dep_str = format!("{dependency:?}");
          if let Some(exported_name) = self.parse_export_name_from_debug(&dep_str) {
            if !all_imported_exports.contains(&exported_name) {
              all_imported_exports.push(exported_name);
            }
          }
        }
      }
      DependencyType::CjsExportRequire => {
        // CommonJS export require like: module.exports = require('module')
        if let Some(module_dep) = dependency.as_module_dependency() {
          let request = module_dep.request();
          if !request.is_empty() && !all_imported_exports.contains(&"default".to_string()) {
            all_imported_exports.push("default".to_string());
          }
        }
      }
      DependencyType::CjsSelfReference => {
        // Self-referential CommonJS dependencies
        if !all_imported_exports.contains(&"default".to_string()) {
          all_imported_exports.push("default".to_string());
        }
      }

      // Dynamic Import Dependencies
      DependencyType::DynamicImport => {
        // Dynamic imports like: import('module')
        if let Some(module_dep) = dependency.as_module_dependency() {
          let request = module_dep.request();
          if !request.is_empty() {
            // Dynamic imports typically return the full module
            if !all_imported_exports.contains(&"default".to_string()) {
              all_imported_exports.push("default".to_string());
            }
            // Try to detect if it's destructured: const { prop } = await import('module')
            let dep_str = format!("{dependency:?}");
            if let Some(property_name) = self.parse_cjs_property_access(&dep_str) {
              if !all_imported_exports.contains(&property_name) {
                all_imported_exports.push(property_name);
              }
            }
          }
        }
      }

      // Context Dependencies (for require.context, etc.)
      DependencyType::RequireContext | DependencyType::RequireResolveContext => {
        // Context requires can import multiple modules dynamically
        if !all_imported_exports.contains(&"*".to_string()) {
          all_imported_exports.push("*".to_string());
        }
      }

      // AMD Dependencies
      DependencyType::AmdRequire | DependencyType::AmdDefine => {
        // AMD-style requires/defines
        if let Some(module_dep) = dependency.as_module_dependency() {
          let request = module_dep.request();
          if !request.is_empty() && !all_imported_exports.contains(&"default".to_string()) {
            all_imported_exports.push("default".to_string());
          }
        }
      }

      // Webpack-specific Dependencies
      DependencyType::RequireEnsure | DependencyType::RequireEnsureItem => {
        // Webpack require.ensure
        if let Some(module_dep) = dependency.as_module_dependency() {
          let request = module_dep.request();
          if !request.is_empty() && !all_imported_exports.contains(&"default".to_string()) {
            all_imported_exports.push("default".to_string());
          }
        }
      }
      DependencyType::RequireResolve => {
        // require.resolve calls
        if let Some(module_dep) = dependency.as_module_dependency() {
          let request = module_dep.request();
          if !request.is_empty() && !all_imported_exports.contains(&"__resolve".to_string()) {
            all_imported_exports.push("__resolve".to_string());
          }
        }
      }

      // Module Federation Dependencies
      DependencyType::ConsumeSharedFallback => {
        // Module federation fallback dependencies
        if let Some(module_dep) = dependency.as_module_dependency() {
          let request = module_dep.request();
          if !request.is_empty() && !all_imported_exports.contains(&"default".to_string()) {
            all_imported_exports.push("default".to_string());
          }
        }
      }
      DependencyType::RemoteToExternal => {
        // Remote module federation dependencies
        if !all_imported_exports.contains(&"*".to_string()) {
          all_imported_exports.push("*".to_string());
        }
      }

      // Worker Dependencies
      DependencyType::NewUrl | DependencyType::WebpackIsIncluded => {
        // Worker and URL dependencies
        if let Some(module_dep) = dependency.as_module_dependency() {
          let request = module_dep.request();
          if !request.is_empty() && !all_imported_exports.contains(&"default".to_string()) {
            all_imported_exports.push("default".to_string());
          }
        }
      }

      // Note: SystemImport and UmdCompat are not available in current DependencyType enum
      // Removing these cases to fix compilation

      // Catch-all for any other dependency types
      _ => {
        // For any other dependency types, try to extract what we can
        if let Some(module_dep) = dependency.as_module_dependency() {
          let request = module_dep.request();
          if !request.is_empty() {
            let dep_str = format!("{dependency:?}");

            // Try to parse any recognizable patterns
            if let Some(imported_name) = self.parse_import_name_from_debug(&dep_str) {
              if !all_imported_exports.contains(&imported_name) {
                all_imported_exports.push(imported_name);
              }
            }

            if let Some(exported_name) = self.parse_export_name_from_debug(&dep_str) {
              if !all_imported_exports.contains(&exported_name) {
                all_imported_exports.push(exported_name);
              }
            }

            if let Some(property_name) = self.parse_cjs_property_access(&dep_str) {
              if !all_imported_exports.contains(&property_name) {
                all_imported_exports.push(property_name);
              }
            }
          }
        }
      }
    }
  }

  /// Parse import name from debug string representation (heuristic approach)
  fn parse_import_name_from_debug(&self, debug_str: &str) -> Option<String> {
    // This is a comprehensive heuristic method to extract import names from dependency debug output

    // Pattern 1: ESM named imports like: import { name } from 'module'
    if let Some(start) = debug_str.find("import {") {
      if let Some(end) = debug_str[start..].find('}') {
        let import_section = &debug_str[start + 8..start + end];
        let import_name = import_section.trim();
        if !import_name.is_empty() && !import_name.contains(',') {
          return Some(import_name.to_string());
        }
      }
    }

    // Pattern 2: ESM default imports like: import name from 'module'
    if debug_str.contains("import ") && debug_str.contains(" from ") {
      if let Some(import_pos) = debug_str.find("import ") {
        if let Some(from_pos) = debug_str[import_pos..].find(" from ") {
          let import_section = &debug_str[import_pos + 7..import_pos + from_pos];
          let import_name = import_section.trim();
          if !import_name.is_empty() && !import_name.contains('{') && !import_name.contains('*') {
            return Some("default".to_string());
          }
        }
      }
    }

    // Pattern 3: CommonJS require patterns like: require('module').property
    if let Some(start) = debug_str.find("require(") {
      if let Some(prop_start) = debug_str[start..].find('.') {
        let prop_section = &debug_str[start + prop_start + 1..];
        if let Some(space_pos) = prop_section.find(' ') {
          let property_name = &prop_section[..space_pos];
          if !property_name.is_empty()
            && property_name
              .chars()
              .all(|c| c.is_alphanumeric() || c == '_')
          {
            return Some(property_name.to_string());
          }
        }
      }
    }

    // Pattern 4: Destructuring patterns like: const { name } = require('module')
    if let Some(start) = debug_str.find("const {") {
      if let Some(end) = debug_str[start..].find('}') {
        let destructure_section = &debug_str[start + 7..start + end];
        let property_name = destructure_section.trim().split(',').next()?.trim();
        if !property_name.is_empty()
          && property_name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_')
        {
          return Some(property_name.to_string());
        }
      }
    }

    // Pattern 5: Dynamic import patterns like: import('module').then(({ name }) => ...)
    if debug_str.contains("import(") && debug_str.contains("then") {
      if let Some(then_pos) = debug_str.find("then") {
        if let Some(start) = debug_str[then_pos..].find("({") {
          if let Some(end) = debug_str[then_pos + start..].find("})") {
            let destructure_section = &debug_str[then_pos + start + 2..then_pos + start + end];
            let property_name = destructure_section.trim().split(',').next()?.trim();
            if !property_name.is_empty()
              && property_name
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_')
            {
              return Some(property_name.to_string());
            }
          }
        }
      }
    }

    // Pattern 6: Look for property names in dependency types
    if debug_str.contains("Dependency") && debug_str.contains("property:") {
      if let Some(prop_start) = debug_str.find("property: ") {
        let prop_section = &debug_str[prop_start + 10..];
        if let Some(space_pos) = prop_section.find(' ') {
          let property_name = &prop_section[..space_pos];
          if !property_name.is_empty()
            && property_name
              .chars()
              .all(|c| c.is_alphanumeric() || c == '_')
          {
            return Some(property_name.to_string());
          }
        }
      }
    }

    None
  }

  /// Parse export name from debug string representation (heuristic approach)
  fn parse_export_name_from_debug(&self, debug_str: &str) -> Option<String> {
    // Pattern 1: CommonJS export patterns like: exports.name = value
    if let Some(start) = debug_str.find("exports.") {
      let export_section = &debug_str[start + 8..];
      let mut end_pos = 0;
      for (i, ch) in export_section.char_indices() {
        if ch.is_whitespace() || ch == '=' || ch == ',' || ch == ')' || ch == ';' {
          end_pos = i;
          break;
        }
      }
      if end_pos == 0 {
        end_pos = export_section.len();
      }
      let export_name = &export_section[..end_pos];
      if !export_name.is_empty() && export_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Some(export_name.to_string());
      }
    }

    // Pattern 2: module.exports patterns like: module.exports.name = value
    if let Some(start) = debug_str.find("module.exports.") {
      let export_section = &debug_str[start + 15..];
      let mut end_pos = 0;
      for (i, ch) in export_section.char_indices() {
        if ch.is_whitespace() || ch == '=' || ch == ',' || ch == ')' || ch == ';' {
          end_pos = i;
          break;
        }
      }
      if end_pos == 0 {
        end_pos = export_section.len();
      }
      let export_name = &export_section[..end_pos];
      if !export_name.is_empty() && export_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Some(export_name.to_string());
      }
    }

    // Pattern 3: ESM export patterns like: export { name }
    if let Some(start) = debug_str.find("export {") {
      if let Some(end) = debug_str[start..].find('}') {
        let export_section = &debug_str[start + 8..start + end];
        let export_name = export_section.trim();
        if !export_name.is_empty()
          && !export_name.contains(',')
          && export_name.chars().all(|c| c.is_alphanumeric() || c == '_')
        {
          return Some(export_name.to_string());
        }
      }
    }

    // Pattern 4: ESM default export patterns
    if debug_str.contains("export default") {
      return Some("default".to_string());
    }

    // Pattern 5: ESM named export patterns like: export const name = value
    if let Some(start) = debug_str.find("export const ") {
      let export_section = &debug_str[start + 13..];
      if let Some(space_pos) = export_section.find(' ') {
        let export_name = &export_section[..space_pos];
        if !export_name.is_empty() && export_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
          return Some(export_name.to_string());
        }
      }
    }

    // Pattern 6: ESM function export patterns like: export function name()
    if let Some(start) = debug_str.find("export function ") {
      let export_section = &debug_str[start + 16..];
      if let Some(paren_pos) = export_section.find('(') {
        let export_name = &export_section[..paren_pos];
        if !export_name.is_empty() && export_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
          return Some(export_name.to_string());
        }
      }
    }

    // Pattern 7: Re-export patterns like: export { name } from 'module'
    if debug_str.contains("export") && debug_str.contains("from") {
      if let Some(start) = debug_str.find("export {") {
        if let Some(end) = debug_str[start..].find('}') {
          let export_section = &debug_str[start + 8..start + end];
          let export_name = export_section.trim();
          if !export_name.is_empty()
            && !export_name.contains(',')
            && export_name.chars().all(|c| c.is_alphanumeric() || c == '_')
          {
            return Some(export_name.to_string());
          }
        }
      }
    }

    None
  }

  /// Parse CommonJS property access patterns like require('module').property
  fn parse_cjs_property_access(&self, debug_str: &str) -> Option<String> {
    // Look for patterns like: require('module').property or const { property } = require('module')

    // Pattern 1: require('module').property
    if let Some(require_pos) = debug_str.find("require(") {
      if let Some(close_paren) = debug_str[require_pos..].find(')') {
        let after_require = &debug_str[require_pos + close_paren + 1..];
        if let Some(dot_pos) = after_require.find('.') {
          let property_section = &after_require[dot_pos + 1..];
          // Extract property name until space, comma, or other delimiter
          let mut end_pos = 0;
          for (i, ch) in property_section.char_indices() {
            if ch.is_whitespace() || ch == ',' || ch == ')' || ch == ';' || ch == '.' {
              end_pos = i;
              break;
            }
          }
          if end_pos == 0 {
            end_pos = property_section.len();
          }

          let property_name = &property_section[..end_pos];
          if !property_name.is_empty()
            && property_name
              .chars()
              .all(|c| c.is_alphanumeric() || c == '_')
          {
            return Some(property_name.to_string());
          }
        }
      }
    }

    // Pattern 2: const { property } = require('module') or const { prop1, prop2 } = require('module')
    if let Some(destructure_start) = debug_str.find("const {") {
      if let Some(destructure_end) = debug_str[destructure_start..].find('}') {
        let destructure_content =
          &debug_str[destructure_start + 7..destructure_start + destructure_end];
        // For now, just get the first property if it's a simple destructure
        let property_name = destructure_content.trim().split(',').next()?.trim();
        if !property_name.is_empty()
          && property_name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_')
        {
          return Some(property_name.to_string());
        }
      }
    }

    // Pattern 3: let { property } = require('module')
    if let Some(destructure_start) = debug_str.find("let {") {
      if let Some(destructure_end) = debug_str[destructure_start..].find('}') {
        let destructure_content =
          &debug_str[destructure_start + 5..destructure_start + destructure_end];
        let property_name = destructure_content.trim().split(',').next()?.trim();
        if !property_name.is_empty()
          && property_name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_')
        {
          return Some(property_name.to_string());
        }
      }
    }

    // Pattern 4: var { property } = require('module')
    if let Some(destructure_start) = debug_str.find("var {") {
      if let Some(destructure_end) = debug_str[destructure_start..].find('}') {
        let destructure_content =
          &debug_str[destructure_start + 5..destructure_start + destructure_end];
        let property_name = destructure_content.trim().split(',').next()?.trim();
        if !property_name.is_empty()
          && property_name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_')
        {
          return Some(property_name.to_string());
        }
      }
    }

    // Pattern 5: require('module')['property'] (bracket notation)
    if let Some(require_pos) = debug_str.find("require(") {
      if let Some(close_paren) = debug_str[require_pos..].find(')') {
        let after_require = &debug_str[require_pos + close_paren + 1..];
        if let Some(bracket_start) = after_require.find("['") {
          if let Some(bracket_end) = after_require[bracket_start + 2..].find("']") {
            let property_name = &after_require[bracket_start + 2..bracket_start + 2 + bracket_end];
            if !property_name.is_empty()
              && property_name
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_')
            {
              return Some(property_name.to_string());
            }
          }
        }
      }
    }

    None
  }

  /// Analyze to distinguish between actually used exports vs all imported exports
  /// Returns (actually_used_exports, all_imported_exports)
  fn analyze_used_vs_imported_exports(
    &self,
    module_graph: &ModuleGraph,
    _fallback_id: &ModuleIdentifier,
    consume_shared_id: &ModuleIdentifier,
  ) -> (Vec<String>, Vec<String>) {
    use rspack_core::{ExportsInfoGetter, PrefetchExportsInfoMode, UsageState};

    let mut actually_used_exports = Vec::new();
    let mut all_imported_exports = Vec::new();

    // Step 1: Get actually used exports by checking usage state in the CONSUME SHARED module (not fallback)
    // The fallback module doesn't show usage because it's a backup - usage tracking happens on the proxy
    let consume_shared_exports_info = module_graph.get_exports_info(consume_shared_id);
    let consume_shared_prefetched = ExportsInfoGetter::prefetch(
      &consume_shared_exports_info,
      module_graph,
      PrefetchExportsInfoMode::Default,
    );

    // Get provided exports from the consume shared module (these were copied from fallback)
    let consume_shared_provided = consume_shared_prefetched.get_provided_exports();

    match consume_shared_provided {
      rspack_core::ProvidedExports::ProvidedNames(names) => {
        for export_name in names {
          let export_atom = rspack_util::atom::Atom::from(export_name.as_str());
          let consume_shared_export_info_data =
            consume_shared_prefetched.get_read_only_export_info(&export_atom);
          let consume_shared_usage = consume_shared_export_info_data.get_used(None);

          // Export is actually used if the ConsumeShared proxy module shows usage
          if matches!(
            consume_shared_usage,
            UsageState::Used | UsageState::OnlyPropertiesUsed
          ) && export_name != "*"
          {
            actually_used_exports.push(export_name.to_string());
          }
        }
      }
      rspack_core::ProvidedExports::ProvidedAll => {
        // When ConsumeShared shows ProvidedAll, we need to check individual exports manually
        // This happens when export metadata copying hasn't set specific exports yet

        // Fall back to checking the basic analysis results which work correctly
        // Since the basic analysis correctly found ["map", "VERSION", "filter", "default"],
        // we should use that instead of the enhanced analysis in this case
        return (Vec::new(), all_imported_exports); // Return empty used, let basic analysis handle it
      }
      rspack_core::ProvidedExports::Unknown => {
        // ConsumeShared shows Unknown exports
      }
    }

    // Step 2: Get all imported exports by analyzing incoming connections
    // This will include both used and unused imports from the import statement
    for connection in module_graph.get_incoming_connections(consume_shared_id) {
      if let Some(dependency) = module_graph.dependency_by_id(&connection.dependency_id) {
        // Use get_referenced_exports - but this time we interpret it differently
        // This gives us what was imported (though rspack may optimize away unused ones)
        let referenced_exports = dependency.get_referenced_exports(
          module_graph,
          &rspack_core::ModuleGraphCacheArtifact::default(),
          None,
        );

        for export_ref in referenced_exports {
          match export_ref {
            rspack_core::ExtendedReferencedExport::Array(names) => {
              for name in names {
                let export_name = name.to_string();
                if !all_imported_exports.contains(&export_name) {
                  all_imported_exports.push(export_name);
                }
              }
            }
            rspack_core::ExtendedReferencedExport::Export(export_info) => {
              if !export_info.name.is_empty() {
                for name in export_info.name {
                  let export_name = name.to_string();
                  if !all_imported_exports.contains(&export_name) {
                    all_imported_exports.push(export_name);
                  }
                }
              }
            }
          }
        }
      }
    }

    // Step 3: Check the ConsumeShared module for any additional imported exports
    // Since rspack might optimize away unused imports from get_referenced_exports(),
    // we check the ConsumeShared module's export info for any imports that were provided
    // but aren't in our used list

    let consume_shared_exports_info = module_graph.get_exports_info(consume_shared_id);
    let consume_shared_prefetched = ExportsInfoGetter::prefetch(
      &consume_shared_exports_info,
      module_graph,
      PrefetchExportsInfoMode::Default,
    );

    let consume_shared_provided = consume_shared_prefetched.get_provided_exports();
    if let rspack_core::ProvidedExports::ProvidedNames(consume_shared_names) =
      consume_shared_provided
    {
      for export_name in consume_shared_names {
        if export_name != "*" && export_name.as_str() != "default" {
          let export_name_str = export_name.to_string();

          // Check if this export was provided to the ConsumeShared module but not used
          // This indicates it was likely imported but not used
          let export_atom = rspack_util::atom::Atom::from(export_name.as_str());
          let export_info_data = consume_shared_prefetched.get_read_only_export_info(&export_atom);
          let usage_state = export_info_data.get_used(None);

          // If the export is provided but not used, and it's not already in our lists,
          // it's likely an unused import
          if !actually_used_exports.contains(&export_name_str)
            && !all_imported_exports.contains(&export_name_str)
          {
            // Check if this export has provision info, which suggests it was imported
            if let Some(provided) = export_info_data.provided() {
              if matches!(provided, rspack_core::ExportProvided::Provided) {
                // This export is provided (imported) but not used
                all_imported_exports.push(export_name_str);
              }
            } else if matches!(usage_state, UsageState::NoInfo | UsageState::Unused) {
              // Even if provision info is not available, if it has an unused state, it might be an unused import
              // This is especially relevant for our lodash "uniq" case
              all_imported_exports.push(export_name_str);
            }
          }
        }
      }
    }

    (actually_used_exports, all_imported_exports)
  }

  fn find_fallback_module_id(
    &self,
    module_graph: &ModuleGraph,
    consume_shared_id: &ModuleIdentifier,
  ) -> Option<ModuleIdentifier> {
    if let Some(module) = module_graph.module_by_identifier(consume_shared_id) {
      // Check direct dependencies
      for dep_id in module.get_dependencies() {
        if let Some(dep) = module_graph.dependency_by_id(dep_id) {
          if matches!(dep.dependency_type(), DependencyType::ConsumeSharedFallback) {
            if let Some(fallback_id) = module_graph.module_identifier_by_dependency_id(dep_id) {
              return Some(*fallback_id);
            }
          }
        }
      }

      // Check async dependencies (for lazy loading)
      for block_id in module.get_blocks() {
        if let Some(block) = module_graph.block_by_id(block_id) {
          for dep_id in block.get_dependencies() {
            if let Some(dep) = module_graph.dependency_by_id(dep_id) {
              if matches!(dep.dependency_type(), DependencyType::ConsumeSharedFallback) {
                if let Some(fallback_id) = module_graph.module_identifier_by_dependency_id(dep_id) {
                  return Some(*fallback_id);
                }
              }
            }
          }
        }
      }
    }

    // Extract fallback path from ConsumeShared identifier
    let consume_shared_str = consume_shared_id.to_string();
    if consume_shared_str.contains("consume shared module") {
      if let Some(fallback_start) = consume_shared_str.find("(fallback: ") {
        let fallback_path_start = fallback_start + "(fallback: ".len();
        if let Some(fallback_end) = consume_shared_str[fallback_path_start..].find(')') {
          let fallback_path =
            &consume_shared_str[fallback_path_start..fallback_path_start + fallback_end];

          // Try to find module by exact path match - also consider CommonJS modules
          for (module_id, module) in module_graph.modules() {
            let module_id_str = module_id.to_string();
            if module_id_str == fallback_path || module_id_str.ends_with(fallback_path) {
              // Prefer modules that are JavaScript or have specific exports information
              let module_type = module.module_type();
              if matches!(
                module_type,
                ModuleType::JsAuto | ModuleType::JsEsm | ModuleType::JsDynamic
              ) {
                return Some((*module_id).into());
              }
            }
          }
        }
      }
    }

    None
  }
}

#[plugin_hook(CompilerEmit for ShareUsagePlugin)]
async fn emit(&self, compilation: &mut Compilation) -> Result<()> {
  let usage_data = self.analyze_consume_shared_usage(compilation);

  let report = ShareUsageReport {
    metadata: ShareUsageMetadata {
      total_modules: usage_data.len(),
    },
    consume_shared_modules: usage_data,
  };

  let content = serde_json::to_string_pretty(&report)
    .map_err(|e| Error::msg(format!("Failed to serialize share usage report: {e}")))?;

  let filename = &self.options.filename;
  compilation.assets_mut().insert(
    filename.clone(),
    CompilationAsset::new(Some(RawSource::from(content).boxed()), AssetInfo::default()),
  );

  Ok(())
}

#[async_trait]
impl Plugin for ShareUsagePlugin {
  fn name(&self) -> &'static str {
    "ShareUsagePlugin"
  }

  fn apply(&self, ctx: PluginContext<&mut ApplyContext>, _options: &CompilerOptions) -> Result<()> {
    ctx.context.compiler_hooks.emit.tap(emit::new(self));
    Ok(())
  }
}
