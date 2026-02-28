use cow_utils::CowUtils;
use derive_more::Debug;
use futures::future::BoxFuture;
use itertools::Itertools;
use rspack_collections::{Identifier, IdentifierMap};
use rspack_core::{
  Compilation, CompilationOptimizeModules, DependencyType, ModuleIdentifier, Plugin,
};
use rspack_error::{Diagnostic, Result};
use rspack_hook::{plugin, plugin_hook};
use rspack_regex::RspackRegex;
use rustc_hash::FxHashSet as HashSet;

struct CycleDetector<'a> {
  module_map: &'a IdentifierMap<GraphModule>,
}

impl<'a> CycleDetector<'a> {
  fn new(module_map: &'a IdentifierMap<GraphModule>) -> Self {
    Self { module_map }
  }

  fn get_module(&self, id: &ModuleIdentifier) -> &GraphModule {
    self
      .module_map
      .get(id)
      .expect("Module map should only contain references to existing modules")
  }

  /// Returns all dependency cycles contained in the dependency graph starting at
  /// `initial_module_id`.
  fn find_cycles_from(
    &mut self,
    initial_module_id: ModuleIdentifier,
  ) -> Vec<Vec<ModuleIdentifier>> {
    let mut cycles = vec![];
    self.recurse_dependencies(
      initial_module_id,
      &mut HashSet::default(),
      &mut vec![initial_module_id],
      &mut cycles,
    );
    // sort to keep output stable
    cycles.sort();
    cycles
  }

  fn recurse_dependencies(
    &self,
    current_module_id: ModuleIdentifier,
    seen_relations: &mut HashSet<(ModuleIdentifier, ModuleIdentifier)>,
    current_path: &mut Vec<ModuleIdentifier>,
    found_cycles: &mut Vec<Vec<ModuleIdentifier>>,
  ) {
    current_path.push(current_module_id);
    for target_id in self.get_module(&current_module_id).dependencies.keys() {
      // If the current path already contains the dependent module, then it
      // creates a cycle and doesn't need to be traversed further. Otherwise,
      // recurse through that dependency to keep searching.
      //
      // It's possible for this cycle to connect back at any point in the
      // current path, so the recorded cycle is split at that position to only
      // include the modules involved in the cycle.
      if let Some(cycle_start) = current_path
        .iter()
        .rposition(|element| element == target_id)
      {
        let mut cycle = current_path[cycle_start..].to_vec();
        cycle.push(*target_id);
        found_cycles.push(cycle);
        continue;
      }

      if seen_relations.contains(&(current_module_id, *target_id)) {
        continue;
      }
      seen_relations.insert((current_module_id, *target_id));
      self.recurse_dependencies(*target_id, seen_relations, current_path, found_cycles);
    }
    current_path.pop();
  }
}

/// Single Dependency representing all types of connections to a target module.
#[derive(Debug)]
struct AggregatedDependency {
  #[allow(unused)]
  target_id: ModuleIdentifier,
  types: Vec<DependencyType>,
}

impl AggregatedDependency {
  fn new(target_id: ModuleIdentifier, types: Vec<DependencyType>) -> Self {
    Self { target_id, types }
  }

  /// Returns true if _every_ type of this dependency is dynamic, meaning there
  /// are _no_ static connections that would cause a cycle.
  fn is_asynchronous_only(&self) -> bool {
    self.types.iter().all(|ty| {
      matches!(
        ty,
        // This list of types is made purely by intuition, since dynamic
        // dependencies are not the same as weak or "async" dependencies in
        // the context of circular detection.
        DependencyType::DynamicImport
          | DependencyType::DynamicImportEager
          | DependencyType::LazyImport
          | DependencyType::ImportMetaHotAccept
          | DependencyType::ImportMetaHotDecline
          | DependencyType::ImportMetaResolve
          | DependencyType::ModuleHotAccept
          | DependencyType::ModuleHotDecline
          | DependencyType::RequireResolve
      )
    })
  }
}

#[derive(Debug)]
struct GraphModule {
  #[allow(unused)]
  id: Identifier,
  is_source: bool,
  dependencies: IdentifierMap<AggregatedDependency>,
}

impl GraphModule {
  fn new(id: ModuleIdentifier, is_source: bool) -> Self {
    Self {
      id,
      is_source,
      dependencies: IdentifierMap::default(),
    }
  }

  fn add_dependency(&mut self, target_id: ModuleIdentifier, ty: DependencyType) {
    self
      .dependencies
      .entry(target_id)
      .and_modify(|dep| dep.types.push(ty))
      .or_insert_with(|| AggregatedDependency::new(target_id, vec![ty]));
  }
}

fn build_module_map(compilation: &Compilation) -> IdentifierMap<GraphModule> {
  let module_graph = compilation.get_module_graph();

  let mut module_map: IdentifierMap<GraphModule> = IdentifierMap::default();
  module_map.reserve(module_graph.modules_len());
  for (&id, module) in module_graph.modules() {
    let mut graph_module = GraphModule::new(id, module.source().is_some());
    // if allow async cycles, the async dependencies should not be collected to check for cycles
    for dependency_id in module_graph
      .get_outgoing_connections(&id)
      .map(|conn| conn.dependency_id)
    {
      let dependency = module_graph.dependency_by_id(&dependency_id);
      let Some(dependent_module) = module_graph.get_module_by_dependency_id(&dependency_id) else {
        continue;
      };
      // Only include dependencies that represent a real source file.
      if dependent_module.source().is_none() {
        continue;
      }
      // Self dependencies are added in various ways, but don't mean anything here.
      if dependent_module.identifier() == id {
        continue;
      }
      graph_module.add_dependency(dependent_module.identifier(), *dependency.dependency_type());
    }

    module_map.insert(id, graph_module);
  }
  module_map
}

#[derive(Debug)]
pub enum CircularDependencyIgnoredConnectionEntry {
  String(String),
  Pattern(RspackRegex),
}

impl CircularDependencyIgnoredConnectionEntry {
  pub fn test(&self, value: &str) -> bool {
    match self {
      Self::String(string) => value.contains(string),
      Self::Pattern(pattern) => pattern.test(value),
    }
  }
}

#[derive(Debug)]
pub struct CircularDependencyIgnoredConnection(
  pub CircularDependencyIgnoredConnectionEntry,
  pub CircularDependencyIgnoredConnectionEntry,
);

impl CircularDependencyIgnoredConnection {
  /// Returns true if the given connection should be ignored according to this
  /// instance. If either side of this connection is [None], only the other
  /// side is required to match for it to be ignored.
  pub fn is_ignored(&self, from: &str, to: &str) -> bool {
    self.0.test(from) && self.1.test(to)
  }
}

pub type CycleHandlerFn =
  Box<dyn Fn(String, Vec<String>) -> BoxFuture<'static, Result<()>> + Sync + Send>;
pub type CompilationHookFn = Box<dyn Fn() -> BoxFuture<'static, Result<()>> + Sync + Send>;

#[derive(Debug)]
pub struct CircularDependencyRspackPluginOptions {
  /// When `true`, the plugin will emit Error diagnostics rather than the
  /// default Warn severity.
  pub fail_on_error: bool,
  /// Cycles containing any module name that matches this regex will not be
  /// counted as a cycle.
  pub exclude: Option<RspackRegex>,
  /// List of dependency connections that should not count for creating cycles.
  /// Connections are represented as `[from, to]`, where each entry must be an
  /// exact match for the module path.
  pub ignored_connections: Option<Vec<CircularDependencyIgnoredConnection>>,
  /// Handler function called for every detected cycle. Providing this handler
  /// overrides the default behavior of adding diagnostics to the compilation.
  #[debug(skip)]
  pub on_detected: Option<CycleHandlerFn>,
  #[debug(skip)]
  pub on_ignored: Option<CycleHandlerFn>,
  #[debug(skip)]
  pub on_start: Option<CompilationHookFn>,
  #[debug(skip)]
  pub on_end: Option<CompilationHookFn>,
}

#[plugin]
#[derive(Debug)]
pub struct CircularDependencyRspackPlugin {
  options: CircularDependencyRspackPluginOptions,
}

impl CircularDependencyRspackPlugin {
  pub fn new(options: CircularDependencyRspackPluginOptions) -> Self {
    Self::new_inner(options)
  }

  fn is_ignored_module(&self, name: &str) -> bool {
    match &self.options.exclude {
      Some(pattern) => pattern.test(name),
      None => false,
    }
  }

  fn is_ignored_connection(&self, from: &str, to: &str) -> bool {
    match &self.options.ignored_connections {
      Some(ignored_connections) => ignored_connections
        .iter()
        .any(|connection| connection.is_ignored(from, to)),
      None => false,
    }
  }

  fn is_cycle_ignored(
    &self,
    module_map: &IdentifierMap<GraphModule>,
    cycle: &[ModuleIdentifier],
    compilation: &Compilation,
  ) -> bool {
    for window in cycle.windows(2) {
      let [module_id, target_id] = [&window[0], &window[1]];
      // If any dependency in the cycle is purely asynchronous, then it does not count as a runtime
      // circular dependency, since execution order will be guaranteed.
      if module_map[module_id].dependencies[target_id].is_asynchronous_only() {
        return true;
      }

      let Some(module) = compilation
        .module_by_identifier(module_id)
        .and_then(|m| m.as_normal_module())
      else {
        continue;
      };

      // Not all cycles are errors, so filter out any cycles containing
      // explicitly-ignored modules.
      if self.is_ignored_module(module.resource_resolved_data().resource())
        || self.is_ignored_connection(module_id, target_id)
      {
        return true;
      }
    }
    false
  }

  async fn handle_cycle_ignored(
    &self,
    entrypoint: String,
    cycle: Vec<ModuleIdentifier>,
    _diagnostics: &mut Vec<Diagnostic>,
  ) -> Result<()> {
    match &self.options.on_ignored {
      Some(callback) => callback(entrypoint, cycle.iter().map(ToString::to_string).collect()).await,
      _ => Ok(()),
    }
  }

  async fn handle_cycle_detected(
    &self,
    entrypoint: String,
    cycle: Vec<ModuleIdentifier>,
    compilation: &Compilation,
    diagnostics: &mut Vec<Diagnostic>,
  ) -> Result<()> {
    if let Some(callback) = &self.options.on_detected {
      return callback(entrypoint, cycle.iter().map(ToString::to_string).collect()).await;
    }

    let diagnostic_factory = if self.options.fail_on_error {
      Diagnostic::error
    } else {
      Diagnostic::warn
    };

    let cwd = std::env::current_dir()
      .expect("cwd should be available")
      .to_string_lossy()
      .to_string();

    // remove the root path here.
    let cycle_without_root: Vec<String> = cycle
      .iter()
      .filter_map(|module_identifier| {
        compilation
          .module_by_identifier(module_identifier)
          .map(|module| {
            module
              .readable_identifier(&compilation.options.context)
              .to_string()
              .cow_replace(&cwd, "")
              .trim_start_matches('/')
              .trim_start_matches('\\')
              .to_string()
          })
      })
      .collect();

    diagnostics.push(diagnostic_factory(
      "Circular Dependency".to_string(),
      format!(
        "Circular dependency detected:\n {}",
        cycle_without_root.iter().join(" -> ")
      ),
    ));
    Ok(())
  }
}

#[plugin_hook(CompilationOptimizeModules for CircularDependencyRspackPlugin)]
async fn optimize_modules(
  &self,
  compilation: &Compilation,
  diagnostics: &mut Vec<Diagnostic>,
) -> Result<Option<bool>> {
  if let Some(on_start) = &self.options.on_start {
    on_start().await?;
  };

  let module_map = build_module_map(compilation);
  let mut detector = CycleDetector::new(&module_map);
  for (entrypoint_name, chunk_group_key) in
    compilation.build_chunk_graph_artifact.entrypoints.clone()
  {
    let chunk_group = compilation
      .build_chunk_graph_artifact
      .chunk_group_by_ukey
      .get(&chunk_group_key)
      .expect("Compilation should contain entrypoint chunk groups");
    let mut entry_modules = compilation
      .build_chunk_graph_artifact
      .chunk_graph
      .get_chunk_entry_modules(&chunk_group.get_entrypoint_chunk());

    entry_modules.sort();

    for module_id in entry_modules {
      // Only consider entrypoint modules coming from existing source code.
      // This skips internal things like runtime and generated chunks.
      if !detector.get_module(&module_id).is_source {
        continue;
      };

      for cycle in detector.find_cycles_from(module_id) {
        if self.is_cycle_ignored(&module_map, &cycle, compilation) {
          self
            .handle_cycle_ignored(entrypoint_name.clone(), cycle, diagnostics)
            .await?
        } else {
          self
            .handle_cycle_detected(entrypoint_name.clone(), cycle, compilation, diagnostics)
            .await?
        }
      }
    }
  }

  if let Some(on_end) = &self.options.on_end {
    on_end().await?;
  }
  Ok(None)
}

// implement apply method for the plugin
impl Plugin for CircularDependencyRspackPlugin {
  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx
      .compilation_hooks
      .optimize_modules
      .tap(optimize_modules::new(self));
    Ok(())
  }
}
