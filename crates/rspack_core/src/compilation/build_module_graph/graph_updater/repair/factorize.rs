use std::sync::Arc;

use rspack_error::Diagnostic;
use rspack_sources::BoxSource;

use super::{super::BuildModuleGraphArtifact, FactorizeDependencies, TaskContext, add::AddTask};
use crate::{
  BoxDependency, CompilationId, CompilerId, CompilerOptions, Context, DependencyId, FactorizeInfo,
  ModuleFactory, ModuleFactoryCreateData, ModuleFactoryResult, ModuleIdentifier, ModuleLayer,
  Resolve, ResolverFactory,
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
  pub dependencies: FactorizeDependencies,
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
  async fn background_run(self: Box<Self>) -> TaskResult<TaskContext> {
    let Self {
      compiler_id,
      compilation_id,
      module_factory,
      original_module_identifier,
      original_module_source,
      original_module_context,
      issuer,
      issuer_layer,
      dependencies,
      resolve_options,
      options,
      resolver_factory,
      from_unlazy,
    } = *self;
    let dependency = dependencies.first_dependency();

    let context = if let Some(context) = dependency.get_context()
      && !context.is_empty()
    {
      context
    } else if let Some(context) = &original_module_context
      && !context.is_empty()
    {
      context
    } else {
      &options.context
    }
    .clone();

    let issuer_layer = dependency.get_layer().or(issuer_layer.as_ref()).cloned();

    let request = dependency
      .as_module_dependency()
      .map(|d| d.request().to_string())
      .or_else(|| {
        dependency
          .as_context_dependency()
          .map(|dependency| dependency.request().to_string())
      })
      .unwrap_or_default();
    let (factory_dependencies, deferred_dependency_ids) = dependencies.into_factory_parts();
    // Error and result are not mutually exclusive in webpack module factorization.
    // Rspack puts results that need to be shared in both error and ok in [ModuleFactoryCreateData].
    let mut create_data = ModuleFactoryCreateData {
      compiler_id,
      compilation_id,
      resolve_options,
      options: options.clone(),
      context,
      request,
      dependencies: factory_dependencies,
      issuer,
      issuer_identifier: original_module_identifier,
      issuer_layer,
      resolver_factory,

      file_dependencies: Default::default(),
      missing_dependencies: Default::default(),
      context_dependencies: Default::default(),
      diagnostics: Default::default(),
    };
    let factory_result = match module_factory.create(&mut create_data).await {
      Ok(result) => Some(result),
      Err(mut e) => {
        // Wrap source code if available
        if let Some(s) = original_module_source {
          let has_source_code = e.src.is_some();
          if !has_source_code {
            e.src = Some(s.source().into_string_lossy().into_owned());
          }
        }
        // Bail out if `options.bail` set to `true`,
        // which means 'Fail out on the first error instead of tolerating it.'
        if options.bail {
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
      deferred_dependency_ids.clone().unwrap_or_else(|| {
        create_data
          .dependencies
          .iter()
          .map(|dependency| *dependency.id())
          .collect()
      }),
      create_data.file_dependencies,
      create_data.context_dependencies,
      create_data.missing_dependencies,
    );
    Ok(vec![Box::new(FactorizeResultTask {
      original_module_identifier,
      factory_result,
      dependencies: FactorizeDependencies::from_factory_parts(
        create_data.dependencies,
        deferred_dependency_ids,
      ),
      factorize_info,
      from_unlazy,
    })])
  }
}

#[derive(Debug)]
pub struct FactorizeResultTask {
  //  pub dependency: DependencyId,
  pub original_module_identifier: Option<ModuleIdentifier>,
  /// Result will be available if [crate::ModuleFactory::create] returns `Ok`.
  pub factory_result: Option<ModuleFactoryResult>,
  pub dependencies: FactorizeDependencies,
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
      original_module_identifier,
      factory_result,
      dependencies,
      mut factorize_info,
      from_unlazy,
    } = *self;

    let primary_dependency_id = dependencies.primary_dependency_id();
    let artifact = &mut context.artifact;
    if !factorize_info.is_success() {
      artifact
        .make_failed_dependencies
        .insert(primary_dependency_id);
    }
    let resource_id = ResourceId::from(primary_dependency_id);
    artifact
      .file_dependencies
      .add_files(&resource_id, factorize_info.file_dependencies());
    artifact
      .context_dependencies
      .add_files(&resource_id, factorize_info.context_dependencies());
    artifact
      .missing_dependencies
      .add_files(&resource_id, factorize_info.missing_dependencies());
    let dependency_ids = sync_factorize_dependencies(artifact, dependencies, &mut factorize_info);
    let Some(factory_result) = factory_result else {
      let dep = artifact
        .module_graph
        .dependency_by_id(&primary_dependency_id);
      tracing::trace!("Module created with failure, but without bailout: {dep:?}");
      return Ok(vec![]);
    };

    let Some(module) = factory_result.module else {
      let dep = artifact
        .module_graph
        .dependency_by_id(&primary_dependency_id);
      tracing::trace!("Module ignored: {dep:?}");
      return Ok(vec![]);
    };
    let module_identifier = module.identifier();
    let mut mgm = ModuleGraphModule::new(module.identifier());
    mgm.set_issuer_if_unset(original_module_identifier);

    tracing::trace!("Module created: {}", &module_identifier);

    Ok(vec![Box::new(AddTask {
      original_module_identifier,
      module,
      module_graph_module: Box::new(mgm),
      dependency_ids,
      from_unlazy,
    })])
  }
}

fn sync_factorize_dependencies(
  artifact: &mut BuildModuleGraphArtifact,
  dependencies: FactorizeDependencies,
  factorize_info: &mut FactorizeInfo,
) -> Vec<DependencyId> {
  match dependencies {
    FactorizeDependencies::Complete(mut dependencies) => {
      let dependency_ids = dependencies
        .iter()
        .map(|dependency| *dependency.id())
        .collect::<Vec<_>>();
      for dependency_id in &dependency_ids {
        artifact.affected_dependencies.mark_as_add(dependency_id);
      }
      for dependency in &mut dependencies {
        // write factorize_info to dependencies[0] and set success factorize_info to others
        *dependency_factorize_info_mut(dependency) = std::mem::take(factorize_info);
      }
      let module_graph = artifact.get_module_graph_mut();
      for dependency in dependencies {
        module_graph.add_dependency(dependency);
      }
      dependency_ids
    }
    FactorizeDependencies::Deferred {
      mut first_dependency,
      dependency_ids,
    } => {
      for dependency_id in &dependency_ids {
        artifact.affected_dependencies.mark_as_add(dependency_id);
      }
      *dependency_factorize_info_mut(&mut first_dependency) = std::mem::take(factorize_info);
      let module_graph = artifact.get_module_graph_mut();
      module_graph.add_dependency(first_dependency);
      for dependency_id in dependency_ids.iter().skip(1) {
        *dependency_factorize_info_mut(module_graph.dependency_by_id_mut(dependency_id)) =
          std::mem::take(factorize_info);
      }
      dependency_ids
    }
  }
}

fn dependency_factorize_info_mut(dependency: &mut BoxDependency) -> &mut FactorizeInfo {
  if dependency.as_context_dependency().is_some() {
    dependency
      .as_context_dependency_mut()
      .expect("context dependency expected")
      .factorize_info_mut()
  } else if dependency.as_module_dependency().is_some() {
    dependency
      .as_module_dependency_mut()
      .expect("module dependency expected")
      .factorize_info_mut()
  } else {
    unreachable!("only module dependency and context dependency can factorize")
  }
}
