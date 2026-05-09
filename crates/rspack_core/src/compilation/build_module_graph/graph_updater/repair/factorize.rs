use std::sync::Arc;

use rspack_error::Diagnostic;
use rspack_sources::BoxSource;

use super::{TaskContext, add::AddTask};
use crate::{
  BoxDependency, BoxModule, CompilationId, CompilerId, CompilerOptions, Context, FactorizeInfo,
  ImportPhase, ModuleFactory, ModuleFactoryCreateData, ModuleIdentifier, ModuleLayer, Resolve,
  ResolverFactory,
  compilation::build_module_graph::ForwardedIdSet,
  dependency::DependencyType,
  module_graph::ModuleGraphModule,
  utils::{
    ResourceId,
    task_loop::{Task, TaskResult, TaskType},
  },
};

#[derive(Debug)]
pub struct FactorizeTask {
  pub compiler_id: CompilerId,
  pub compilation_id: CompilationId,
  pub module_factory: Arc<dyn ModuleFactory>,
  pub original_module_identifier: Option<ModuleIdentifier>,
  pub original_module_source: Option<BoxSource>,
  pub original_module_context: Option<Box<Context>>,
  pub issuer: Option<Box<str>>,
  pub issuer_layer: Option<ModuleLayer>,
  pub dependencies: Vec<BoxDependency>,
  pub resolve_options: Option<Arc<Resolve>>,
  pub options: Arc<CompilerOptions>,
  pub resolver_factory: Arc<ResolverFactory>,
  pub from_unlazy: bool,
}

#[async_trait::async_trait]
impl Task<TaskContext> for FactorizeTask {
  fn get_task_type(&self) -> TaskType {
    TaskType::Background
  }
  async fn background_run(mut self: Box<Self>) -> TaskResult<TaskContext> {
    let dependency = &self.dependencies[0];

    let context = if let Some(context) = dependency.get_context()
      && !context.is_empty()
    {
      context
    } else if let Some(context) = &self.original_module_context
      && !context.is_empty()
    {
      context
    } else {
      &self.options.context
    }
    .clone();

    let issuer_layer = dependency
      .get_layer()
      .or(self.issuer_layer.as_ref())
      .cloned();

    let request = self.dependencies[0]
      .as_module_dependency()
      .map(|d| d.request().to_string())
      .or_else(|| {
        self.dependencies[0]
          .as_context_dependency()
          .map(|d| d.request().to_string())
      })
      .unwrap_or_default();
    // Error and result are not mutually exclusive in webpack module factorization.
    // Rspack puts results that need to be shared in both error and ok in [ModuleFactoryCreateData].
    let mut create_data = ModuleFactoryCreateData {
      compiler_id: self.compiler_id,
      compilation_id: self.compilation_id,
      resolve_options: self.resolve_options,
      options: self.options.clone(),
      context,
      request,
      dependencies: self.dependencies,
      issuer: self.issuer,
      issuer_identifier: self.original_module_identifier,
      issuer_layer,
      resolver_factory: self.resolver_factory,

      file_dependencies: Default::default(),
      missing_dependencies: Default::default(),
      context_dependencies: Default::default(),
      diagnostics: Default::default(),
    };
    let factory_result = match self.module_factory.create(&mut create_data).await {
      Ok(result) => Some(result),
      Err(mut e) => {
        // Wrap source code if available
        if let Some(s) = self.original_module_source {
          let has_source_code = e.src.is_some();
          if !has_source_code {
            e.src = Some(s.source().into_string_lossy().into_owned());
          }
        }
        // Bail out if `options.bail` set to `true`,
        // which means 'Fail out on the first error instead of tolerating it.'
        if self.options.bail {
          return Err(e);
        }
        let mut diagnostic = Diagnostic::from(e);
        diagnostic.loc = create_data.dependencies[0].loc();
        create_data.diagnostics.insert(0, diagnostic);
        None
      }
    };

    let factorize_info = FactorizeInfo::new(
      create_data.diagnostics,
      create_data
        .dependencies
        .iter()
        .map(|dep| *dep.id())
        .collect(),
      create_data.file_dependencies,
      create_data.context_dependencies,
      create_data.missing_dependencies,
    );

    let mut dependencies = create_data.dependencies;
    let outcome = match factory_result {
      None => FactorizeOutcome::Failed,
      Some(factory_result) => match factory_result.module {
        None => FactorizeOutcome::Ignored,
        Some(module) => {
          if skip_side_effect_free_esm_import_side_effect_dependencies(&module, &dependencies) {
            for dep in &mut dependencies {
              dep.set_lazy();
            }
            FactorizeOutcome::SideEffectSkipped
          } else {
            let mut mgm = ModuleGraphModule::new(module.identifier());
            mgm.set_issuer_if_unset(self.original_module_identifier);
            FactorizeOutcome::Created {
              module,
              module_graph_module: Box::new(mgm),
            }
          }
        }
      },
    };
    let forwarded_ids = ForwardedIdSet::from_dependencies(&dependencies);

    Ok(vec![Box::new(FactorizeResultTask {
      original_module_identifier: self.original_module_identifier,
      outcome,
      dependencies,
      factorize_info,
      from_unlazy: self.from_unlazy,
      forwarded_ids,
    })])
  }
}

/// Decision made in [`FactorizeTask::background_run`] about how to integrate the factorize result
/// into the module graph. Pre-computing this in the background keeps [`FactorizeResultTask`]'s
/// main-thread work limited to artifact and module graph mutations.
#[derive(Debug)]
pub enum FactorizeOutcome {
  Failed,
  Ignored,
  /// Side-effect-free ESM evaluation-only import; deps already marked lazy in the background.
  SideEffectSkipped,
  Created {
    module: BoxModule,
    module_graph_module: Box<ModuleGraphModule>,
  },
}

impl FactorizeOutcome {
  /// Trace label for the non-`Created` variants. Returns `None` for `Created` since it has its own
  /// trace path that includes the module identifier.
  fn skip_trace_label(&self) -> Option<&'static str> {
    match self {
      Self::Failed => Some("Module created with failure, but without bailout"),
      Self::SideEffectSkipped => Some("Module make-skipped as side-effect-only import"),
      Self::Ignored => Some("Module ignored"),
      Self::Created { .. } => None,
    }
  }
}

#[derive(Debug)]
pub struct FactorizeResultTask {
  pub original_module_identifier: Option<ModuleIdentifier>,
  pub outcome: FactorizeOutcome,
  pub dependencies: Vec<BoxDependency>,
  pub factorize_info: FactorizeInfo,
  pub from_unlazy: bool,
  pub forwarded_ids: ForwardedIdSet,
}

#[async_trait::async_trait]
impl Task<TaskContext> for FactorizeResultTask {
  fn get_task_type(&self) -> TaskType {
    TaskType::Main
  }
  async fn main_run(self: Box<Self>, context: &mut TaskContext) -> TaskResult<TaskContext> {
    let FactorizeResultTask {
      original_module_identifier,
      outcome,
      mut dependencies,
      mut factorize_info,
      from_unlazy,
      forwarded_ids,
    } = *self;

    let first_dep_id = *dependencies[0].id();
    let artifact = &mut context.artifact;
    if !factorize_info.is_success() {
      artifact.make_failed_dependencies.insert(first_dep_id);
    }
    let resource_id = ResourceId::from(first_dep_id);
    artifact
      .file_dependencies
      .add_files(&resource_id, factorize_info.file_dependencies());
    artifact
      .context_dependencies
      .add_files(&resource_id, factorize_info.context_dependencies());
    artifact
      .missing_dependencies
      .add_files(&resource_id, factorize_info.missing_dependencies());

    for dep in &mut dependencies {
      // Some dependencies do not come from the process_dependencies task,
      // so add all dependencies here.
      artifact.affected_dependencies.mark_as_add(dep.id());

      let dep_factorize_info = if let Some(d) = dep.as_context_dependency_mut() {
        d.factorize_info_mut()
      } else if let Some(d) = dep.as_module_dependency_mut() {
        d.factorize_info_mut()
      } else {
        unreachable!("only module dependency and context dependency can factorize")
      };
      // write factorize_info to dependencies[0] and set success factorize_info to others
      *dep_factorize_info = std::mem::take(&mut factorize_info);
    }

    let module_graph = artifact.get_module_graph_mut();
    if let Some(trace_msg) = outcome.skip_trace_label() {
      tracing::trace!("{}: {:?}", trace_msg, &dependencies[0]);
      for dep in dependencies {
        module_graph.add_dependency(dep);
      }
      return Ok(vec![]);
    }

    let FactorizeOutcome::Created {
      module,
      module_graph_module,
    } = outcome
    else {
      unreachable!("non-Created outcomes were handled by the skip_trace_label fast path above")
    };
    tracing::trace!("Module created: {}", module.identifier());
    Ok(vec![Box::new(AddTask {
      original_module_identifier,
      module,
      module_graph_module,
      dependencies,
      from_unlazy,
      forwarded_ids,
    })])
  }
}

fn skip_side_effect_free_esm_import_side_effect_dependencies(
  module: &crate::BoxModule,
  dependencies: &[BoxDependency],
) -> bool {
  module.as_normal_module().is_some()
    && module.factory_meta().and_then(|meta| meta.side_effect_free) == Some(true)
    && dependencies.iter().all(|dep| {
      dep.dependency_type() == &DependencyType::EsmImport
        && dep.get_phase() == ImportPhase::Evaluation
    })
}
