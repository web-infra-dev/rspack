use rspack_collections::{IdentifierMap, IdentifierSet};
use rspack_core::{
  Compilation, CompilationOptimizeModules, ContextTypePrefix, DependencyType, ModuleGraph,
  ModuleIdentifier, Plugin,
};
use rspack_error::{Diagnostic, Result};
use rspack_hook::{plugin, plugin_hook};
use smallvec::SmallVec;

#[derive(Clone, Copy, Default, Eq, Ord, PartialEq, PartialOrd)]
struct ModuleIndex(u32);

impl ModuleIndex {
  #[inline]
  fn from_usize(index: usize) -> Self {
    Self(u32::try_from(index).expect("module index should fit in u32"))
  }

  #[inline]
  fn to_usize(self) -> usize {
    usize::try_from(self.0).expect("module index should fit in usize")
  }

  #[inline]
  fn incremented(self) -> Self {
    Self(
      self
        .0
        .checked_add(1)
        .expect("module index should fit in u32"),
    )
  }
}

type ModuleEdges = SmallVec<[ModuleIndex; 6]>;

struct CycleGraph {
  modules: Vec<ModuleIdentifier>,
  edges: Vec<ModuleEdges>,
  self_loop_modules: Vec<bool>,
}

impl CycleGraph {
  fn build(module_graph: &ModuleGraph) -> Self {
    let mut modules = module_graph
      .modules()
      .filter_map(|(&id, module)| module.source().is_some().then_some(id))
      .collect::<Vec<_>>();
    modules.sort_unstable();

    let mut indexes = IdentifierMap::default();
    indexes.reserve(modules.len());
    for (index, id) in modules.iter().enumerate() {
      indexes.insert(*id, ModuleIndex::from_usize(index));
    }

    let mut edges = vec![ModuleEdges::new(); modules.len()];
    let mut self_loop_modules = vec![false; modules.len()];
    for (index, module_id) in modules.iter().enumerate() {
      let module_edges = &mut edges[index];
      for connection in module_graph.get_outgoing_connections(module_id) {
        let dependency = module_graph.dependency_by_id(&connection.dependency_id);
        if should_ignore_dependency_type(*dependency.dependency_type()) {
          continue;
        }

        let target_id = *connection.module_identifier();
        if target_id == *module_id {
          self_loop_modules[index] = true;
          continue;
        }

        if let Some(&target_index) = indexes.get(&target_id) {
          module_edges.push(target_index);
        }
      }

      module_edges.sort_unstable();
      module_edges.dedup();
    }

    Self {
      modules,
      edges,
      self_loop_modules,
    }
  }

  #[inline]
  fn edges(&self, index: ModuleIndex) -> &ModuleEdges {
    &self.edges[index.to_usize()]
  }

  #[inline]
  fn module(&self, index: ModuleIndex) -> ModuleIdentifier {
    self.modules[index.to_usize()]
  }

  #[inline]
  fn has_self_loop(&self, index: ModuleIndex) -> bool {
    self.self_loop_modules[index.to_usize()]
  }
}

#[derive(Clone, Copy, Default)]
struct NodeState {
  index: Option<ModuleIndex>,
  low_link: ModuleIndex,
  on_stack: bool,
}

struct ConnectFrame {
  module_index: ModuleIndex,
  next_edge_index: usize,
  parent_index: Option<ModuleIndex>,
}

struct CycleDetector<'a> {
  graph: &'a CycleGraph,
  next_index: ModuleIndex,
  states: Vec<NodeState>,
  stack: Vec<ModuleIndex>,
  circular_modules: IdentifierSet,
}

impl<'a> CycleDetector<'a> {
  fn new(graph: &'a CycleGraph) -> Self {
    let size = graph.modules.len();
    Self {
      graph,
      next_index: ModuleIndex::default(),
      states: vec![NodeState::default(); size],
      stack: Vec::with_capacity(size),
      circular_modules: IdentifierSet::default(),
    }
  }

  fn find_circular_modules(mut self) -> IdentifierSet {
    for module_index in 0..self.graph.modules.len() {
      let module_index = ModuleIndex::from_usize(module_index);
      if self.state(module_index).index.is_none() {
        self.connect(module_index);
      }
    }

    self.circular_modules
  }

  fn connect(&mut self, module_index: ModuleIndex) {
    self.start_connect(module_index);
    let mut visit_stack = vec![ConnectFrame {
      module_index,
      next_edge_index: 0,
      parent_index: None,
    }];

    while !visit_stack.is_empty() {
      let next_target = {
        let frame = visit_stack
          .last_mut()
          .expect("visit stack should not be empty");
        let module_index = frame.module_index;
        let edges = self.graph.edges(module_index);
        if frame.next_edge_index < edges.len() {
          let target_index = edges[frame.next_edge_index];
          frame.next_edge_index += 1;
          Some((module_index, target_index))
        } else {
          None
        }
      };

      if let Some((module_index, target_index)) = next_target {
        if self.state(target_index).index.is_none() {
          self.start_connect(target_index);
          visit_stack.push(ConnectFrame {
            module_index: target_index,
            next_edge_index: 0,
            parent_index: Some(module_index),
          });
        } else if self.state(target_index).on_stack {
          self.state_mut(module_index).low_link = self
            .state(module_index)
            .low_link
            .min(self.state(target_index).index.expect("indexed"));
        }
        continue;
      }

      let frame = visit_stack.pop().expect("visit stack should not be empty");
      let module_index = frame.module_index;
      self.finish_connect(module_index);

      if let Some(parent_index) = frame.parent_index {
        self.state_mut(parent_index).low_link = self
          .state(parent_index)
          .low_link
          .min(self.state(module_index).low_link);
      }
    }
  }

  fn start_connect(&mut self, module_index: ModuleIndex) {
    let index = self.next_index;
    self.next_index = self.next_index.incremented();
    self.states[module_index.to_usize()] = NodeState {
      index: Some(index),
      low_link: index,
      on_stack: true,
    };
    self.stack.push(module_index);
  }

  fn finish_connect(&mut self, module_index: ModuleIndex) {
    if self.state(module_index).low_link == self.state(module_index).index.expect("indexed") {
      let mut component = Vec::new();
      loop {
        let current = self.stack.pop().expect("root should be on the stack");
        self.state_mut(current).on_stack = false;
        component.push(current);
        if current == module_index {
          break;
        }
      }

      if component.len() > 1 || self.graph.has_self_loop(module_index) {
        self
          .circular_modules
          .extend(component.into_iter().map(|index| self.graph.module(index)));
      }
    }
  }

  #[inline]
  fn state(&self, index: ModuleIndex) -> &NodeState {
    &self.states[index.to_usize()]
  }

  #[inline]
  fn state_mut(&mut self, index: ModuleIndex) -> &mut NodeState {
    &mut self.states[index.to_usize()]
  }
}

#[plugin]
#[derive(Debug, Default)]
pub struct CircularModulesInfoPlugin {}

fn should_ignore_dependency_type(ty: DependencyType) -> bool {
  matches!(
    ty,
    // Async boundaries. These edges do not synchronously evaluate the target
    // module while the current module is initializing.
    DependencyType::DynamicImport
      | DependencyType::DynamicImportEager
      | DependencyType::DynamicImportWeak
      | DependencyType::ImportContext
      | DependencyType::LazyImport
      | DependencyType::ContextElement(ContextTypePrefix::Import)
      | DependencyType::RequireEnsure
      | DependencyType::RequireEnsureItem
      // Resolve/id-only references. They need a target module for resolution or
      // ID generation, but do not evaluate the target module.
      | DependencyType::ImportMetaResolve
      | DependencyType::ImportMetaResolveContext
      | DependencyType::RequireResolve
      | DependencyType::RequireResolveContext
      | DependencyType::IsIncluded
      // HMR accept/decline references are invoked by the hot runtime later, not
      // by normal module initialization.
      | DependencyType::ImportMetaHotAccept
      | DependencyType::ImportMetaHotDecline
      | DependencyType::ModuleHotAccept
      | DependencyType::ModuleHotDecline
      // URL, worker, asset, and CSS resource references do not synchronously
      // execute the referenced module in the current JavaScript module graph.
      | DependencyType::NewUrl
      | DependencyType::NewUrlContext
      | DependencyType::NewWorker
      | DependencyType::CreateScriptUrl
      | DependencyType::CssUrl
      | DependencyType::CssImport
      | DependencyType::CssCompose
      | DependencyType::CssExport
      | DependencyType::CssLocalIdent
      | DependencyType::CssSelfReferenceLocalIdent
      | DependencyType::ExtractCSS
      // Build-time or metadata-only dependencies.
      | DependencyType::ExportInfoApi
      | DependencyType::StaticExports
      | DependencyType::LoaderImport
      | DependencyType::ModuleDecorator
      | DependencyType::RstestModulePath
      | DependencyType::RstestMockModuleId
      | DependencyType::RstestHoistMock
      | DependencyType::RstestDynamicImportOrigin
  )
}

#[plugin_hook(CompilationOptimizeModules for CircularModulesInfoPlugin)]
async fn optimize_modules(
  &self,
  compilation: &Compilation,
  circular_modules: &mut Option<IdentifierSet>,
  _diagnostics: &mut Vec<Diagnostic>,
) -> Result<Option<bool>> {
  let module_graph = compilation.get_module_graph();
  let graph = CycleGraph::build(module_graph);
  *circular_modules = Some(CycleDetector::new(&graph).find_circular_modules());
  Ok(None)
}

impl Plugin for CircularModulesInfoPlugin {
  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx
      .compilation_hooks
      .optimize_modules
      .tap(optimize_modules::new(self));
    Ok(())
  }
}
