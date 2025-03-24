#![feature(array_windows)]

use std::sync::Arc;

use derive_more::Debug;
use itertools::Itertools;
use rspack_collections::{Identifier, IdentifierMap, IdentifierSet};
use rspack_core::{
  ApplyContext, Compilation, CompilerOptions, ModuleIdentifier, Plugin, PluginContext,
};
use rspack_core::{CompilationOptimizeModules, DependencyType};
use rspack_error::{Diagnostic, Result};
use rspack_hook::{plugin, plugin_hook};
use rspack_regex::RspackRegex;

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
      &mut IdentifierSet::default(),
      &mut vec![initial_module_id],
      &mut cycles,
    );
    cycles
  }

  fn recurse_dependencies(
    &self,
    current_module_id: ModuleIdentifier,
    seen_set: &mut IdentifierSet,
    current_path: &mut Vec<ModuleIdentifier>,
    found_cycles: &mut Vec<Vec<ModuleIdentifier>>,
  ) {
    seen_set.insert(current_module_id);
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

      // If a module has already been encountered in this traversal, then by
      // necessity it is either already part of a cycle being detected as
      // captured above, or it _and all of its dependencies_ are not part of
      // any cycles involving the current module. If that were not true, then
      // this module would have already been encountered previously.
      if seen_set.contains(target_id) {
        continue;
      }

      self.recurse_dependencies(*target_id, seen_set, current_path, found_cycles);
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
  let modules = module_graph.modules();

  let mut module_map: IdentifierMap<GraphModule> = IdentifierMap::default();
  module_map.reserve(modules.len());
  for (id, module) in modules {
    let mut graph_module = GraphModule::new(id, module.original_source().is_some());
    for dependency_id in module.get_dependencies() {
      let Some(dependency) = module_graph.dependency_by_id(dependency_id) else {
        continue;
      };
      let Some(dependent_module) = module_graph.get_module_by_dependency_id(dependency_id) else {
        continue;
      };
      // Only include dependencies that represent a real source file.
      if dependent_module.original_source().is_none() {
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
  Arc<dyn Fn(String, Vec<String>, &mut Compilation) -> Result<()> + Send + Sync>;
pub type CompilationHookFn = Arc<dyn Fn(&mut Compilation) -> Result<()> + Send + Sync>;

#[derive(Debug)]
pub struct CircularDependencyRspackPluginOptions {
  /// When `true`, the plugin will emit Error diagnostics rather than the
  /// default Warn severity.
  pub fail_on_error: bool,
  /// When `true`, asynchronous imports like `import("some-module")` will not
  /// be considered connections that can create cycles.
  pub allow_async_cycles: bool,
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
  ) -> bool {
    for [module_id, target_id] in cycle.array_windows::<2>() {
      // If any dependency in the cycle is purely asynchronous, then it does not count as a runtime
      // circular dependency, since execution order will be guaranteed.
      if module_map[module_id].dependencies[target_id].is_asynchronous_only() {
        return true;
      }

      // Not all cycles are errors, so filter out any cycles containing
      // explicitly-ignored modules.
      if self.is_ignored_module(module_id) || self.is_ignored_connection(module_id, target_id) {
        return true;
      }
    }
    false
  }

  fn handle_cycle_ignored(
    &self,
    entrypoint: String,
    cycle: Vec<ModuleIdentifier>,
    compilation: &mut Compilation,
  ) -> Result<()> {
    match &self.options.on_ignored {
      Some(callback) => callback(
        entrypoint,
        cycle.iter().map(ToString::to_string).collect(),
        compilation,
      ),
      _ => Ok(()),
    }
  }

  fn handle_cycle_detected(
    &self,
    entrypoint: String,
    cycle: Vec<ModuleIdentifier>,
    compilation: &mut Compilation,
  ) -> Result<()> {
    if let Some(callback) = &self.options.on_detected {
      return callback(
        entrypoint,
        cycle.iter().map(ToString::to_string).collect(),
        compilation,
      );
    }

    let diagnostic_factory = if self.options.fail_on_error {
      Diagnostic::error
    } else {
      Diagnostic::warn
    };

    compilation.push_diagnostic(diagnostic_factory(
      "Circular Dependency".to_string(),
      cycle.iter().join(" -> "),
    ));
    Ok(())
  }
}

#[plugin_hook(CompilationOptimizeModules for CircularDependencyRspackPlugin)]
async fn optimize_modules(&self, compilation: &mut Compilation) -> Result<Option<bool>> {
  if let Some(on_start) = &self.options.on_start {
    on_start(compilation)?;
  };

  let module_map = build_module_map(compilation);
  let mut detector = CycleDetector::new(&module_map);
  for (entrypoint_name, chunk_group_key) in compilation.entrypoints.clone() {
    let chunk_group = compilation
      .chunk_group_by_ukey
      .get(&chunk_group_key)
      .expect("Compilation should contain entrypoint chunk groups");
    let entry_modules = compilation
      .chunk_graph
      .get_chunk_entry_modules(&chunk_group.get_entry_point_chunk());

    for module_id in entry_modules {
      // Only consider entrypoint modules coming from existing source code.
      // This skips internal things like runtime and generated chunks.
      if !detector.get_module(&module_id).is_source {
        continue;
      };

      for cycle in detector.find_cycles_from(module_id) {
        if self.is_cycle_ignored(&module_map, &cycle) {
          self.handle_cycle_ignored(entrypoint_name.clone(), cycle, compilation)?
        } else {
          self.handle_cycle_detected(entrypoint_name.clone(), cycle, compilation)?
        }
      }
    }
  }

  if let Some(on_end) = &self.options.on_end {
    on_end(compilation)?;
  }
  Ok(None)
}

// implement apply method for the plugin
impl Plugin for CircularDependencyRspackPlugin {
  fn apply(&self, ctx: PluginContext<&mut ApplyContext>, _options: &CompilerOptions) -> Result<()> {
    ctx
      .context
      .compilation_hooks
      .optimize_modules
      .tap(optimize_modules::new(self));
    Ok(())
  }
}
