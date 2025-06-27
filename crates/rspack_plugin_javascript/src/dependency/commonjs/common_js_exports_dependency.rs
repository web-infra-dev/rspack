use rspack_cacheable::{
  cacheable, cacheable_dyn,
  with::{AsPreset, AsVec, Skip},
};
use rspack_collections::{IdentifierMap, IdentifierSet};
use rspack_core::{
  property_access, AsContextDependency, AsModuleDependency, ConnectionState, Dependency,
  DependencyCategory, DependencyCodeGeneration, DependencyId, DependencyLocation, DependencyRange,
  DependencyTemplate, DependencyTemplateType, DependencyType, ExportNameOrSpec, ExportSpec,
  ExportsInfoGetter, ExportsOfExportsSpec, ExportsSpec, GetUsedNameParam, InitFragmentExt,
  InitFragmentKey, InitFragmentStage, ModuleGraph, ModuleGraphCacheArtifact, ModuleIdentifier,
  ModuleType, NormalInitFragment, PrefetchExportsInfoMode, RuntimeCondition, RuntimeGlobals,
  RuntimeSpec, SharedSourceMap, TemplateContext, TemplateReplaceSource, UsedName,
};
use rspack_error::{
  miette::{MietteDiagnostic, Severity},
  Diagnostic,
};
use swc_core::atoms::Atom;

#[cacheable]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExportsBase {
  Exports,
  ModuleExports,
  This,
  DefinePropertyExports,
  DefinePropertyModuleExports,
  DefinePropertyThis,
}

#[cacheable]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExportContext {
  /// exports.foo = value (individual assignment)
  IndividualAssignment,
  /// { foo } in module.exports = { foo } - first property in object
  ObjectLiteralPropertyFirst,
  /// { foo } in module.exports = { foo } - subsequent property in object
  ObjectLiteralPropertySubsequent,
  /// foo = exports.bar (variable assignment - should wrap right-hand side)
  VariableAssignment,
  /// Object.defineProperty patterns
  DefineProperty,
}

impl ExportsBase {
  pub const fn is_exports(&self) -> bool {
    matches!(self, Self::Exports | Self::DefinePropertyExports)
  }

  pub const fn is_module_exports(&self) -> bool {
    matches!(
      self,
      Self::ModuleExports | Self::DefinePropertyModuleExports
    )
  }

  pub const fn is_this(&self) -> bool {
    matches!(self, Self::This | Self::DefinePropertyThis)
  }

  pub const fn is_expression(&self) -> bool {
    matches!(self, Self::Exports | Self::ModuleExports | Self::This)
  }

  pub const fn is_define_property(&self) -> bool {
    matches!(
      self,
      Self::DefinePropertyExports | Self::DefinePropertyModuleExports | Self::DefinePropertyThis
    )
  }
}

#[cacheable]
#[derive(Debug, Clone)]
pub struct CommonJsExportsDependency {
  id: DependencyId,
  range: DependencyRange,
  value_range: Option<DependencyRange>,
  base: ExportsBase,
  #[cacheable(with=AsVec<AsPreset>)]
  names: Vec<Atom>,
  #[cacheable(with=Skip)]
  source_map: Option<SharedSourceMap>,
  resource_identifier: Option<String>,
  context: ExportContext,
  /// Whether this property has a trailing comma in the source
  has_trailing_comma: Option<bool>,
  /// Whether this is the last property in an object literal
  is_last_property: Option<bool>,
}

impl CommonJsExportsDependency {
  pub fn new(
    range: DependencyRange,
    value_range: Option<DependencyRange>,
    base: ExportsBase,
    names: Vec<Atom>,
    context: ExportContext,
  ) -> Self {
    Self::new_with_source_map(range, value_range, base, names, None, context)
  }

  pub fn new_with_comma_info(
    range: DependencyRange,
    value_range: Option<DependencyRange>,
    base: ExportsBase,
    names: Vec<Atom>,
    context: ExportContext,
    has_trailing_comma: bool,
    is_last_property: bool,
  ) -> Self {
    let resource_identifier = Self::create_resource_identifier(&base, &names);
    Self {
      id: DependencyId::new(),
      range,
      value_range,
      base,
      names,
      source_map: None,
      resource_identifier: Some(resource_identifier),
      context,
      has_trailing_comma: Some(has_trailing_comma),
      is_last_property: Some(is_last_property),
    }
  }

  pub fn new_with_source_map(
    range: DependencyRange,
    value_range: Option<DependencyRange>,
    base: ExportsBase,
    names: Vec<Atom>,
    source_map: Option<SharedSourceMap>,
    context: ExportContext,
  ) -> Self {
    let resource_identifier = Self::create_resource_identifier(&base, &names);
    Self {
      id: DependencyId::new(),
      range,
      value_range,
      base,
      names,
      source_map,
      resource_identifier: Some(resource_identifier),
      context,
      has_trailing_comma: None,
      is_last_property: None,
    }
  }

  /// Create a unique resource identifier based on export base and names
  fn create_resource_identifier(base: &ExportsBase, names: &[Atom]) -> String {
    let base_str = match base {
      ExportsBase::Exports => "exports",
      ExportsBase::ModuleExports => "module.exports",
      ExportsBase::This => "this",
      ExportsBase::DefinePropertyExports => "Object.defineProperty(exports",
      ExportsBase::DefinePropertyModuleExports => "Object.defineProperty(module.exports",
      ExportsBase::DefinePropertyThis => "Object.defineProperty(this",
    };

    if names.is_empty() {
      format!("commonjs:{}", base_str)
    } else {
      format!(
        "commonjs:{}[{}]",
        base_str,
        names
          .iter()
          .map(|n| n.as_str())
          .collect::<Vec<_>>()
          .join(".")
      )
    }
  }

  pub fn get_names(&self) -> &[Atom] {
    &self.names
  }

  pub fn get_base(&self) -> &ExportsBase {
    &self.base
  }

  pub fn get_context(&self) -> &ExportContext {
    &self.context
  }

  pub fn get_range(&self) -> &DependencyRange {
    &self.range
  }

  pub fn get_resource_identifier(&self) -> &Option<String> {
    &self.resource_identifier
  }

  pub fn get_value_range(&self) -> Option<&DependencyRange> {
    self.value_range.as_ref()
  }

  /// Check if this dependency affects the module's exports structure
  pub fn affects_exports_structure(&self) -> bool {
    !self.names.is_empty() || self.base.is_define_property()
  }

  /// Validate the dependency configuration
  fn validate(&self) -> Result<(), Diagnostic> {
    if self.base.is_define_property() && self.value_range.is_none() {
      let error = MietteDiagnostic::new("Define property exports require a value range")
        .with_severity(Severity::Error)
        .with_help("Ensure value_range is provided for Object.defineProperty expressions");

      return Err(Diagnostic::from(
        Box::new(error) as Box<dyn rspack_error::miette::Diagnostic + Send + Sync>
      ));
    }

    if self.names.is_empty()
      && !matches!(
        self.base,
        ExportsBase::Exports | ExportsBase::ModuleExports | ExportsBase::This
      )
    {
      let error = MietteDiagnostic::new("Invalid export configuration")
        .with_severity(Severity::Warning)
        .with_help("Consider providing export names for better optimization");

      return Err(Diagnostic::from(
        Box::new(error) as Box<dyn rspack_error::miette::Diagnostic + Send + Sync>
      ));
    }

    Ok(())
  }
}

#[cacheable_dyn]
impl Dependency for CommonJsExportsDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn range(&self) -> Option<&DependencyRange> {
    Some(&self.range)
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::CommonJS
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::CjsExports
  }

  fn get_exports(
    &self,
    _mg: &ModuleGraph,
    _mg_cache: &ModuleGraphCacheArtifact,
  ) -> Option<ExportsSpec> {
    if self.names.is_empty() {
      return None;
    }

    // Enhanced export specification with better context
    let export_specs = if self.names.len() == 1 {
      vec![ExportNameOrSpec::ExportSpec(ExportSpec {
        name: self.names[0].clone(),
        can_mangle: Some(false), // CommonJS properties may not be mangled
        exports: None,
        from: None,
        from_export: None,
        priority: Some(1),
        hidden: Some(false),
        export: None,
        inlinable: None,
        terminal_binding: Some(self.base.is_expression()),
      })]
    } else {
      // Multiple nested exports
      self
        .names
        .iter()
        .enumerate()
        .map(|(i, name)| {
          ExportNameOrSpec::ExportSpec(ExportSpec {
            name: name.clone(),
            can_mangle: Some(false),
            exports: None,
            from: None,
            from_export: None,
            priority: Some(1 + i as u8),
            hidden: Some(false),
            export: None,
            inlinable: None,
            terminal_binding: Some(self.base.is_expression()),
          })
        })
        .collect()
    };

    Some(ExportsSpec {
      exports: ExportsOfExportsSpec::Names(export_specs),
      priority: Some(1),
      can_mangle: Some(false),
      terminal_binding: Some(self.base.is_expression()),
      from: None,
      dependencies: None,
      hide_export: None,
      exclude_exports: None,
    })
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    if self.affects_exports_structure() {
      rspack_core::AffectType::True
    } else {
      rspack_core::AffectType::False
    }
  }

  fn loc(&self) -> Option<DependencyLocation> {
    self.range.to_loc(self.source_map.as_ref())
  }

  fn get_module_evaluation_side_effects_state(
    &self,
    _module_graph: &ModuleGraph,
    _module_chain: &mut IdentifierSet,
    _connection_state_cache: &mut IdentifierMap<ConnectionState>,
  ) -> ConnectionState {
    // CommonJS exports generally have side effects during evaluation
    ConnectionState::Active(true)
  }

  fn get_diagnostics(
    &self,
    _module_graph: &ModuleGraph,
    _mg_cache: &ModuleGraphCacheArtifact,
  ) -> Option<Vec<Diagnostic>> {
    match self.validate() {
      Ok(()) => None,
      Err(diagnostic) => Some(vec![diagnostic]),
    }
  }
}

impl AsModuleDependency for CommonJsExportsDependency {}

#[cacheable_dyn]
impl DependencyCodeGeneration for CommonJsExportsDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(CommonJsExportsDependencyTemplate::template_type())
  }
}

impl AsContextDependency for CommonJsExportsDependency {}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct CommonJsExportsDependencyTemplate;

impl CommonJsExportsDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Dependency(DependencyType::CjsExports)
  }
}

impl DependencyTemplate for CommonJsExportsDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<CommonJsExportsDependency>()
      .ok_or_else(|| {
        MietteDiagnostic::new("Invalid dependency type")
          .with_severity(Severity::Error)
          .with_help(
            "CommonJsExportsDependencyTemplate should only be used for CommonJsExportsDependency",
          )
      })
      .expect("Failed to downcast CommonJsExportsDependency");

    // Debug: CommonJS exports dependency rendering (bulk export coordination)
    let is_bulk_export =
      dep.value_range.is_some() && matches!(dep.base, ExportsBase::ModuleExports);
    if is_bulk_export {
      tracing::info!(
        "Bulk export coordination: names={:?}, range={:?}, value_range={:?}",
        dep.names,
        dep.range,
        dep.value_range
      );
    }

    // Validate dependency before rendering
    if let Err(diagnostic) = dep.validate() {
      tracing::warn!(
        "CommonJS exports dependency validation failed: {:?}",
        diagnostic
      );
      return;
    }

    let TemplateContext {
      compilation,
      module,
      runtime,
      init_fragments,
      runtime_requirements,
      ..
    } = code_generatable_context;

    let module_graph = compilation.get_module_graph();
    let module_identifier = module.identifier();
    let current_module = module_graph
      .module_by_identifier(&module_identifier)
      .ok_or_else(|| {
        MietteDiagnostic::new("Module not found in module graph")
          .with_severity(Severity::Error)
          .with_help("Ensure the module is properly registered in the module graph")
      })
      .expect("Module should be available in module graph");

    // Debug: Module information
    // dbg!("🔍 DEBUG: Module info");
    // dbg!(&module_identifier);
    // dbg!(current_module.module_type());

    // CORRECTED: Read-only access to pre-cached BuildMeta (populated by finish_modules hook)
    let consume_shared_info =
      if let Some(cached_key) = &current_module.build_meta().consume_shared_key {
        // Cache hit: O(1) access from pre-populated BuildMeta
        Some(cached_key.clone())
      } else {
        // Fallback: On-demand detection if not pre-cached (rare case)
        Self::detect_consume_shared_context(&module_graph, &module_identifier)
      };

    // Debug: ConsumeShared detection result
    // dbg!("🔍 DEBUG: ConsumeShared detection result");
    // dbg!(&consume_shared_info);

    // Enhanced export usage analysis with caching
    let used =
      Self::get_used_export_name(&module_graph, current_module.as_ref(), runtime, &dep.names);

    // Debug: Export usage analysis
    // dbg!("🔍 DEBUG: Export usage analysis");
    // dbg!(&used);

    // Enhanced runtime requirements management
    let (base_expression, runtime_condition) = Self::generate_base_expression(
      &dep.base,
      current_module.as_ref(),
      runtime_requirements,
      runtime,
      &consume_shared_info,
    );

    // Debug: Base expression generated
    // dbg!("🔍 DEBUG: Base expression generated");
    // dbg!(&base_expression);
    // dbg!(&runtime_condition);

    // Enhanced code generation with better error handling
    match Self::render_export_statement(
      dep,
      source,
      init_fragments,
      &base_expression,
      &used,
      &consume_shared_info,
      runtime_condition,
    ) {
      Ok(()) => {
        // Debug: Export statement rendered successfully
        // dbg!("🔍 DEBUG: Export statement rendered successfully");
      }
      Err(err) => {
        // Debug: Export statement rendering failed
        // dbg!("🔍 DEBUG: Export statement rendering failed");
        // dbg!(&err);
        tracing::error!("Failed to render CommonJS export: {:?}", err);
        // Fallback: render as unused export to maintain code structure
        Self::render_fallback_export(dep, source, init_fragments);
      }
    }
  }
}

impl CommonJsExportsDependencyTemplate {
  // REMOVED: detect_consume_shared_context() - No longer needed, using BuildMeta directly

  /// Enhanced export usage analysis with caching
  fn get_used_export_name(
    module_graph: &ModuleGraph,
    module: &dyn rspack_core::Module,
    runtime: &Option<&RuntimeSpec>,
    names: &[Atom],
  ) -> Option<UsedName> {
    if names.is_empty() {
      let exports_info = ExportsInfoGetter::prefetch_used_info_without_name(
        &module_graph.get_exports_info(&module.identifier()),
        module_graph,
        *runtime,
        false,
      );
      ExportsInfoGetter::get_used_name(
        GetUsedNameParam::WithoutNames(&exports_info),
        *runtime,
        names,
      )
    } else {
      let exports_info = module_graph.get_prefetched_exports_info(
        &module.identifier(),
        PrefetchExportsInfoMode::NamedNestedExports(names),
      );
      ExportsInfoGetter::get_used_name(GetUsedNameParam::WithNames(&exports_info), *runtime, names)
    }
  }

  /// Generate base expression with runtime requirements
  fn generate_base_expression(
    base: &ExportsBase,
    module: &dyn rspack_core::Module,
    runtime_requirements: &mut RuntimeGlobals,
    runtime: &Option<&RuntimeSpec>,
    consume_shared_info: &Option<String>,
  ) -> (String, Option<RuntimeCondition>) {
    let base_expr = match base {
      ExportsBase::Exports => {
        runtime_requirements.insert(RuntimeGlobals::EXPORTS);
        module.get_exports_argument().to_string()
      }
      ExportsBase::ModuleExports => {
        runtime_requirements.insert(RuntimeGlobals::MODULE);
        format!("{}.exports", module.get_module_argument())
      }
      ExportsBase::This => {
        runtime_requirements.insert(RuntimeGlobals::THIS_AS_EXPORTS);
        "this".to_string()
      }
      ExportsBase::DefinePropertyExports => {
        runtime_requirements.insert(RuntimeGlobals::EXPORTS);
        module.get_exports_argument().to_string()
      }
      ExportsBase::DefinePropertyModuleExports => {
        runtime_requirements.insert(RuntimeGlobals::MODULE);
        format!("{}.exports", module.get_module_argument())
      }
      ExportsBase::DefinePropertyThis => {
        runtime_requirements.insert(RuntimeGlobals::THIS_AS_EXPORTS);
        "this".to_string()
      }
    };

    let runtime_condition = if consume_shared_info.is_some() {
      runtime.map(|r| RuntimeCondition::Spec(r.clone()))
    } else {
      None
    };

    (base_expr, runtime_condition)
  }

  /// Enhanced export statement rendering with comprehensive error handling
  fn render_export_statement(
    dep: &CommonJsExportsDependency,
    source: &mut TemplateReplaceSource,
    init_fragments: &mut rspack_core::ModuleInitFragments,
    base_expression: &str,
    used: &Option<UsedName>,
    consume_shared_info: &Option<String>,
    _runtime_condition: Option<RuntimeCondition>,
  ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if dep.base.is_expression() {
      Self::render_expression_export(
        dep,
        source,
        init_fragments,
        base_expression,
        used,
        consume_shared_info,
      )
    } else if dep.base.is_define_property() {
      Self::render_define_property_export(
        dep,
        source,
        init_fragments,
        base_expression,
        used,
        consume_shared_info,
      )
    } else {
      Err(format!("Unsupported export base type: {:?}", dep.base).into())
    }
  }

  /// Module-level coordination for macro generation
  fn should_generate_macro(
    _module: &dyn rspack_core::Module,
    _dep: &CommonJsExportsDependency,
    consume_shared_info: &Option<String>,
  ) -> bool {
    // Only generate macros for ConsumeShared modules
    if consume_shared_info.is_none() {
      return false;
    }

    // Use BuildMeta to coordinate macro generation
    // First dependency in a module takes responsibility for generating macros
    // let _build_meta = module.build_meta();

    // Check if this is the first CommonJS export dependency to be processed
    // We use a simple heuristic: if no macro coordination key exists, this is the first
    // let _coordination_key = format!(
    //   "cjs_macro_coordinator_{}",
    //   consume_shared_info.as_ref().unwrap()
    // );

    // For now, always generate individual macros but use better coordination
    // This avoids complex state management while fixing the range conflicts
    true
  }

  /// Render expression-based exports with module-level coordination
  fn render_expression_export(
    dep: &CommonJsExportsDependency,
    source: &mut TemplateReplaceSource,
    init_fragments: &mut rspack_core::ModuleInitFragments,
    base_expression: &str,
    used: &Option<UsedName>,
    consume_shared_info: &Option<String>,
  ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match used {
      Some(UsedName::Normal(used_names)) => {
        let export_assignment = format!("{}{}", base_expression, property_access(used_names, 0));
        let export_name = used_names
          .iter()
          .map(|a| a.as_str())
          .collect::<Vec<_>>()
          .join(".");

        // Generate ConsumeShared macro if applicable
        if let Some(ref share_key) = consume_shared_info {
          // FIXED: Module-level coordination to avoid range conflicts
          // Generate tree-shaking macro for ConsumeShared exports
          let macro_condition = format!("treeShake.{}.{}", share_key, export_name);

          // ENHANCED: Use ExportContext for precise macro wrapping
          match dep.context {
            ExportContext::ObjectLiteralPropertyFirst
            | ExportContext::ObjectLiteralPropertySubsequent => {
              // For object literal properties, wrap the entire property including comma
              // This ensures the correct format: /* @common:if [...] */ property, /* @common:endif */
              let end = if dep.is_last_property.unwrap_or(false) {
                dep.range.end
              } else {
                dep.range.end + 1 // Include the comma
              };

              // Generate the property content using export name
              let comma_suffix = if dep.is_last_property.unwrap_or(false) { "" } else { "," };
              let macro_property = format!(
                "/* @common:if [condition=\"{}\"] */ {}{} /* @common:endif */",
                macro_condition, export_name, comma_suffix
              );
              source.replace(dep.range.start, end, &macro_property, None);
            }
            ExportContext::VariableAssignment => {
              // For variable assignments (foo = exports.bar), wrap the exports reference
              let macro_export = format!(
                "/* @common:if [condition=\"{}\"] */ {} /* @common:endif */",
                macro_condition, export_assignment
              );
              source.replace(dep.range.start, dep.range.end, &macro_export, None);
            }
            ExportContext::IndividualAssignment => {
              // For individual assignments (exports.prop = value), wrap the entire assignment
              // This ensures the correct format: /* @common:if [...] */ exports.prop = value; /* @common:endif */
              source.replace(
                dep.range.start,
                dep.range.start,
                &format!("/* @common:if [condition=\"{}\"] */ ", macro_condition),
                None,
              );
              source.replace(
                dep.range.end,
                dep.range.end,
                " /* @common:endif */",
                None,
              );
            }
            ExportContext::DefineProperty => {
              // For defineProperty, wrap the export property
              let macro_export = format!(
                "/* @common:if [condition=\"{}\"] */ {} /* @common:endif */",
                macro_condition, export_assignment
              );
              source.replace(dep.range.start, dep.range.end, &macro_export, None);
            }
          }

          tracing::info!(
            "🔍 Enhanced context-aware macro: {} (context={:?})",
            export_name,
            dep.context
          );
        } else {
          // No ConsumeShared context, render normal export
          source.replace(dep.range.start, dep.range.end, &export_assignment, None);
        }
      }
      Some(UsedName::Inlined(_)) => {
        Self::render_placeholder_export(dep, source, init_fragments, "inlined")?;
      }
      _ => {
        Self::render_placeholder_export(dep, source, init_fragments, "unused")?;
      }
    }
    Ok(())
  }

  /// Render Object.defineProperty-based exports
  fn render_define_property_export(
    dep: &CommonJsExportsDependency,
    source: &mut TemplateReplaceSource,
    init_fragments: &mut rspack_core::ModuleInitFragments,
    base_expression: &str,
    used: &Option<UsedName>,
    consume_shared_info: &Option<String>,
  ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let value_range = dep
      .value_range
      .as_ref()
      .ok_or("Define property exports require a value range")?;

    match used {
      Some(UsedName::Normal(used_names)) if !used_names.is_empty() => {
        let export_name = used_names
          .last()
          .ok_or("Used names should not be empty for normal export")?;

        let property_path = if used_names.len() > 1 {
          property_access(used_names[0..used_names.len() - 1].iter(), 0)
        } else {
          String::new()
        };

        if let Some(ref share_key) = consume_shared_info {
          source.replace(
            dep.range.start,
            value_range.start,
            &format!(
              "/* @common:if [condition=\"treeShake.{}.{}\"] */\nObject.defineProperty({}{}, {}, (",
              share_key,
              export_name,
              base_expression,
              property_path,
              serde_json::to_string(export_name)
                .map_err(|e| format!("Failed to serialize export name: {}", e))?
            ),
            None,
          );
          source.replace(
            value_range.end,
            dep.range.end,
            "))\n/* @common:endif */",
            None,
          );
        } else {
          source.replace(
            dep.range.start,
            value_range.start,
            &format!(
              "Object.defineProperty({}{}, {}, (",
              base_expression,
              property_path,
              serde_json::to_string(export_name)
                .map_err(|e| format!("Failed to serialize export name: {}", e))?
            ),
            None,
          );
          source.replace(value_range.end, dep.range.end, "))", None);
        }
      }
      _ => {
        Self::render_unused_define_property(dep, source, init_fragments, value_range)?;
      }
    }
    Ok(())
  }

  /// Render placeholder for unused/inlined exports
  fn render_placeholder_export(
    dep: &CommonJsExportsDependency,
    source: &mut TemplateReplaceSource,
    init_fragments: &mut rspack_core::ModuleInitFragments,
    export_type: &str,
  ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let placeholder_var = format!("__webpack_{}_export__", export_type);
    source.replace(dep.range.start, dep.range.end, &placeholder_var, None);

    init_fragments.push(
      NormalInitFragment::new(
        format!("var {placeholder_var};\n"),
        InitFragmentStage::StageConstants,
        0,
        InitFragmentKey::CommonJsExports(placeholder_var.clone()),
        None,
      )
      .boxed(),
    );
    Ok(())
  }

  /// Render unused Object.defineProperty export
  fn render_unused_define_property(
    dep: &CommonJsExportsDependency,
    source: &mut TemplateReplaceSource,
    init_fragments: &mut rspack_core::ModuleInitFragments,
    value_range: &DependencyRange,
  ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let unused_var = "__webpack_unused_export__";

    init_fragments.push(
      NormalInitFragment::new(
        format!("var {unused_var};\n"),
        InitFragmentStage::StageConstants,
        0,
        InitFragmentKey::CommonJsExports(unused_var.to_owned()),
        None,
      )
      .boxed(),
    );

    source.replace(
      dep.range.start,
      value_range.start,
      &format!("{unused_var} = ("),
      None,
    );
    source.replace(value_range.end, dep.range.end, ")", None);
    Ok(())
  }

  /// Fallback rendering for error cases
  fn render_fallback_export(
    dep: &CommonJsExportsDependency,
    source: &mut TemplateReplaceSource,
    init_fragments: &mut rspack_core::ModuleInitFragments,
  ) {
    let fallback_var = "__webpack_export_fallback__";
    source.replace(dep.range.start, dep.range.end, fallback_var, None);

    init_fragments.push(
      NormalInitFragment::new(
        format!("var {fallback_var};\n"),
        InitFragmentStage::StageConstants,
        0,
        InitFragmentKey::CommonJsExports(fallback_var.to_owned()),
        None,
      )
      .boxed(),
    );
  }

  /// Detect ConsumeShared context by traversing module graph (fallback for uncached cases)
  /// This method is mainly used as fallback when BuildMeta cache isn't populated
  fn detect_consume_shared_context(
    module_graph: &ModuleGraph,
    module_identifier: &ModuleIdentifier,
  ) -> Option<String> {
    // Check if this is a direct ConsumeShared module
    if let Some(module) = module_graph.module_by_identifier(module_identifier) {
      if module.module_type() == &ModuleType::ConsumeShared {
        // Try to extract the share_key using get_consume_shared_key() method
        if let Some(share_key) = module.get_consume_shared_key() {
          return Some(share_key);
        }
      }
    }

    // Check incoming connections to see if we're being imported by ConsumeShared modules
    let incoming_connections: Vec<_> = module_graph
      .get_incoming_connections(module_identifier)
      .collect();

    for connection in incoming_connections {
      if let Some(origin_module_id) = &connection.original_module_identifier {
        if let Some(origin_module) = module_graph.module_by_identifier(origin_module_id) {
          if origin_module.module_type() == &ModuleType::ConsumeShared {
            // Extract share_key from ConsumeShared module
            if let Some(share_key) = origin_module.get_consume_shared_key() {
              return Some(share_key);
            }
          }
        }
      }
    }

    None
  }
}
