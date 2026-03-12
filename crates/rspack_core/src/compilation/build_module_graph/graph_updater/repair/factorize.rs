use std::sync::Arc;

use rspack_error::Diagnostic;
use rspack_sources::BoxSource;

use super::{TaskContext, add::AddTask};
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
pub struct ModuleDependencies {
  dependencies: Vec<BoxDependency>,
  dependency_ids: Vec<DependencyId>,
}

impl ModuleDependencies {
  pub fn new(dependencies: Vec<BoxDependency>) -> Self {
    Self {
      dependency_ids: dependencies
        .iter()
        .map(|dependency| *dependency.id())
        .collect(),
      dependencies,
    }
  }

  pub fn from_dependency(dependency: BoxDependency) -> Self {
    Self::new(vec![dependency])
  }

  pub fn primary_dependency(&self) -> &BoxDependency {
    self
      .dependencies
      .first()
      .expect("should have at least one dependency")
  }

  pub fn primary_id(&self) -> DependencyId {
    *self.primary_dependency().id()
  }

  pub fn dependency_ids(&self) -> &[DependencyId] {
    &self.dependency_ids
  }

  pub fn dependencies(&self) -> &[BoxDependency] {
    &self.dependencies
  }

  pub fn into_parts(self) -> (Vec<BoxDependency>, Vec<DependencyId>) {
    (self.dependencies, self.dependency_ids)
  }
}

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
  pub dependencies: ModuleDependencies,
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
    let FactorizeTask {
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
    let dependency = dependencies.primary_dependency();

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
          .map(|d| d.request().to_string())
      })
      .unwrap_or_default();
    let (dependencies, _) = dependencies.into_parts();
    // Error and result are not mutually exclusive in webpack module factorization.
    // Rspack puts results that need to be shared in both error and ok in [ModuleFactoryCreateData].
    let mut create_data = ModuleFactoryCreateData {
      compiler_id,
      compilation_id,
      resolve_options,
      options: options.clone(),
      context,
      request,
      dependencies,
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
      create_data
        .dependencies
        .iter()
        .map(|dependency| *dependency.id())
        .collect(),
      create_data.file_dependencies,
      create_data.context_dependencies,
      create_data.missing_dependencies,
    );
    Ok(vec![Box::new(FactorizeResultTask {
      original_module_identifier,
      factory_result,
      dependencies: ModuleDependencies::new(create_data.dependencies),
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
  pub dependencies: ModuleDependencies,
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

    let artifact = &mut context.artifact;
    if !factorize_info.is_success() {
      artifact
        .make_failed_dependencies
        .insert(dependencies.primary_id());
    }
    let resource_id = ResourceId::from(dependencies.primary_id());
    artifact
      .file_dependencies
      .add_files(&resource_id, factorize_info.file_dependencies());
    artifact
      .context_dependencies
      .add_files(&resource_id, factorize_info.context_dependencies());
    artifact
      .missing_dependencies
      .add_files(&resource_id, factorize_info.missing_dependencies());

    for dependency_id in dependencies.dependency_ids() {
      // Some dependencies do not come from the process_dependencies task,
      // so add all dependencies here.
      artifact.affected_dependencies.mark_as_add(dependency_id);
    }

    let (mut dependencies, _) = dependencies.into_parts();
    for dep in &mut dependencies {
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
    let Some(factory_result) = factory_result else {
      let dep = &dependencies[0];
      tracing::trace!("Module created with failure, but without bailout: {dep:?}");
      // sync dependencies to mg
      for dep in dependencies {
        module_graph.add_dependency(dep)
      }
      return Ok(vec![]);
    };

    let Some(module) = factory_result.module else {
      let dep = &dependencies[0];
      tracing::trace!("Module ignored: {dep:?}");
      // sync dependencies to mg
      for dep in dependencies {
        module_graph.add_dependency(dep)
      }
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
      dependencies: ModuleDependencies::new(dependencies),
      from_unlazy,
    })])
  }
}
