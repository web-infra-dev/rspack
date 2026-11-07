use std::{sync::Arc, time::Instant};

use rspack_collections::{IdentifierMap, IdentifierSet};
use rspack_error::Result;
use rustc_hash::FxHashSet;

use super::{
  UpdateParam,
  repair::{
    context::TaskContext,
    factorize::{FactorizeInput, is_side_effect_only_import},
  },
  update_module_graph,
};
use crate::{
  BoxModule, BuildContext, BuildModuleGraphArtifact, Compilation, DependencyId,
  ExportsInfoArtifact, FactorizeInfo, FactoryChoice, Logger, ModuleFactoryCreateData,
  ModuleIdentifier, NormalModule,
  module_graph::{build_input_dependencies, can_refactorize_group},
  normal_module::NormalFactoryDecision,
  utils::task_loop::{Task, TaskResult, TaskType, run_task_loop},
};

#[derive(Debug, PartialEq, Eq)]
enum GroupOutcome {
  Module(ModuleIdentifier),
  SideEffectOnly,
  Ignored,
  Failed,
}

#[derive(Default)]
struct VerifyContext {
  verified: FxHashSet<DependencyId>,
  repaired: FxHashSet<DependencyId>,
  refactorize: FxHashSet<DependencyId>,
  rebuild: IdentifierSet,
  rebuilt: IdentifierSet,
  /// The fresh modules of the groups whose decision differs from their target's build, per target.
  /// Kept until settling ends, because a repair can leave the target without its producer.
  disagreements: IdentifierMap<Vec<FactoryChoice>>,
  /// The target and group, in the form of `ModuleGraphModule::producer`, of every group reachable
  /// from the entries in the current graph.
  reaching: FxHashSet<(ModuleIdentifier, Option<DependencyId>)>,
}

#[derive(Debug)]
struct RecordedGroup {
  parent: Option<ModuleIdentifier>,
  dependencies: Vec<DependencyId>,
  can_refactorize: bool,
  outcome: GroupOutcome,
  decision: Option<NormalFactoryDecision>,
}

#[derive(Debug)]
struct VerifyTask {
  input: FactorizeInput,
  recorded: RecordedGroup,
}

#[async_trait::async_trait]
impl Task<VerifyContext> for VerifyTask {
  fn get_task_type(&self) -> TaskType {
    TaskType::Background
  }

  async fn background_run(self: Box<Self>) -> TaskResult<VerifyContext> {
    let Self {
      input,
      mut recorded,
    } = *self;
    let (data, result) = input.run().await;
    let (fresh, fresh_decision, fresh_module) = match result {
      Ok(result) if data.diagnostics.is_empty() => match result.module {
        Some(module) if is_side_effect_only_import(&module, &data.dependencies) => {
          (GroupOutcome::SideEffectOnly, None, None)
        }
        Some(module) => {
          let identifier = if module.as_self_module().is_some() {
            recorded.decision = None;
            recorded.parent.expect("self module should have a parent")
          } else {
            module.identifier()
          };
          match module.as_normal_module() {
            Some(normal) => match matches_no_parse(&data, normal).await {
              Ok(no_parse) => {
                let decision = normal.factory_decision(no_parse);
                (
                  GroupOutcome::Module(identifier),
                  Some(decision),
                  Some(module),
                )
              }
              Err(_) => (GroupOutcome::Failed, None, None),
            },
            None => (GroupOutcome::Module(identifier), None, Some(module)),
          }
        }
        None => (GroupOutcome::Ignored, None, None),
      },
      _ => (GroupOutcome::Failed, None, None),
    };
    Ok(vec![Box::new(CompareTask {
      recorded,
      fresh,
      fresh_decision,
      fresh_module,
    })])
  }
}

async fn matches_no_parse(data: &ModuleFactoryCreateData, module: &NormalModule) -> Result<bool> {
  match &data.build_context.compiler_options.module.no_parse {
    Some(no_parse) => no_parse.try_match(module.request()).await,
    None => Ok(false),
  }
}

#[derive(Debug)]
struct CompareTask {
  recorded: RecordedGroup,
  fresh: GroupOutcome,
  fresh_decision: Option<NormalFactoryDecision>,
  fresh_module: Option<BoxModule>,
}

#[async_trait::async_trait]
impl Task<VerifyContext> for CompareTask {
  fn get_task_type(&self) -> TaskType {
    TaskType::Main
  }

  async fn main_run(self: Box<Self>, context: &mut VerifyContext) -> TaskResult<VerifyContext> {
    match (&self.recorded.outcome, &self.fresh) {
      (GroupOutcome::Module(recorded), GroupOutcome::Module(fresh)) if recorded == fresh => {
        if !NormalFactoryDecision::agree(&self.recorded.decision, &self.fresh_decision)
          && let Some(module) = self.fresh_module
        {
          context
            .disagreements
            .entry(*recorded)
            .or_default()
            .push(FactoryChoice {
              module,
              parent: self.recorded.parent,
              producer: self.recorded.parent.map(|_| self.recorded.dependencies[0]),
            });
        }
      }
      (GroupOutcome::Ignored, GroupOutcome::Ignored)
      | (GroupOutcome::SideEffectOnly, GroupOutcome::SideEffectOnly) => {}
      _ => {
        if self.recorded.can_refactorize {
          context.refactorize.extend(self.recorded.dependencies);
        } else if let Some(parent) = self.recorded.parent {
          context.rebuild.insert(parent);
        }
      }
    }
    Ok(vec![])
  }
}

/// Schedules the rebuilds that the recorded disagreements call for.
///
/// The group that produced a build decides whether it is stale, so a disagreement of any other
/// group keeps the module, and the producer's fresh module is the one to rebuild. A producer that
/// no longer reaches the module cannot decide, so any remaining disagreement rebuilds it.
///
/// A rebuild replaces the groups of a module, so a group whose parent is being rebuilt cannot
/// decide yet. Its target waits for the next iteration, when the fresh groups have reported.
fn rebuild_disagreeing_modules(
  artifact: &mut BuildModuleGraphArtifact,
  context: &mut VerifyContext,
) {
  let VerifyContext {
    refactorize,
    disagreements,
    reaching,
    rebuild,
    rebuilt,
    ..
  } = context;
  let BuildModuleGraphArtifact {
    module_graph,
    factorization_artifact,
    factory_choices,
    ..
  } = artifact;
  // A decision that differs on every factory call can never settle, as with a function `use`
  // that returns a timestamp in its options.
  rebuild.retain(|id| !rebuilt.contains(id));
  let mut deciding = IdentifierMap::default();
  disagreements.retain(|target, groups| {
    if rebuilt.contains(target) {
      return false;
    }
    let Some(mgm) = module_graph.module_graph_module_by_identifier(target) else {
      return false;
    };
    // Refactorizing a group can reorder its dependencies, which changes its owner.
    let producer = mgm.producer().map(|id| {
      factorization_artifact
        .get(&id)
        .map_or(id, FactorizeInfo::owner_dep_id)
    });
    let group = if reaching.contains(&(*target, producer)) {
      groups.iter().position(|group| group.producer == producer)
    } else {
      groups
        .iter()
        .position(|group| reaching.contains(&(*target, group.producer)))
    };
    if let Some(group) = group {
      deciding.insert(*target, group);
    }
    true
  });
  let rebuilding = rebuild
    .iter()
    .chain(deciding.keys())
    .copied()
    .collect::<IdentifierSet>();
  let (mut ready, waiting): (Vec<_>, Vec<_>) = deciding.into_iter().partition(|(target, group)| {
    let choice = &disagreements[target][*group];
    choice
      .parent
      .is_none_or(|parent| !rebuilding.contains(&parent))
      && choice.producer.is_none_or(|id| !refactorize.contains(&id))
  });
  // The targets of an import cycle can wait for each other, and then no rebuild ever comes. One has
  // to decide first, and the fresh groups of its rebuild report to the others.
  if ready.is_empty() && rebuild.is_empty() && refactorize.is_empty() {
    ready.extend(waiting.into_iter().min_by_key(|(target, _)| *target));
  }
  for (target, group) in ready {
    let mut groups = disagreements
      .remove(&target)
      .expect("should have disagreements of a deciding target");
    factory_choices.insert(target, groups.swap_remove(group));
    rebuild.insert(target);
  }
  rebuilt.extend(rebuild.iter().copied());
}

fn unverified_groups(
  compilation: &Compilation,
  artifact: &BuildModuleGraphArtifact,
  build_context: &Arc<BuildContext>,
  context: &mut VerifyContext,
) -> Vec<Box<dyn Task<VerifyContext>>> {
  let graph = artifact.get_module_graph();
  let mut tasks: Vec<Box<dyn Task<VerifyContext>>> = vec![];
  let mut queue = vec![(
    None,
    super::super::entry_dependencies(compilation)
      .into_iter()
      .collect::<Vec<_>>(),
  )];
  let mut reachable = IdentifierSet::default();
  context.reaching.clear();
  while let Some((parent_id, ids)) = queue.pop() {
    let parent = parent_id.and_then(|id| graph.module_by_identifier(&id));
    let inputs = parent
      .map(|parent| build_input_dependencies(parent.as_ref()))
      .unwrap_or_default();
    for id in ids {
      if let Some(target) = graph.module_identifier_by_dependency_id(&id) {
        let owner = artifact
          .factorize_info(&id)
          .map_or(id, FactorizeInfo::owner_dep_id);
        context.reaching.insert((*target, parent_id.map(|_| owner)));
        if reachable.insert(*target)
          && let Some(module) = graph.module_graph_module_by_identifier(target)
        {
          queue.push((Some(*target), module.all_dependencies().to_vec()));
        }
      }
      let Some(info) = artifact.factorize_info(&id) else {
        continue;
      };
      if !info.is_success() || !context.verified.insert(info.owner_dep_id()) {
        continue;
      }
      let dependencies = info
        .related_dep_ids()
        .iter()
        .map(|id| {
          inputs
            .get(id)
            .copied()
            .unwrap_or_else(|| graph.dependency_ref_by_id(id))
            .clone()
        })
        .collect::<Vec<_>>();
      let Some(factory) = compilation
        .dependency_factories
        .get(dependencies[0].dependency_type())
      else {
        continue;
      };
      let target = graph.get_module_by_dependency_id(&id);
      let recorded = if let Some(target) = target {
        GroupOutcome::Module(target.identifier())
      } else if dependencies.iter().all(|dep| dep.lazy().is_some()) {
        GroupOutcome::SideEffectOnly
      } else {
        GroupOutcome::Ignored
      };
      let can_refactorize = parent.is_none_or(|parent| {
        can_refactorize_group(
          parent.as_ref(),
          artifact
            .module_to_lazy_make
            .get_lazy_dependencies(&parent.identifier()),
          dependencies
            .iter()
            .map(|dep| (*dep.id(), dep, inputs.contains_key(dep.id()))),
        )
      });
      tasks.push(Box::new(VerifyTask {
        input: FactorizeInput::new(
          build_context.clone(),
          factory.clone(),
          parent.map(|parent| parent.as_ref()),
          dependencies,
        ),
        recorded: RecordedGroup {
          parent: parent_id,
          dependencies: info.related_dep_ids().to_vec(),
          can_refactorize,
          outcome: recorded,
          decision: target
            .and_then(|module| module.as_normal_module())
            .map(|module| module.factory_decision(!module.parsed())),
        },
      }));
    }
  }
  tasks
}

pub(in super::super) async fn settle_factory_decisions(
  compilation: &Compilation,
  mut artifact: BuildModuleGraphArtifact,
  mut exports: ExportsInfoArtifact,
  complete: bool,
) -> Result<(BuildModuleGraphArtifact, ExportsInfoArtifact)> {
  let Some(verified) = artifact.verify_factory_decisions.take() else {
    return Ok((artifact, exports));
  };
  let start = Instant::now();
  let mut context = VerifyContext {
    verified,
    ..Default::default()
  };
  let build_context = TaskContext::build_context(compilation);
  let mut changed_targets = 0;
  let mut changed_modules = 0;
  loop {
    let tasks = unverified_groups(compilation, &artifact, &build_context, &mut context);
    run_task_loop(&mut context, tasks).await?;
    rebuild_disagreeing_modules(&mut artifact, &mut context);
    if context.refactorize.is_empty() && context.rebuild.is_empty() {
      if !complete {
        compilation.get_logger("rspack.persistentCache").debug(format!(
          "verified {} relocated factory groups in {:?}, changed {changed_targets} targets and {changed_modules} modules",
          context.verified.len(), start.elapsed()
        ));
        artifact.verify_factory_decisions = Some(context.verified);
      }
      artifact.factory_choices.clear();
      return Ok((artifact, exports));
    }
    // A rebuild replaces the groups of a module, and the new ones report their own decisions.
    let VerifyContext {
      rebuild,
      disagreements,
      ..
    } = &mut context;
    let replaced =
      |parent: Option<ModuleIdentifier>| parent.is_some_and(|parent| rebuild.contains(&parent));
    for groups in disagreements.values_mut() {
      groups.retain(|group| !replaced(group.parent));
    }
    artifact
      .factory_choices
      .retain(|_, choice| !replaced(choice.parent));
    for id in &context.refactorize {
      if let Some(info) = artifact.factorize_info(id)
        && context.repaired.insert(info.owner_dep_id())
      {
        context.verified.remove(&info.owner_dep_id());
      }
    }
    changed_targets += context.refactorize.len();
    changed_modules += context.rebuild.len();
    artifact.refactorize_dependencies = std::mem::take(&mut context.refactorize);
    artifact.check_issuer_cycles = true;
    (artifact, exports) = update_module_graph(
      compilation,
      artifact,
      exports,
      vec![
        UpdateParam::ForceBuildModules(std::mem::take(&mut context.rebuild)),
        UpdateParam::BuildEntryAndClean(super::super::entry_dependencies(compilation)),
      ],
    )
    .await?;
  }
}
