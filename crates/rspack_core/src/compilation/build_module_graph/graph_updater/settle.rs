use std::{sync::Arc, time::Instant};

use rspack_collections::{IdentifierMap, IdentifierSet};
use rspack_error::Result;
use rustc_hash::FxHashSet;

use super::{
  UpdateParam,
  cutout::repair_issuer_cycles,
  repair::{
    context::TaskContext,
    factorize::{FactorizeInput, is_side_effect_only_import},
    process_dependencies::producer_group,
  },
  update_module_graph,
};
use crate::{
  BoxModule, BuildContext, BuildModuleGraphArtifact, Compilation, DependencyId,
  ExportsInfoArtifact, FactorizeInfo, FactoryChoice, Logger, Module, ModuleFactoryCreateData,
  ModuleIdentifier, NormalModule, ProducerGroup,
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
  /// The groups of `refactorize`.
  refactorizing: FxHashSet<ProducerGroup>,
  rebuild: IdentifierSet,
  rebuilt: IdentifierSet,
  /// The fresh modules of the groups whose decision differs from their target's build, per target.
  /// Kept until settling ends, because a repair can leave the target without its producer.
  disagreements: IdentifierMap<Vec<FactoryChoice>>,
  /// The target and group, in the form of `ModuleGraphModule::producer`, of every group reachable
  /// from the entries in the current graph.
  reaching: FxHashSet<(ModuleIdentifier, Option<ProducerGroup>)>,
  /// The modules reachable from the entries in the current graph.
  reachable: IdentifierSet,
  /// The targets of `disagreements` per parent of a disagreeing group.
  reporters: IdentifierMap<IdentifierSet>,
  /// The targets to decide again, because their groups, producer or reaching groups changed.
  pending: IdentifierSet,
  /// The targets with a deciding group, and the producer of that group.
  deciding: IdentifierMap<Option<ProducerGroup>>,
  /// The deciding targets that wait for the rebuild of a parent of their deciding group, per parent.
  waiters: IdentifierMap<IdentifierSet>,
  /// The deciding targets that wait for their deciding group to be refactorized.
  refactor_waiters: Vec<ModuleIdentifier>,
  /// The modules rebuilt by the last update, whose waiters can decide now.
  released: Vec<ModuleIdentifier>,
}

type GroupTarget = (ModuleIdentifier, Option<ProducerGroup>);

/// What a factory produced for a module, which a fresh factory result must reproduce for the
/// module to be kept.
#[derive(Debug)]
enum FactoryDecision {
  Normal(NormalFactoryDecision),
  /// The serialized factory outputs of a context or external module.
  Other(Vec<u8>),
}

/// The decision of a module that isn't a normal module, or `None` when nothing decides it, as for
/// a context module whose dependencies a plugin lists.
fn other_factory_decision(module: &dyn Module) -> Option<Vec<u8>> {
  match module.as_context_module() {
    Some(context) => context.factory_decision(),
    None => module.as_external_module()?.factory_decision(),
  }
}

#[derive(Debug)]
struct RecordedGroup {
  parent: Option<ModuleIdentifier>,
  producer: Option<ProducerGroup>,
  dependencies: Vec<DependencyId>,
  can_refactorize: bool,
  outcome: GroupOutcome,
  decision: Option<FactoryDecision>,
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
    let Self { input, recorded } = *self;
    let (data, result) = input.run().await;
    let (fresh, agrees, fresh_module) = match result {
      Ok(result) if data.diagnostics.is_empty() => match result.module {
        Some(module) if is_side_effect_only_import(&module, &data.dependencies) => {
          (GroupOutcome::SideEffectOnly, true, None)
        }
        // A self reference is its parent, whose own group decides it.
        Some(module) if module.as_self_module().is_some() => (
          GroupOutcome::Module(recorded.parent.expect("self module should have a parent")),
          true,
          None,
        ),
        Some(module) => match module.as_normal_module() {
          Some(normal) => match matches_no_parse(&data, normal).await {
            Ok(no_parse) => {
              let fresh = normal.factory_decision(no_parse);
              let agrees = matches!(&recorded.decision,
                Some(FactoryDecision::Normal(recorded)) if recorded.agrees_with(&fresh));
              (
                GroupOutcome::Module(module.identifier()),
                agrees,
                Some(module),
              )
            }
            Err(_) => (GroupOutcome::Failed, false, None),
          },
          // A module without a decision takes the fresh module.
          None => {
            let fresh = other_factory_decision(module.as_ref());
            let agrees = matches!((&recorded.decision, fresh),
              (Some(FactoryDecision::Other(recorded)), Some(fresh)) if *recorded == fresh);
            (
              GroupOutcome::Module(module.identifier()),
              agrees,
              Some(module),
            )
          }
        },
        None => (GroupOutcome::Ignored, true, None),
      },
      _ => (GroupOutcome::Failed, false, None),
    };
    Ok(vec![Box::new(CompareTask {
      recorded,
      fresh,
      agrees,
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
  /// Whether the fresh module agrees with the recorded decision of the target.
  agrees: bool,
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
        if !self.agrees
          && let Some(module) = self.fresh_module
        {
          context
            .disagreements
            .entry(*recorded)
            .or_default()
            .push(FactoryChoice {
              module,
              producer: self.recorded.producer,
            });
          if let Some(parent) = self.recorded.parent {
            context
              .reporters
              .entry(parent)
              .or_default()
              .insert(*recorded);
          }
          context.pending.insert(*recorded);
        }
      }
      (GroupOutcome::Ignored, GroupOutcome::Ignored)
      | (GroupOutcome::SideEffectOnly, GroupOutcome::SideEffectOnly) => {}
      _ => {
        if self.recorded.can_refactorize {
          context.refactorize.extend(self.recorded.dependencies);
          context.refactorizing.extend(self.recorded.producer);
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
///
/// Only the targets whose groups changed are decided again, and only the waiters of a parent that
/// stopped rebuilding are checked again, so a chain of rebuilds costs what it changes.
fn rebuild_disagreeing_modules(
  artifact: &mut BuildModuleGraphArtifact,
  context: &mut VerifyContext,
) {
  let VerifyContext {
    refactorize,
    refactorizing,
    disagreements,
    reaching,
    rebuild,
    rebuilt,
    pending,
    deciding,
    waiters,
    refactor_waiters,
    released,
    ..
  } = context;
  let BuildModuleGraphArtifact {
    module_graph,
    factory_choices,
    ..
  } = artifact;
  // A decision that differs on every factory call can never settle, as with a function `use`
  // that returns a timestamp in its options.
  rebuild.retain(|id| !rebuilt.contains(id));
  let mut checking = std::mem::take(refactor_waiters);
  for parent in released.drain(..) {
    pending.insert(parent);
    checking.extend(waiters.remove(&parent).into_iter().flatten());
  }
  for target in pending.drain() {
    let mgm = module_graph
      .module_graph_module_by_identifier(&target)
      .filter(|_| !rebuilt.contains(&target));
    let Some(mgm) = mgm else {
      disagreements.remove(&target);
      if deciding.remove(&target).is_some() {
        checking.extend(waiters.remove(&target).into_iter().flatten());
      }
      continue;
    };
    let producer = mgm.producer();
    let group = disagreements.get(&target).and_then(|groups| {
      if reaching.contains(&(target, producer)) {
        groups.iter().find(|group| group.producer == producer)
      } else {
        groups
          .iter()
          .find(|group| reaching.contains(&(target, group.producer)))
      }
    });
    match group {
      Some(group) => {
        deciding.insert(target, group.producer);
        checking.push(target);
      }
      // A target that stops deciding stops blocking its waiters.
      None => {
        if deciding.remove(&target).is_some() {
          checking.extend(waiters.remove(&target).into_iter().flatten());
        }
      }
    }
  }
  let mut ready = vec![];
  for target in checking {
    let Some(producer) = deciding.get(&target) else {
      continue;
    };
    match producer {
      Some(producer)
        if rebuild.contains(&producer.parent) || deciding.contains_key(&producer.parent) =>
      {
        waiters.entry(producer.parent).or_default().insert(target);
      }
      Some(producer) if refactorizing.contains(producer) => refactor_waiters.push(target),
      _ => ready.push(target),
    }
  }
  ready.sort_unstable();
  ready.dedup();
  // The targets of an import cycle can wait for each other, and then no rebuild ever comes. One has
  // to decide first, and the fresh groups of its rebuild report to the others.
  if ready.is_empty() && rebuild.is_empty() && refactorize.is_empty() {
    ready.extend(deciding.keys().min().copied());
  }
  for target in ready {
    let producer = deciding
      .remove(&target)
      .expect("should have the deciding group of a ready target");
    let mut groups = disagreements
      .remove(&target)
      .expect("should have disagreements of a deciding target");
    let group = groups
      .iter()
      .position(|group| group.producer == producer)
      .expect("should have the deciding group");
    factory_choices.insert(target, groups.swap_remove(group));
    rebuild.insert(target);
  }
  rebuilt.extend(rebuild.iter().copied());
}

/// A factory can replace the first dependency of a group, and the producer holds the result.
fn group_producer(
  artifact: &BuildModuleGraphArtifact,
  parent: Option<ModuleIdentifier>,
  id: DependencyId,
) -> Option<ProducerGroup> {
  parent.and_then(|parent| {
    let owner = artifact
      .factorize_info(&id)
      .map_or(id, FactorizeInfo::owner_dep_id);
    producer_group(parent, artifact.get_module_graph().dependency_by_id(&owner))
  })
}

/// The targets and groups of the dependencies of `parent`, or `None` when it left the graph.
fn group_targets(
  artifact: &BuildModuleGraphArtifact,
  parent: ModuleIdentifier,
) -> Option<FxHashSet<GroupTarget>> {
  let graph = artifact.get_module_graph();
  let module = graph.module_graph_module_by_identifier(&parent)?;
  Some(
    module
      .all_dependencies()
      .iter()
      .filter_map(|id| {
        let target = graph.module_identifier_by_dependency_id(id)?;
        Some((*target, group_producer(artifact, Some(parent), *id)))
      })
      .collect(),
  )
}

/// The groups to verify, found from the entries, or from `changed` modules when the graph only
/// gained groups since the last walk.
fn unverified_groups(
  compilation: &Compilation,
  artifact: &BuildModuleGraphArtifact,
  build_context: &Arc<BuildContext>,
  context: &mut VerifyContext,
  changed: Option<Vec<ModuleIdentifier>>,
) -> Vec<Box<dyn Task<VerifyContext>>> {
  let graph = artifact.get_module_graph();
  let mut tasks: Vec<Box<dyn Task<VerifyContext>>> = vec![];
  let mut queue = match changed {
    Some(changed) => changed
      .into_iter()
      .filter_map(|id| {
        let module = graph.module_graph_module_by_identifier(&id)?;
        Some((Some(id), module.all_dependencies().to_vec()))
      })
      .collect(),
    None => {
      context.reachable.clear();
      context.reaching.clear();
      vec![(
        None,
        super::super::entry_dependencies(compilation)
          .into_iter()
          .collect::<Vec<_>>(),
      )]
    }
  };
  while let Some((parent_id, ids)) = queue.pop() {
    let parent = parent_id.and_then(|id| graph.module_by_identifier(&id));
    let inputs = parent
      .map(|parent| build_input_dependencies(parent.as_ref()))
      .unwrap_or_default();
    for id in ids {
      let producer = group_producer(artifact, parent_id, id);
      if let Some(target) = graph.module_identifier_by_dependency_id(&id) {
        context.reaching.insert((*target, producer));
        if context.reachable.insert(*target)
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
          producer,
          dependencies: info.related_dep_ids().to_vec(),
          can_refactorize,
          outcome: recorded,
          decision: target.and_then(|module| match module.as_normal_module() {
            Some(normal) => Some(FactoryDecision::Normal(
              normal.factory_decision(!normal.parsed()),
            )),
            None => other_factory_decision(module.as_ref()).map(FactoryDecision::Other),
          }),
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
  let mut changed = None;
  loop {
    let tasks = unverified_groups(
      compilation,
      &artifact,
      &build_context,
      &mut context,
      changed.take(),
    );
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
      reporters,
      pending,
      released,
      ..
    } = &mut context;
    for parent in rebuild.iter() {
      for target in reporters.remove(parent).into_iter().flatten() {
        if let Some(groups) = disagreements.get_mut(&target) {
          groups.retain(|choice| choice.producer.is_none_or(|group| group.parent != *parent));
          pending.insert(target);
        }
      }
    }
    released.extend(rebuild.iter().copied());
    artifact.factory_choices.retain(|_, choice| {
      choice
        .producer
        .is_none_or(|group| !rebuild.contains(&group.parent))
    });
    for id in &context.refactorize {
      if let Some(info) = artifact.factorize_info(id)
        && context.repaired.insert(info.owner_dep_id())
      {
        context.verified.remove(&info.owner_dep_id());
      }
    }
    // Only the rebuilt modules and the parents of refactorized groups change their groups. When
    // they keep every group they had, nothing became unreachable, and their new groups are the
    // only ones left to walk. Otherwise the next walk starts over from the entries.
    let graph = artifact.get_module_graph();
    let parents = context
      .refactorize
      .iter()
      .map(|id| graph.get_parent_module(id).copied())
      .chain(context.rebuild.iter().copied().map(Some))
      .collect::<Option<IdentifierSet>>();
    let before = parents.map(|parents| {
      parents
        .into_iter()
        .map(|parent| (parent, group_targets(&artifact, parent)))
        .collect::<Vec<_>>()
    });
    changed_targets += context.refactorize.len();
    changed_modules += context.rebuild.len();
    artifact.refactorize_dependencies = std::mem::take(&mut context.refactorize);
    context.refactorizing.clear();
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
    // The targets whose groups changed decide again.
    let mut added = false;
    changed = before.and_then(|before| {
      let mut changed = Vec::with_capacity(before.len());
      for (parent, before) in before {
        let after = group_targets(&artifact, parent)?;
        for (target, _) in before.iter().flatten().chain(&after) {
          if context.disagreements.contains_key(target) {
            context.pending.insert(*target);
          }
        }
        added |= after
          .iter()
          .any(|(target, _)| !context.reachable.contains(target));
        if !before.is_none_or(|before| before.is_subset(&after)) {
          return None;
        }
        changed.push(parent);
      }
      Some(changed)
    });
    if changed.is_none() {
      context
        .pending
        .extend(context.disagreements.keys().copied());
    }
    // A rebuild restores the issuer of its module, so only a module added to the graph, such as one
    // that cache recovery dropped, can close an issuer cycle.
    if added || changed.is_none() {
      repair_issuer_cycles(&mut artifact);
    }
  }
}
