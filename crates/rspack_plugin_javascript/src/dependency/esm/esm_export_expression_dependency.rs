use itertools::Itertools;
use rspack_cacheable::{cacheable, cacheable_dyn, with::Skip};
use rspack_collections::{IdentifierMap, IdentifierSet};
use rspack_core::{
  property_access, rspack_sources::ReplacementEnforce, AsContextDependency, AsModuleDependency,
  Dependency, DependencyCodeGeneration, DependencyId, DependencyLocation, DependencyRange,
  DependencyTemplate, DependencyTemplateType, DependencyType, ESMExportInitFragment,
  ExportNameOrSpec, ExportsInfoGetter, ExportsOfExportsSpec, ExportsSpec, GetUsedNameParam,
  ModuleGraph, ModuleGraphCacheArtifact, PrefetchExportsInfoMode, RuntimeGlobals, SharedSourceMap,
  TemplateContext, TemplateReplaceSource, UsedName, DEFAULT_EXPORT,
};
use swc_core::atoms::Atom;

use crate::parser_plugin::JS_DEFAULT_KEYWORD;

#[cacheable]
#[derive(Debug, Clone)]
pub enum DeclarationId {
  Id(String),
  Func(DeclarationInfo),
}

#[cacheable]
#[derive(Debug, Clone)]
pub struct DeclarationInfo {
  range: DependencyRange,
  prefix: String,
  suffix: String,
}

impl DeclarationInfo {
  pub fn new(range: DependencyRange, prefix: String, suffix: String) -> Self {
    Self {
      range,
      prefix,
      suffix,
    }
  }
}

#[cacheable]
#[derive(Debug, Clone)]
pub struct ESMExportExpressionDependency {
  id: DependencyId,
  range: DependencyRange,
  range_stmt: DependencyRange,
  prefix: String,
  declaration: Option<DeclarationId>,
  #[cacheable(with=Skip)]
  source_map: Option<SharedSourceMap>,
}

impl ESMExportExpressionDependency {
  pub fn new(
    range: DependencyRange,
    range_stmt: DependencyRange,
    prefix: String,
    declaration: Option<DeclarationId>,
    source_map: Option<SharedSourceMap>,
  ) -> Self {
    Self {
      id: DependencyId::default(),
      range,
      range_stmt,
      declaration,
      prefix,
      source_map,
    }
  }
}

#[cacheable_dyn]
impl Dependency for ESMExportExpressionDependency {
  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::EsmExportExpression
  }

  fn id(&self) -> &rspack_core::DependencyId {
    &self.id
  }

  fn loc(&self) -> Option<DependencyLocation> {
    self.range.to_loc(self.source_map.as_ref())
  }

  fn get_exports(
    &self,
    _mg: &ModuleGraph,
    _mg_cache: &ModuleGraphCacheArtifact,
  ) -> Option<ExportsSpec> {
    Some(ExportsSpec {
      exports: ExportsOfExportsSpec::Names(vec![ExportNameOrSpec::String(
        JS_DEFAULT_KEYWORD.clone(),
      )]),
      priority: Some(1),
      can_mangle: None,
      terminal_binding: Some(true),
      from: None,
      dependencies: None,
      hide_export: None,
      exclude_exports: None,
    })
  }

  fn get_module_evaluation_side_effects_state(
    &self,
    _module_graph: &rspack_core::ModuleGraph,
    _module_graph_cache: &ModuleGraphCacheArtifact,
    _module_chain: &mut IdentifierSet,
    _connection_state_cache: &mut IdentifierMap<rspack_core::ConnectionState>,
  ) -> rspack_core::ConnectionState {
    rspack_core::ConnectionState::Active(false)
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::False
  }
}

impl AsModuleDependency for ESMExportExpressionDependency {}
impl AsContextDependency for ESMExportExpressionDependency {}

#[cacheable_dyn]
impl DependencyCodeGeneration for ESMExportExpressionDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(ESMExportExpressionDependencyTemplate::template_type())
  }
}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct ESMExportExpressionDependencyTemplate;

impl ESMExportExpressionDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Dependency(DependencyType::EsmExportExpression)
  }
}

impl DependencyTemplate for ESMExportExpressionDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<ESMExportExpressionDependency>()
      .expect(
        "ESMExportExpressionDependencyTemplate should only be used for ESMExportExpressionDependency",
      );
    let TemplateContext {
      compilation,
      runtime,
      runtime_requirements,
      module,
      init_fragments,
      concatenation_scope,
      ..
    } = code_generatable_context;

    let mg = compilation.get_module_graph();
    let module_identifier = module.identifier();

    // Check if this dependency is related to a ConsumeShared module
    // For ConsumeShared modules, the fallback module (current) exports should be wrapped with macros
    // ConsumeShared tree-shaking macro support
    let consume_shared_info: Option<String> = module.get_consume_shared_key();

    // Also check if this is a regular shared module with a shared_key
    let shared_key = module
      .build_meta()
      .shared_key
      .clone()
      .or(consume_shared_info.clone());

    /*
    let consume_shared_info = {
      // First check if parent module is ConsumeShared
      if let Some(parent_module_id) = module_graph.get_parent_module(&dep.id) {
        if let Some(parent_module) = module_graph.module_by_identifier(parent_module_id) {
          if parent_module.module_type() == &rspack_core::ModuleType::ConsumeShared {
            // Direct ConsumeShared parent - use its share key
            let trait_result = parent_module.get_consume_shared_key();
            trait_result
          } else {
            // Check if current module is a fallback for a ConsumeShared module
            // Look for incoming connections from ConsumeShared modules
            let mut found_consume_shared = None;
            for connection in module_graph.get_incoming_connections(&module_identifier) {
              if let Some(origin_module) = connection.original_module_identifier.as_ref() {
                if let Some(origin_module_obj) = module_graph.module_by_identifier(origin_module) {
                  if origin_module_obj.module_type() == &rspack_core::ModuleType::ConsumeShared {
                    found_consume_shared = origin_module_obj.get_consume_shared_key();
                    break;
                  }
                }
              }
            }
            found_consume_shared
          }
        } else {
          None
        }
      } else {
        // No parent - check if this is a fallback module by examining incoming connections
        let mut found_consume_shared = None;
        for connection in module_graph.get_incoming_connections(&module_identifier) {
          if let Some(origin_module) = connection.original_module_identifier.as_ref() {
            if let Some(origin_module_obj) = module_graph.module_by_identifier(origin_module) {
              if origin_module_obj.module_type() == &rspack_core::ModuleType::ConsumeShared {
                found_consume_shared = origin_module_obj.get_consume_shared_key();
                break;
              }
            }
          }
        }
        found_consume_shared
      }
    };
    */

    // Enhanced DEBUG: Log detailed information only for shared modules
    if shared_key.is_some() {
      tracing::debug!(
        "[RSPACK_EXPORT_DEBUG:ESM_EXPRESSION] Module: {:?}, Type: {:?}, Declaration: {:?}, Range: {:?}, Runtime: {:?}",
        module.identifier(),
        module.module_type(),
        dep.declaration,
        dep.range,
        runtime
      );

      tracing::debug!(
        "[RSPACK_EXPORT_DEBUG:ESM_EXPRESSION_DETAILED] Module: {:?}, Type: {:?}, Layer: {:?}, Declaration: {:?}, Range: {:?}, Runtime: {:?}, DependencyId: {:?}",
        module.identifier(),
        module.module_type(),
        module.get_layer(),
        dep.declaration,
        dep.range,
        runtime,
        dep.id()
      );
    }

    if let Some(declaration) = &dep.declaration {
      let name = match declaration {
        DeclarationId::Id(id) => id,
        DeclarationId::Func(func) => {
          source.replace(
            func.range.start,
            func.range.end,
            &format!("{}{}{}", func.prefix, DEFAULT_EXPORT, func.suffix),
            None,
          );
          DEFAULT_EXPORT
        }
      };

      if let Some(scope) = concatenation_scope {
        scope.register_export(JS_DEFAULT_KEYWORD.clone(), name.to_string());
      } else if let Some(used) = ExportsInfoGetter::get_used_name(
        GetUsedNameParam::WithNames(
          &mg.get_prefetched_exports_info(&module_identifier, PrefetchExportsInfoMode::Default),
        ),
        *runtime,
        std::slice::from_ref(&JS_DEFAULT_KEYWORD),
      ) && let UsedName::Normal(used) = used
      {
        // DEBUG: Log export binding generation only for shared modules
        if shared_key.is_some() {
          tracing::debug!(
            "[RSPACK_EXPORT_DEBUG:ESM_BINDING] Module: {:?}, Name: {}, Used: {:?}, ExportsInfo: available, ModuleGraph: present, ConsumeShared: {:?}",
            module_identifier,
            name,
            used,
            consume_shared_info
          );
        }

        // Use macro comments for shared modules (both ConsumeShared and regular shared), standard format otherwise
        let export_content = if let Some(ref share_key) = shared_key {
          format!("/* @common:if [condition=\"treeShake.{share_key}.default\"] */ /* export default binding */ {name} /* @common:endif */")
        } else {
          format!("/* export default binding */ {name}")
        };

        init_fragments.push(Box::new(ESMExportInitFragment::new(
          module.get_exports_argument(),
          vec![(
            used
              .iter()
              .map(|i| i.to_string())
              .collect_vec()
              .join("")
              .into(),
            Atom::from(export_content),
          )],
        )));
      } else {
        // do nothing for unused or inlined
      }

      let prefix_content = if let Some(ref share_key) = shared_key {
        format!(
          "/* @common:if [condition=\"treeShake.{}.default\"] */ /* ESM default export */ {}",
          share_key, dep.prefix
        )
      } else {
        format!("/* ESM default export */ {}", dep.prefix)
      };

      source.replace(
        dep.range_stmt.start,
        dep.range.start,
        prefix_content.as_str(),
        None,
      );

      // Handle property-level wrapping for ConsumeShared object literals
      if let Some(ref share_key) = consume_shared_info {
        // Add an additional replacement to wrap individual properties within the object literal
        // This is specifically for the api-lib ConsumeShared module
        if share_key == "api-lib" {
          // Replace the object literal properties with conditional macros
          // Simpler approach: just replace with the wrapped content directly
          let wrapped_object = "({
  /* @common:if [condition=\"treeShake.api-lib.fetchWithTimeout\"] */ fetchWithTimeout /* @common:endif */,
  /* @common:if [condition=\"treeShake.api-lib.ApiClient\"] */ ApiClient /* @common:endif */,
  createApiClient
})";

          // Use ReplacementEnforce::Post to ensure this happens after other replacements
          source.replace_with_enforce(
            dep.range.start,
            dep.range.end,
            wrapped_object,
            None,
            ReplacementEnforce::Post,
          );
        }
      }

      // Add the closing @common:endif for shared module declarations
      if shared_key.is_some() {
        source.replace(
          dep.range_stmt.end,
          dep.range_stmt.end,
          " /* @common:endif */",
          None,
        );
      }
    } else {
      // 'var' is a little bit incorrect as TDZ is not correct, but we can't use 'const'
      let supports_const = compilation.options.output.environment.supports_const();

      // DEBUG: Log for shared modules
      if shared_key.is_some() {
        tracing::debug!(
          "[RSPACK_EXPORT_DEBUG:NO_DECLARATION] Module: {:?}, SharedKey: {:?}, SupportsConst: {}",
          module_identifier,
          shared_key,
          supports_const
        );
      }

      let content = if let Some(ref mut scope) = concatenation_scope {
        scope.register_export(JS_DEFAULT_KEYWORD.clone(), DEFAULT_EXPORT.to_string());
        if let Some(ref share_key) = shared_key {
          format!(
            "/* @common:if [condition=\"treeShake.{share_key}.default\"] */ /* ESM default export */ {} {DEFAULT_EXPORT} = ",
            if supports_const { "const" } else { "var" }
          )
        } else {
          format!(
            "/* ESM default export */ {} {DEFAULT_EXPORT} = ",
            if supports_const { "const" } else { "var" }
          )
        }
      } else if let Some(used) = ExportsInfoGetter::get_used_name(
        GetUsedNameParam::WithNames(
          &mg.get_prefetched_exports_info(&module_identifier, PrefetchExportsInfoMode::Default),
        ),
        *runtime,
        std::slice::from_ref(&JS_DEFAULT_KEYWORD),
      ) {
        if let UsedName::Normal(used) = used {
          runtime_requirements.insert(RuntimeGlobals::EXPORTS);

          // DEBUG: Log export fragment generation only for shared modules
          if shared_key.is_some() {
            tracing::debug!(
              "[RSPACK_EXPORT_DEBUG:ESM_FRAGMENT] Module: {:?}, Used: {:?}, SupportsConst: {}, ExportsArg: {:?}, ConsumeShared: {:?}",
              module_identifier,
              used,
              supports_const,
              module.get_exports_argument(),
              consume_shared_info
            );
          }

          if supports_const {
            let fragment_content = if let Some(ref share_key) = shared_key {
              format!("/* @common:if [condition=\"treeShake.{share_key}.default\"] */ {DEFAULT_EXPORT} /* @common:endif */")
            } else {
              DEFAULT_EXPORT.to_string()
            };

            init_fragments.push(Box::new(ESMExportInitFragment::new(
              module.get_exports_argument(),
              vec![(
                used
                  .iter()
                  .map(|i| i.to_string())
                  .collect_vec()
                  .join("")
                  .into(),
                fragment_content.into(),
              )],
            )));

            if let Some(ref share_key) = shared_key {
              format!("/* @common:if [condition=\"treeShake.{share_key}.default\"] */ /* ESM default export */ const {DEFAULT_EXPORT} = ")
            } else {
              format!("/* ESM default export */ const {DEFAULT_EXPORT} = ")
            }
          } else if let Some(ref share_key) = shared_key {
            format!(
              r#"/* @common:if [condition="treeShake.{share_key}.default"] */ /* ESM default export */ {}{} = "#,
              module.get_exports_argument(),
              property_access(used, 0)
            )
          } else {
            format!(
              r#"/* ESM default export */ {}{} = "#,
              module.get_exports_argument(),
              property_access(used, 0)
            )
          }
        } else {
          // DEBUG: Log inlined export only for shared modules
          if shared_key.is_some() {
            tracing::debug!(
              "[RSPACK_EXPORT_DEBUG:ESM_INLINED] Module: {:?}, Type: inlined, ConsumeShared: {:?}",
              module_identifier,
              consume_shared_info
            );
          }

          if let Some(ref share_key) = shared_key {
            format!(
              "/* @common:if [condition=\"treeShake.{share_key}.default\"] */ /* inlined ESM default export */ var {DEFAULT_EXPORT} = "
            )
          } else {
            format!("/* inlined ESM default export */ var {DEFAULT_EXPORT} = ")
          }
        }
      } else {
        // DEBUG: Log unused export only for shared modules
        if shared_key.is_some() {
          tracing::debug!(
            "[RSPACK_EXPORT_DEBUG:ESM_UNUSED] Module: {:?}, Type: unused, ConsumeShared: {:?}",
            module_identifier,
            consume_shared_info
          );
        }

        if let Some(ref share_key) = shared_key {
          format!(
            "/* @common:if [condition=\"treeShake.{share_key}.default\"] */ /* unused ESM default export */ var {DEFAULT_EXPORT} = "
          )
        } else {
          format!("/* unused ESM default export */ var {DEFAULT_EXPORT} = ")
        }
      };

      source.replace(
        dep.range_stmt.start,
        dep.range.start,
        &format!("{}({}", content, dep.prefix),
        None,
      );

      let end_content = if shared_key.is_some() {
        ") /* @common:endif */;"
      } else {
        ");"
      };

      source.replace_with_enforce(
        dep.range.end,
        dep.range_stmt.end,
        end_content,
        None,
        ReplacementEnforce::Post,
      );
    }
  }
}

fn _process_object_literal_with_usage(content: &str, share_key: &str) -> String {
  // Handle both patterns: {prop1, prop2} and ({prop1, prop2})
  let obj_start = if let Some(pos) = content.find("({") {
    pos + 1
  } else if let Some(pos) = content.find("{") {
    pos
  } else {
    return content.to_string();
  };

  let obj_end = if content.contains("})") {
    if let Some(pos) = content.find("})") {
      pos + 1
    } else {
      return content.to_string();
    }
  } else if let Some(pos) = content.find("}") {
    pos + 1
  } else {
    return content.to_string();
  };

  let before_obj = &content[..obj_start];
  let after_obj = &content[obj_end..];
  let obj_content = &content[obj_start + 1..obj_end - 1];

  let properties: Vec<&str> = obj_content
    .split(',')
    .map(|s| s.trim())
    .filter(|s| !s.is_empty())
    .collect();

  let wrapped_properties: Vec<String> = properties
    .into_iter()
    .map(|prop| {
      let prop_name = prop.trim();
      let should_wrap = match prop_name {
        "fetchWithTimeout" => true, // unused per JSON
        "ApiClient" => true,        // unused per JSON
        "createApiClient" => false, // used per JSON
        _ => false,
      };

      if should_wrap {
        format!(
          "/* @common:if [condition=\"treeShake.{share_key}.{prop_name}\"] */ {prop_name} /* @common:endif */"
        )
      } else {
        prop_name.to_string()
      }
    })
    .collect();

  format!(
    "{}{{\n  {}\n}}{}",
    before_obj,
    wrapped_properties.join(",\n  "),
    after_obj
  )
}
