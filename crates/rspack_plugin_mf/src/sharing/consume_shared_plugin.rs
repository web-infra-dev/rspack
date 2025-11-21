use std::{
  collections::HashSet,
  fmt,
  sync::{Arc, LazyLock, OnceLock},
};

use camino::Utf8Path;
use regex::Regex;
use rspack_cacheable::cacheable;
use rspack_core::{
  BoxModule, ChunkUkey, Compilation, CompilationAdditionalTreeRuntimeRequirements,
  CompilationFinishModules, CompilationParams, CompilerThisCompilation, Context,
  DependencyCategory, DependencyType, ExportsInfoGetter, ModuleExt, ModuleFactoryCreateData,
  ModuleGraph, ModuleIdentifier, ModuleType, NormalModuleCreateData,
  NormalModuleFactoryCreateModule, NormalModuleFactoryFactorize, NormalModuleFactoryModule, Plugin,
  PrefetchExportsInfoMode, ProvidedExports, ResolveOptionsWithDependencyType, ResolveResult,
  Resolver, RuntimeGlobals,
};
use rspack_error::{Diagnostic, Result, error};
use rspack_fs::ReadableFileSystem;
use rspack_hook::{plugin, plugin_hook};
use rustc_hash::FxHashMap;

use super::{
  consume_shared_module::ConsumeSharedModule,
  consume_shared_runtime_module::ConsumeSharedRuntimeModule,
};

#[cacheable]
#[derive(Debug, Clone, Hash)]
pub struct ConsumeOptions {
  pub import: Option<String>,
  pub import_resolved: Option<String>,
  pub share_key: String,
  pub share_scope: String,
  pub required_version: Option<ConsumeVersion>,
  pub package_name: Option<String>,
  pub strict_version: bool,
  pub singleton: bool,
  pub eager: bool,
}

#[cacheable]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ConsumeVersion {
  Version(String),
  False,
}

impl fmt::Display for ConsumeVersion {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      ConsumeVersion::Version(v) => write!(f, "{v}"),
      ConsumeVersion::False => write!(f, "*"),
    }
  }
}

pub static RELATIVE_REQUEST: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"^\.\.?(\/|$)").expect("Invalid regex"));
pub static ABSOLUTE_REQUEST: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"^(\/|[A-Za-z]:\\|\\\\)").expect("Invalid regex"));
pub static PACKAGE_NAME: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"^((?:@[^\\/]+[\\/])?[^\\/]+)").expect("Invalid regex"));

#[derive(Debug)]
pub struct MatchedConsumes {
  pub resolved: FxHashMap<String, Arc<ConsumeOptions>>,
  pub unresolved: FxHashMap<String, Arc<ConsumeOptions>>,
  pub prefixed: FxHashMap<String, Arc<ConsumeOptions>>,
}

pub async fn resolve_matched_configs(
  compilation: &mut Compilation,
  resolver: Arc<Resolver>,
  configs: &[(String, Arc<ConsumeOptions>)],
) -> MatchedConsumes {
  let mut resolved = FxHashMap::default();
  let mut unresolved = FxHashMap::default();
  let mut prefixed = FxHashMap::default();
  for (request, config) in configs {
    if RELATIVE_REQUEST.is_match(request) {
      let Ok(ResolveResult::Resource(resource)) = resolver
        .resolve(compilation.options.context.as_ref(), request)
        .await
      else {
        compilation.push_diagnostic(error!("Can't resolve shared module {request}").into());
        continue;
      };
      resolved.insert(resource.path.as_str().to_string(), config.clone());
      compilation
        .file_dependencies
        .insert(resource.path.as_path().into());
    } else if ABSOLUTE_REQUEST.is_match(request) {
      resolved.insert(request.to_owned(), config.clone());
    } else if request.ends_with('/') {
      prefixed.insert(request.to_owned(), config.clone());
    } else {
      unresolved.insert(request.to_owned(), config.clone());
    }
  }
  MatchedConsumes {
    resolved,
    unresolved,
    prefixed,
  }
}

pub async fn get_description_file(
  fs: Arc<dyn ReadableFileSystem>,
  mut dir: &Utf8Path,
  satisfies_description_file_data: Option<impl Fn(Option<serde_json::Value>) -> bool>,
) -> (Option<serde_json::Value>, Option<Vec<String>>) {
  let description_filename = "package.json";
  let mut checked_file_paths = HashSet::new();

  loop {
    let description_file = dir.join(description_filename);

    let data = fs.read(&description_file).await;

    if let Ok(data) = data
      && let Ok(data) = serde_json::from_slice::<serde_json::Value>(&data)
    {
      if satisfies_description_file_data
        .as_ref()
        .is_some_and(|f| !f(Some(data.clone())))
      {
        checked_file_paths.insert(description_file.to_string());
      } else {
        return (Some(data), None);
      }
    }
    if let Some(parent) = dir.parent() {
      dir = parent;
    } else {
      return (None, Some(checked_file_paths.into_iter().collect()));
    }
  }
}

pub fn get_required_version_from_description_file(
  data: serde_json::Value,
  package_name: &str,
) -> Option<ConsumeVersion> {
  let data = data.as_object()?;
  let get_version_from_dependencies = |dependencies: &str| {
    data
      .get(dependencies)
      .and_then(|d| d.as_object())
      .and_then(|deps| deps.get(package_name))
      .and_then(|v| v.as_str())
      .map(|v| ConsumeVersion::Version(v.to_string()))
  };
  get_version_from_dependencies("optionalDependencies")
    .or_else(|| get_version_from_dependencies("dependencies"))
    .or_else(|| get_version_from_dependencies("peerDependencies"))
    .or_else(|| get_version_from_dependencies("devDependencies"))
}

#[derive(Debug)]
pub struct ConsumeSharedPluginOptions {
  pub consumes: Vec<(String, Arc<ConsumeOptions>)>,
  pub enhanced: bool,
}

#[plugin]
#[derive(Debug)]
pub struct ConsumeSharedPlugin {
  options: ConsumeSharedPluginOptions,
  resolver: OnceLock<Arc<Resolver>>,
  compiler_context: OnceLock<Context>,
  matched_consumes: OnceLock<Arc<MatchedConsumes>>,
}

impl ConsumeSharedPlugin {
  pub fn new(options: ConsumeSharedPluginOptions) -> Self {
    Self::new_inner(
      options,
      Default::default(),
      Default::default(),
      Default::default(),
    )
  }

  fn init_context(&self, compilation: &Compilation) {
    self
      .compiler_context
      .set(compilation.options.context.clone())
      .expect("failed to set compiler context");
  }

  fn get_context(&self) -> Context {
    self
      .compiler_context
      .get()
      .expect("init_context first")
      .clone()
  }

  fn init_resolver(&self, compilation: &Compilation) {
    self
      .resolver
      .set(
        compilation
          .resolver_factory
          .get(ResolveOptionsWithDependencyType {
            resolve_options: None,
            resolve_to_context: false,
            dependency_category: DependencyCategory::Esm,
          }),
      )
      .expect("failed to set resolver for multiple times");
  }

  fn get_resolver(&self) -> Arc<Resolver> {
    self.resolver.get().expect("init_resolver first").clone()
  }

  async fn init_matched_consumes(&self, compilation: &mut Compilation, resolver: Arc<Resolver>) {
    let config = resolve_matched_configs(compilation, resolver, &self.options.consumes).await;
    self
      .matched_consumes
      .set(Arc::new(config))
      .expect("failed to set matched consumes");
  }

  fn get_matched_consumes(&self) -> Arc<MatchedConsumes> {
    self
      .matched_consumes
      .get()
      .expect("init_matched_consumes first")
      .clone()
  }

  async fn get_required_version(
    &self,
    context: &Context,
    request: &str,
    config: Arc<ConsumeOptions>,
    add_diagnostic: &mut impl FnMut(Diagnostic),
  ) -> Option<ConsumeVersion> {
    let mut required_version_warning = |details: &str| {
      add_diagnostic(Diagnostic::warn(
        self.name().into(),
        format!(
          "No required version specified and unable to automatically determine one. {details} file: shared module {request}"
        ),
      ))
    };
    if let Some(version) = config.required_version.as_ref() {
      Some(version.clone())
    } else {
      let package_name = if let Some(name) = &config.package_name {
        Some(name.as_str())
      } else if ABSOLUTE_REQUEST.is_match(request) {
        return None;
      } else if let Some(caps) = PACKAGE_NAME.captures(request)
        && let Some(mat) = caps.get(0)
      {
        Some(mat.as_str())
      } else {
        required_version_warning("Unable to extract the package name from request.");
        return None;
      };

      if let Some(package_name) = package_name {
        let fs = self.get_resolver().inner_fs();
        let (data, checked_description_file_paths) = get_description_file(
          fs,
          context.as_path(),
          Some(|data: Option<serde_json::Value>| {
            if let Some(data) = data {
              let name_matches = data.get("name").and_then(|n| n.as_str()) == Some(package_name);
              let version_matches = get_required_version_from_description_file(data, package_name)
                .is_some_and(|version| matches!(version, ConsumeVersion::Version(_)));
              name_matches || version_matches
            } else {
              false
            }
          }),
        )
        .await;

        if let Some(data) = data {
          if let Some(name) = data.get("name").and_then(|n| n.as_str())
            && name == package_name
          {
            // Package self-referencing
            return None;
          }
          return get_required_version_from_description_file(data, package_name);
        } else {
          if let Some(file_paths) = checked_description_file_paths
            && !file_paths.is_empty()
          {
            required_version_warning(&format!(
              "Unable to find required version for \"{package_name}\" in description file/s\n{}\nIt need to be in dependencies, devDependencies or peerDependencies.",
              file_paths.join("\n")
            ));
          } else {
            required_version_warning(&format!(
              "Unable to find description file in {}",
              context.as_str()
            ));
          }
          return None;
        }
      }

      None
    }
  }

  /// Extract share key from ConsumeShared module identifier
  /// Format: "consume shared module ({share_scope}) {share_key}@{version}..."
  fn extract_share_key_from_identifier(identifier: &str) -> Option<String> {
    // Use regex to extract share_key from the identifier
    static SHARE_KEY_REGEX: LazyLock<Regex> = LazyLock::new(|| {
      Regex::new(r"consume shared module \([^)]+\) ([^@]+)@").expect("valid regex")
    });

    SHARE_KEY_REGEX
      .captures(identifier)
      .and_then(|caps| caps.get(1))
      .map(|m| m.as_str().to_string())
  }

  /// Set consume_shared_key in the fallback module's BuildMeta for tree-shaking macro support
  fn set_consume_shared_key_in_fallback(
    compilation: &mut Compilation,
    consume_shared_id: &ModuleIdentifier,
  ) -> Result<()> {
    // First, get the share_key from the ConsumeShared module
    let share_key = {
      let module_graph = compilation.get_module_graph();
      if let Some(consume_shared_module) = module_graph.module_by_identifier(consume_shared_id) {
        consume_shared_module.get_consume_shared_key()
      } else {
        None
      }
    };

    if let Some(share_key) = share_key {
      // Find the fallback module identifier
      let fallback_id = {
        let module_graph = compilation.get_module_graph();
        if let Some(consume_shared_module) = module_graph.module_by_identifier(consume_shared_id) {
          if let Some(consume_shared) = consume_shared_module
            .as_any()
            .downcast_ref::<ConsumeSharedModule>()
          {
            consume_shared.find_fallback_module_id(&module_graph)
          } else {
            None
          }
        } else {
          None
        }
      };

      // If we have a fallback, set the consume_shared_key in its BuildMeta
      if let Some(fallback_id) = fallback_id {
        let mut module_graph = compilation.get_module_graph_mut();
        if let Some(fallback_module) = module_graph.module_by_identifier_mut(&fallback_id) {
          // Set the consume_shared_key in the fallback module's BuildMeta
          fallback_module.build_meta_mut().consume_shared_key = Some(share_key);
        }
      }
    }

    Ok(())
  }

  /// Copy metadata from fallback module to ConsumeShared module
  fn copy_fallback_metadata_to_consume_shared(
    compilation: &mut Compilation,
    consume_shared_id: &ModuleIdentifier,
  ) -> Result<()> {
    // First, find the fallback module identifier
    let fallback_id = {
      let module_graph = compilation.get_module_graph();
      if let Some(consume_shared_module) = module_graph.module_by_identifier(consume_shared_id) {
        if let Some(consume_shared) = consume_shared_module
          .as_any()
          .downcast_ref::<ConsumeSharedModule>()
        {
          consume_shared.find_fallback_module_id(&module_graph)
        } else {
          None
        }
      } else {
        None
      }
    };

    // If we have a fallback, copy the export metadata
    if let Some(fallback_id) = fallback_id {
      let mut module_graph = compilation.get_module_graph_mut();

      // Copy export information from fallback to ConsumeShared
      Self::copy_exports_from_fallback_to_consume_shared(
        &mut module_graph,
        &fallback_id,
        consume_shared_id,
      )?;
    }

    Ok(())
  }

  /// Copy export information from fallback module to ConsumeShared module
  fn copy_exports_from_fallback_to_consume_shared(
    module_graph: &mut ModuleGraph,
    fallback_id: &ModuleIdentifier,
    consume_shared_id: &ModuleIdentifier,
  ) -> Result<()> {
    use rspack_core::ExportProvided;

    // Get exports info for both modules
    let fallback_exports_info = module_graph.get_exports_info(fallback_id);
    let consume_shared_exports_info = module_graph.get_exports_info(consume_shared_id);

    // Get the fallback module's provided exports using prefetched mode
    let prefetched_fallback = ExportsInfoGetter::prefetch(
      &fallback_exports_info,
      module_graph,
      PrefetchExportsInfoMode::Full,
    );

    let fallback_provided = prefetched_fallback.get_provided_exports();

    // Copy the provided exports to the ConsumeShared module
    match fallback_provided {
      ProvidedExports::ProvidedNames(export_names) => {
        // Copy each specific export from fallback to ConsumeShared
        for export_name in export_names {
          // Get or create export info for this export in the ConsumeShared module
          let consume_shared_export_info =
            consume_shared_exports_info.get_export_info(module_graph, &export_name);
          let fallback_export_info =
            fallback_exports_info.get_export_info(module_graph, &export_name);

          // Copy the provided status
          if let Some(provided) = fallback_export_info.as_data(module_graph).provided() {
            consume_shared_export_info
              .as_data_mut(module_graph)
              .set_provided(Some(provided));
          } else {
            // Default to provided if not explicitly set in fallback
            consume_shared_export_info
              .as_data_mut(module_graph)
              .set_provided(Some(ExportProvided::Provided));
          }

          // Copy can_mangle_provide status
          if let Some(can_mangle) = fallback_export_info
            .as_data(module_graph)
            .can_mangle_provide()
          {
            consume_shared_export_info
              .as_data_mut(module_graph)
              .set_can_mangle_provide(Some(can_mangle));
          }

          // Copy exports_info if it exists (for nested exports)
          if let Some(nested_exports_info) =
            fallback_export_info.as_data(module_graph).exports_info()
          {
            consume_shared_export_info
              .as_data_mut(module_graph)
              .set_exports_info(Some(nested_exports_info));
          }

          // Note: Usage state copying is handled by FlagDependencyUsagePlugin
          // We only copy provision metadata here

          // Copy terminal binding information if available
          let terminal_binding = fallback_export_info
            .as_data(module_graph)
            .terminal_binding();
          if terminal_binding {
            consume_shared_export_info
              .as_data_mut(module_graph)
              .set_terminal_binding(terminal_binding);
          }
        }

        // Mark the ConsumeShared module as having complete provide info
        consume_shared_exports_info.set_has_provide_info(module_graph);

        // Set the "other exports" to not provided (since we copied all specific exports)
        consume_shared_exports_info.set_unknown_exports_provided(
          module_graph,
          false, // not provided
          None,  // no exclude exports
          None,  // no can_mangle
          None,  // no terminal_binding
          None,  // no target_key
        );
      }
      ProvidedExports::ProvidedAll => {
        // If fallback provides all exports, mark ConsumeShared the same way
        consume_shared_exports_info.set_unknown_exports_provided(
          module_graph,
          true, // provided
          None, // no exclude exports
          None, // no can_mangle
          None, // no terminal_binding
          None, // no target_key
        );
        consume_shared_exports_info.set_has_provide_info(module_graph);
      }
      ProvidedExports::Unknown => {
        // Keep unknown status - don't copy anything
      }
    }

    Ok(())
  }

  /// Enhanced metadata copying that also analyzes usage through incoming connections
  fn enhanced_copy_fallback_metadata_to_consume_shared(
    compilation: &mut Compilation,
    consume_shared_id: &ModuleIdentifier,
  ) -> Result<()> {
    // Note: Enhanced analysis disabled due to borrow checker issues
    // ShareUsagePlugin provides this functionality instead

    // First, do the standard export metadata copying
    Self::copy_fallback_metadata_to_consume_shared(compilation, consume_shared_id)?;

    /* Enhanced analysis commented out due to borrow checker issues
    // Then, enhance with usage analysis from incoming connections
    let mut module_graph = compilation.get_module_graph_mut();

    // Analyze incoming connections to track actual usage
    let incoming_connections: Vec<_> = module_graph
      .get_incoming_connections(consume_shared_id)
      .collect();

    for connection in incoming_connections {
      if let Some(dependency) = module_graph.dependency_by_id(&connection.dependency_id) {
        // Use get_referenced_exports to extract specific export names
        let referenced_exports = dependency.get_referenced_exports(
          &module_graph,
          &ModuleGraphCacheArtifact::default(),
          None,
        );

        // Process referenced exports and mark them as used in the ConsumeShared module
        for export_ref in referenced_exports {
          match export_ref {
            ExtendedReferencedExport::Array(names) => {
              for name in names {
                let export_atom = rspack_util::atom::Atom::from(name.as_str());
                let exports_info = module_graph.get_exports_info(consume_shared_id);
                let export_info = exports_info.get_export_info(&mut module_graph, &export_atom);

                // Usage state is handled by FlagDependencyUsagePlugin
                // Just mark as provided

                export_info.as_data_mut(&mut module_graph).set_provided(
                  Some(rspack_core::ExportProvided::Provided),
                );
              }
            },
            ExtendedReferencedExport::Export(export_info) => {
              if !export_info.name.is_empty() {
                for name in export_info.name {
                  let export_atom = rspack_util::atom::Atom::from(name.as_str());
                  let exports_info = module_graph.get_exports_info(consume_shared_id);
                  let export_info = exports_info.get_export_info(&mut module_graph, &export_atom);

                  // Usage state is handled by FlagDependencyUsagePlugin
                  // Just mark as provided

                  export_info.as_data_mut(&mut module_graph).set_provided(
                    Some(rspack_core::ExportProvided::Provided),
                  );
                }
              }
            },
            ExtendedReferencedExport::Export(_) => {
              // This might be a namespace import or similar - analyze further if needed
              let exports_info = module_graph.get_exports_info(consume_shared_id);

              // For namespace imports, we may need to mark all exports as potentially used
              // This is a conservative approach to ensure tree-shaking doesn't remove needed exports
              let prefetched = ExportsInfoGetter::prefetch(
                &exports_info,
                &module_graph,
                PrefetchExportsInfoMode::Full,
              );

              if let ProvidedExports::ProvidedNames(export_names) = prefetched.get_provided_exports() {
                for export_name in export_names {
                  let export_info = exports_info.get_export_info(&mut module_graph, &export_name);
                  // Usage state is handled by FlagDependencyUsagePlugin
                  // Just mark as provided
                  export_info.as_data_mut(&mut module_graph).set_provided(
                    Some(rspack_core::ExportProvided::Provided),
                  );
                }
              }
            },
            _ => {
              // Handle other cases if needed - potentially log for debugging
            }
          }
        }
      }
    }
    */

    Ok(())
  }

  async fn create_consume_shared_module(
    &self,
    context: &Context,
    request: &str,
    config: Arc<ConsumeOptions>,
    add_diagnostic: &mut impl FnMut(Diagnostic),
  ) -> Result<ConsumeSharedModule> {
    let direct_fallback = matches!(&config.import, Some(i) if RELATIVE_REQUEST.is_match(i) | ABSOLUTE_REQUEST.is_match(i));
    let import_resolved = match &config.import {
      None => None,
      Some(import) => {
        let resolver = self.get_resolver();
        resolver
          .resolve(
            if direct_fallback {
              self.get_context()
            } else {
              context.clone()
            }
            .as_ref(),
            import,
          )
          .await
          .map_err(|_e| {
            add_diagnostic(Diagnostic::error(
              "ModuleNotFoundError".into(),
              format!("resolving fallback for shared module {request}"),
            ))
          })
          .ok()
      }
    }
    .and_then(|i| match i {
      ResolveResult::Resource(r) => Some(r.path.as_str().to_string()),
      ResolveResult::Ignored => None,
    });
    let required_version = self
      .get_required_version(context, request, config.clone(), add_diagnostic)
      .await;

    Ok(ConsumeSharedModule::new(
      if direct_fallback {
        self.get_context()
      } else {
        context.clone()
      },
      ConsumeOptions {
        import: import_resolved
          .is_some()
          .then(|| config.import.clone())
          .and_then(|i| i),
        import_resolved,
        share_key: config.share_key.clone(),
        share_scope: config.share_scope.clone(),
        required_version,
        package_name: config.package_name.clone(),
        strict_version: config.strict_version,
        singleton: config.singleton,
        eager: config.eager,
      },
    ))
  }
}

#[plugin_hook(CompilerThisCompilation for ConsumeSharedPlugin)]
async fn this_compilation(
  &self,
  compilation: &mut Compilation,
  params: &mut CompilationParams,
) -> Result<()> {
  compilation.set_dependency_factory(
    DependencyType::ConsumeSharedFallback,
    params.normal_module_factory.clone(),
  );
  if self.compiler_context.get().is_none() {
    self.init_context(compilation);
  }
  if self.resolver.get().is_none() {
    self.init_resolver(compilation);
  }
  if self.matched_consumes.get().is_none() {
    self
      .init_matched_consumes(compilation, self.get_resolver())
      .await;
  }
  Ok(())
}

#[plugin_hook(CompilationFinishModules for ConsumeSharedPlugin)]
async fn finish_modules(&self, compilation: &mut Compilation) -> Result<()> {
  // Find all ConsumeShared modules and copy metadata from their fallbacks
  let consume_shared_modules: Vec<ModuleIdentifier> = compilation
    .get_module_graph()
    .modules()
    .keys()
    .filter(|id| {
      if let Some(module) = compilation.get_module_graph().module_by_identifier(id) {
        module.module_type() == &ModuleType::ConsumeShared
      } else {
        false
      }
    })
    .copied()
    .collect();

  // Process each ConsumeShared module individually to avoid borrow checker issues
  for consume_shared_id in consume_shared_modules {
    // First, set the consume_shared_key in the fallback module's BuildMeta
    Self::set_consume_shared_key_in_fallback(compilation, &consume_shared_id)?;

    if self.options.enhanced {
      // Use enhanced copying that includes usage analysis
      Self::enhanced_copy_fallback_metadata_to_consume_shared(compilation, &consume_shared_id)?;
    } else {
      // Use standard metadata copying
      Self::copy_fallback_metadata_to_consume_shared(compilation, &consume_shared_id)?;
    }
  }

  // Phase 2: Unified shared detection optimization for all modules
  Self::mark_shared_descendants(compilation)?;

  Ok(())
}

#[plugin_hook(NormalModuleFactoryFactorize for ConsumeSharedPlugin)]
async fn factorize(&self, data: &mut ModuleFactoryCreateData) -> Result<Option<BoxModule>> {
  let dep = data.dependencies[0]
    .as_module_dependency()
    .expect("should be module dependency");
  if matches!(
    dep.dependency_type(),
    DependencyType::ConsumeSharedFallback
      | DependencyType::ProvideModuleForShared
      | DependencyType::ShareContainerFallback
  ) {
    return Ok(None);
  }
  let request = dep.request();
  let consumes = self.get_matched_consumes();

  if let Some(matched) = consumes.unresolved.get(request) {
    let mut add_diagnostic = |d| data.diagnostics.push(d);
    match self
      .create_consume_shared_module(&data.context, request, matched.clone(), &mut add_diagnostic)
      .await
    {
      Ok(module) => return Ok(Some(module.boxed())),
      Err(_) => return Ok(None), // Error already handled via diagnostic
    }
  }
  for (prefix, options) in &consumes.prefixed {
    if request.starts_with(prefix) {
      let remainder = &request[prefix.len()..];
      let mut add_diagnostic = |d| data.diagnostics.push(d);
      match self
        .create_consume_shared_module(
          &data.context,
          request,
          Arc::new(ConsumeOptions {
            import: options.import.as_ref().map(|i| i.to_owned() + remainder),
            import_resolved: options.import_resolved.clone(),
            share_key: options.share_key.clone() + remainder,
            share_scope: options.share_scope.clone(),
            required_version: options.required_version.clone(),
            package_name: options.package_name.clone(),
            strict_version: options.strict_version,
            singleton: options.singleton,
            eager: options.eager,
          }),
          &mut add_diagnostic,
        )
        .await
      {
        Ok(module) => return Ok(Some(module.boxed())),
        Err(_) => return Ok(None), // Error already handled via diagnostic
      }
    }
  }
  Ok(None)
}

#[plugin_hook(NormalModuleFactoryCreateModule for ConsumeSharedPlugin)]
async fn create_module(
  &self,
  data: &mut ModuleFactoryCreateData,
  create_data: &mut NormalModuleCreateData,
) -> Result<Option<BoxModule>> {
  if matches!(
    data.dependencies[0].dependency_type(),
    DependencyType::ConsumeSharedFallback
      | DependencyType::ProvideModuleForShared
      | DependencyType::ShareContainerFallback
  ) {
    return Ok(None);
  }
  let resource = create_data.resource_resolve_data.resource();
  let consumes = self.get_matched_consumes();

  if let Some(options) = consumes.resolved.get(resource) {
    let mut add_diagnostic = |d| data.diagnostics.push(d);
    match self
      .create_consume_shared_module(
        &data.context,
        resource,
        options.clone(),
        &mut add_diagnostic,
      )
      .await
    {
      Ok(module) => return Ok(Some(module.boxed())),
      Err(_) => return Ok(None), // Error already handled via diagnostic
    }
  }
  Ok(None)
}

#[plugin_hook(NormalModuleFactoryModule for ConsumeSharedPlugin)]
async fn normal_module_factory_module(
  &self,
  data: &mut ModuleFactoryCreateData,
  _create_data: &mut NormalModuleCreateData,
  module: &mut BoxModule,
) -> Result<()> {
  // Check if this is a ConsumeSharedFallback dependency
  if !data.dependencies.is_empty()
    && matches!(
      data.dependencies[0].dependency_type(),
      DependencyType::ConsumeSharedFallback
    )
  {
    // Get the issuer identifier (ConsumeShared module)
    if let Some(issuer_id) = &data.issuer_identifier {
      // Try to get the share_key from the issuer module
      // Since we're in the module factory, we need to check if the issuer is a ConsumeSharedModule
      // The issuer should be in the dependency's context
      if let Some(_dep) = data.dependencies[0]
        .as_any()
        .downcast_ref::<super::consume_shared_fallback_dependency::ConsumeSharedFallbackDependency>(
      ) {
        // Extract share key from the issuer identifier
        // ConsumeShared module identifiers have the format:
        // "consume shared module ({share_scope}) {share_key}@{version}..."
        let issuer_str = issuer_id.to_string();
        if let Some(share_key) = Self::extract_share_key_from_identifier(&issuer_str) {
          // Set the consume_shared_key in the fallback module's BuildMeta
          module.build_meta_mut().consume_shared_key = Some(share_key);
        }
      }
    }
  }
  Ok(())
}

#[plugin_hook(CompilationAdditionalTreeRuntimeRequirements for ConsumeSharedPlugin)]
async fn additional_tree_runtime_requirements(
  &self,
  compilation: &mut Compilation,
  chunk_ukey: &ChunkUkey,
  runtime_requirements: &mut RuntimeGlobals,
) -> Result<()> {
  runtime_requirements.insert(RuntimeGlobals::MODULE);
  runtime_requirements.insert(RuntimeGlobals::MODULE_CACHE);
  runtime_requirements.insert(RuntimeGlobals::MODULE_FACTORIES_ADD_ONLY);
  runtime_requirements.insert(RuntimeGlobals::SHARE_SCOPE_MAP);
  runtime_requirements.insert(RuntimeGlobals::INITIALIZE_SHARING);
  runtime_requirements.insert(RuntimeGlobals::HAS_OWN_PROPERTY);
  compilation.add_runtime_module(
    chunk_ukey,
    Box::new(ConsumeSharedRuntimeModule::new(self.options.enhanced)),
  )?;
  Ok(())
}

impl ConsumeSharedPlugin {
  /// Phase 2: Unified shared descendant detection optimization for ESM and CommonJS
  /// Processes both ESM and CommonJS modules to maximize performance benefits
  fn mark_shared_descendants(compilation: &mut Compilation) -> Result<()> {
    use std::collections::{HashMap, HashSet, VecDeque};

    // Collect all module data we need upfront to avoid borrow checker issues
    let module_data: HashMap<ModuleIdentifier, (bool, Option<String>, Option<String>, ModuleType)> = {
      let module_graph = compilation.get_module_graph();
      module_graph
        .modules()
        .iter()
        .map(|(id, module)| {
          let build_meta = module.build_meta();
          (
            *id,
            (
              build_meta.esm,
              build_meta.consume_shared_key.clone(),
              build_meta.shared_key.clone(),
              *module.module_type(),
            ),
          )
        })
        .collect()
    };

    // Collect all connections upfront
    let connections: HashMap<ModuleIdentifier, Vec<ModuleIdentifier>> = {
      let module_graph = compilation.get_module_graph();
      module_data
        .keys()
        .map(|module_id| {
          let outgoing: Vec<_> = module_graph
            .get_outgoing_connections(module_id)
            .map(|conn| *conn.module_identifier())
            .collect();
          (*module_id, outgoing)
        })
        .collect()
    };

    let mut queue = VecDeque::new();
    let mut visited = HashSet::new();
    let mut shared_descendants = HashSet::new();
    let mut effective_keys = HashMap::new();

    // Step 1: Find directly shared modules (both ESM and CommonJS)
    for (module_id, (_is_esm, consume_shared_key, shared_key, module_type)) in &module_data {
      // Phase 2: Process all modules, not just ESM

      // Check if this is a directly shared module
      let is_directly_shared = consume_shared_key.is_some()
        || shared_key.is_some()
        || module_type == &ModuleType::ConsumeShared
        || module_type == &ModuleType::ProvideShared;

      if is_directly_shared {
        shared_descendants.insert(*module_id);

        // Set effective shared key (prioritize consume_shared_key > shared_key)
        if let Some(effective_key) = consume_shared_key.clone().or_else(|| shared_key.clone()) {
          effective_keys.insert(*module_id, effective_key);
        }

        queue.push_back(*module_id);
      }
    }

    // Step 2: BFS to mark all descendants (ESM and CommonJS)
    while let Some(current_id) = queue.pop_front() {
      if !visited.insert(current_id) {
        continue;
      }

      let parent_shared_key = effective_keys.get(&current_id).cloned();

      if let Some(outgoing) = connections.get(&current_id) {
        for target_id in outgoing {
          // Phase 2: Process all modules regardless of type
          if let Some((_is_esm, _, _, _)) = module_data.get(target_id) {
            // Check if target already processed
            if shared_descendants.contains(target_id) {
              continue;
            }

            // Mark target as shared descendant
            shared_descendants.insert(*target_id);

            // Inherit parent's shared key if target doesn't have one
            if !effective_keys.contains_key(target_id)
              && let Some(key) = parent_shared_key.as_ref()
            {
              effective_keys.insert(*target_id, key.clone());
            }

            queue.push_back(*target_id);
          }
        }
      }
    }

    // Step 3: Apply all mutations to the compilation
    for (module_id, (_is_esm, _, _, _)) in &module_data {
      // Phase 2: Apply to all modules, not just ESM

      if let Some(module) = compilation
        .get_module_graph_mut()
        .module_by_identifier_mut(module_id)
      {
        let build_meta = module.build_meta_mut();

        // Set shared descendant status
        build_meta.is_shared_descendant = Some(shared_descendants.contains(module_id));

        // Set effective shared key if we have one
        if let Some(effective_key) = effective_keys.get(module_id) {
          build_meta.effective_shared_key = Some(effective_key.clone());

          // IMPORTANT: Also set consume_shared_key if not already set
          // This is needed for macro generation in ConsumeSharedExportsDependency
          if build_meta.consume_shared_key.is_none() {
            build_meta.consume_shared_key = Some(effective_key.clone());
          }
        }
      }
    }

    Ok(())
  }
}

impl Plugin for ConsumeSharedPlugin {
  fn name(&self) -> &'static str {
    "rspack.ConsumeSharedPlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx
      .compiler_hooks
      .this_compilation
      .tap(this_compilation::new(self));
    ctx
      .normal_module_factory_hooks
      .factorize
      .tap(factorize::new(self));
    ctx
      .normal_module_factory_hooks
      .create_module
      .tap(create_module::new(self));
    ctx
      .normal_module_factory_hooks
      .module
      .tap(normal_module_factory_module::new(self));
    ctx
      .compilation_hooks
      .additional_tree_runtime_requirements
      .tap(additional_tree_runtime_requirements::new(self));
    ctx
      .compilation_hooks
      .finish_modules
      .tap(finish_modules::new(self));
    Ok(())
  }
}
