use std::sync::Arc;

use rspack_error::Diagnostic;
use rspack_sources::BoxSource;

use super::{TaskContext, add::AddTask};
use crate::{
  BuildContext, Context, DependencyRef, FactorizeInfo, ImportPhase, ModuleFactory,
  ModuleFactoryCreateData, ModuleFactoryResult, ModuleIdentifier, ModuleLayer, Resolve,
  dependency::DependencyType,
  module_graph::ModuleGraphModule,
  utils::task_loop::{Task, TaskResult, TaskType},
};

#[derive(Debug)]
pub struct FactorizeTask {
  pub build_context: Arc<BuildContext>,
  pub module_factory: Arc<dyn ModuleFactory>,
  pub original_module_identifier: Option<ModuleIdentifier>,
  pub original_module_source: Option<BoxSource>,
  pub original_module_context: Option<Box<Context>>,
  pub issuer: Option<Box<str>>,
  pub issuer_layer: Option<ModuleLayer>,
  pub dependencies: Vec<DependencyRef>,
  pub resolve_options: Option<Arc<Resolve>>,
  pub from_unlazy: bool,
}

#[async_trait::async_trait]
impl Task<TaskContext> for FactorizeTask {
  fn get_task_type(&self) -> TaskType {
    TaskType::Background
  }
  async fn background_run(mut self: Box<Self>) -> TaskResult<TaskContext> {
    // Error and result are not mutually exclusive in webpack module factorization.
    // Rspack puts results that need to be shared in both error and ok in [ModuleFactoryCreateData].
    let mut create_data = ModuleFactoryCreateData::new(
      self.build_context,
      self.resolve_options,
      self.original_module_context.as_deref(),
      self.dependencies,
      self.issuer,
      self.original_module_identifier,
      self.issuer_layer,
    );
    let factory_result = match self.module_factory.create(&mut create_data).await {
      Ok(result) => Some(result),
      Err(mut e) => {
        // Wrap source code if available
        if let Some(s) = self.original_module_source {
          let has_source_code = e.src.is_some();
          if !has_source_code {
            e.src = Some(s.source().into_string_lossy().into_owned().into());
          }
        }
        // Bail out if `options.bail` set to `true`,
        // which means 'Fail out on the first error instead of tolerating it.'
        if create_data.build_context.compiler_options.bail {
          return Err(e);
        }
        let mut diagnostic = Diagnostic::from(e);
        diagnostic.loc = create_data
          .dependencies
          .iter()
          .filter_map(|dependency| dependency.loc())
          .min();
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
    Ok(vec![Box::new(FactorizeResultTask {
      build_context: create_data.build_context,
      original_module_identifier: self.original_module_identifier,
      factory_result,
      dependencies: create_data.dependencies,
      factorize_info,
      from_unlazy: self.from_unlazy,
    })])
  }
}

#[derive(Debug)]
pub struct FactorizeResultTask {
  pub build_context: Arc<BuildContext>,
  //  pub dependency: DependencyId,
  pub original_module_identifier: Option<ModuleIdentifier>,
  /// Result will be available if [crate::ModuleFactory::create] returns `Ok`.
  pub factory_result: Option<ModuleFactoryResult>,
  pub dependencies: Vec<DependencyRef>,
  pub factorize_info: FactorizeInfo,
  pub from_unlazy: bool,
}

#[async_trait::async_trait]
impl Task<TaskContext> for FactorizeResultTask {
  fn get_task_type(&self) -> TaskType {
    TaskType::Main
  }
  async fn main_run(self: Box<Self>, context: &mut TaskContext) -> TaskResult<TaskContext> {
    let FactorizeResultTask {
      build_context,
      original_module_identifier,
      factory_result,
      dependencies,
      factorize_info,
      from_unlazy,
    } = *self;

    let artifact = &mut context.artifact;
    artifact.record_factorization(factorize_info);

    for dep in &dependencies {
      // Some dependencies do not come from the process_dependencies task,
      // so add all dependencies here.
      artifact.affected_dependencies.mark_as_add(dep.id());
    }

    let module_graph = artifact.get_module_graph_mut();
    let Some(factory_result) = factory_result else {
      let dep = &dependencies[0];
      tracing::trace!("Module created with failure, but without bailout: {dep:?}");
      // sync dependencies to mg
      for dep in dependencies {
        module_graph.add_dependency_ref(dep)
      }
      return Ok(vec![]);
    };

    if let Some(module) = factory_result.module.as_ref()
      && skip_side_effect_free_esm_import_side_effect_dependencies(module, &dependencies)
    {
      let dep = &dependencies[0];
      tracing::trace!("Module make-skipped as side-effect-only import: {dep:?}");
      for dep in &dependencies {
        dep.set_lazy();
      }
      for dep in dependencies {
        module_graph.add_dependency_ref(dep)
      }
      return Ok(vec![]);
    }

    let Some(module) = factory_result.module else {
      let dep = &dependencies[0];
      tracing::trace!("Module ignored: {dep:?}");
      // sync dependencies to mg
      for dep in dependencies {
        module_graph.add_dependency_ref(dep)
      }
      return Ok(vec![]);
    };
    let module_identifier = module.identifier();
    let mut mgm = ModuleGraphModule::new(module.identifier());
    mgm.set_issuer_if_unset(original_module_identifier);

    tracing::trace!("Module created: {}", &module_identifier);

    Ok(vec![Box::new(AddTask {
      build_context,
      original_module_identifier,
      module,
      module_graph_module: Box::new(mgm),
      dependencies,
      from_unlazy,
    })])
  }
}

fn skip_side_effect_free_esm_import_side_effect_dependencies(
  module: &crate::BoxModule,
  dependencies: &[DependencyRef],
) -> bool {
  module.as_normal_module().is_some()
    && module.factory_meta().and_then(|meta| meta.side_effect_free) == Some(true)
    && dependencies.iter().all(|dep| {
      dep.dependency_type() == &DependencyType::EsmImport
        && dep.get_phase() == ImportPhase::Evaluation
    })
}
