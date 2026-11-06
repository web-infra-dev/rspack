use std::{sync::Arc, time::Instant};

use rspack_collections::IdentifierSet;
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
  BuildContext, BuildModuleGraphArtifact, Compilation, DependencyId, ExportsInfoArtifact, Logger,
  ModuleFactoryCreateData, ModuleIdentifier, NormalModule,
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
    let (fresh, fresh_decision) = match result {
      Ok(result) if data.diagnostics.is_empty() => match result.module {
        Some(module) if is_side_effect_only_import(&module, &data.dependencies) => {
          (GroupOutcome::SideEffectOnly, None)
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
              Ok(no_parse) => (
                GroupOutcome::Module(identifier),
                Some(normal.factory_decision(no_parse)),
              ),
              Err(_) => (GroupOutcome::Failed, None),
            },
            None => (GroupOutcome::Module(identifier), None),
          }
        }
        None => (GroupOutcome::Ignored, None),
      },
      _ => (GroupOutcome::Failed, None),
    };
    Ok(vec![Box::new(CompareTask {
      recorded,
      fresh,
      fresh_decision,
    })])
  }
}

async fn matches_no_parse(data: &ModuleFactoryCreateData, module: &NormalModule) -> Result<bool> {
  match &data.options.module.no_parse {
    Some(no_parse) => no_parse.try_match(module.request()).await,
    None => Ok(false),
  }
}

#[derive(Debug)]
struct CompareTask {
  recorded: RecordedGroup,
  fresh: GroupOutcome,
  fresh_decision: Option<NormalFactoryDecision>,
}

#[async_trait::async_trait]
impl Task<VerifyContext> for CompareTask {
  fn get_task_type(&self) -> TaskType {
    TaskType::Main
  }

  async fn main_run(self: Box<Self>, context: &mut VerifyContext) -> TaskResult<VerifyContext> {
    match (&self.recorded.outcome, &self.fresh) {
      (GroupOutcome::Module(recorded), GroupOutcome::Module(fresh)) if recorded == fresh => {
        if self.recorded.decision != self.fresh_decision {
          context.rebuild.insert(*recorded);
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
  while let Some((parent_id, ids)) = queue.pop() {
    let parent = parent_id.and_then(|id| graph.module_by_identifier(&id));
    let inputs = parent
      .map(|parent| build_input_dependencies(parent.as_ref()))
      .unwrap_or_default();
    for id in ids {
      if let Some(target) = graph.module_identifier_by_dependency_id(&id)
        && reachable.insert(*target)
        && let Some(module) = graph.module_graph_module_by_identifier(target)
      {
        queue.push((Some(*target), module.all_dependencies().to_vec()));
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
    // A decision that differs on every factory call can never settle, as with a function `use`
    // that returns a timestamp in its options.
    let VerifyContext {
      rebuild, rebuilt, ..
    } = &mut context;
    rebuild.retain(|id| rebuilt.insert(*id));
    if context.refactorize.is_empty() && context.rebuild.is_empty() {
      if !complete {
        compilation.get_logger("rspack.persistentCache").debug(format!(
          "verified {} relocated factory groups in {:?}, changed {changed_targets} targets and {changed_modules} modules",
          context.verified.len(), start.elapsed()
        ));
        artifact.verify_factory_decisions = Some(context.verified);
      }
      return Ok((artifact, exports));
    }
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
