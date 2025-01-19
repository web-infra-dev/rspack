use std::sync::Arc;

use rspack_error::Diagnostic;
use rspack_paths::ArcPath;
use rspack_sources::BoxSource;
use rustc_hash::FxHashSet as HashSet;

use super::{add::AddTask, MakeTaskContext};
use crate::{
  module_graph::ModuleGraphModule,
  utils::task_loop::{Task, TaskResult, TaskType},
  BoxDependency, CompilationId, CompilerId, CompilerOptions, Context, ExportInfoData,
  ExportsInfoData, ModuleFactory, ModuleFactoryCreateData, ModuleFactoryResult, ModuleIdentifier,
  ModuleLayer, ModuleProfile, Resolve, ResolverFactory,
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
  pub current_profile: Option<Box<ModuleProfile>>,
  pub resolver_factory: Arc<ResolverFactory>,
}

#[async_trait::async_trait]
impl Task<MakeTaskContext> for FactorizeTask {
  fn get_task_type(&self) -> TaskType {
    TaskType::Async
  }
  async fn background_run(self: Box<Self>) -> TaskResult<MakeTaskContext> {
    if let Some(current_profile) = &self.current_profile {
      current_profile.mark_factory_start();
    }
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

    let other_exports_info = ExportInfoData::new(None, None);
    let side_effects_only_info = ExportInfoData::new(Some("*side effects only*".into()), None);
    let exports_info = ExportsInfoData::new(other_exports_info.id(), side_effects_only_info.id());
    let factorize_result_task = FactorizeResultTask {
      // dependency: dep_id,
      original_module_identifier: self.original_module_identifier,
      factory_result: None,
      dependencies: vec![],
      current_profile: self.current_profile,
      exports_info_related: ExportsInfoRelated {
        exports_info,
        other_exports_info,
        side_effects_info: side_effects_only_info,
      },
      file_dependencies: Default::default(),
      context_dependencies: Default::default(),
      missing_dependencies: Default::default(),
      diagnostics: Default::default(),
    };

    // Error and result are not mutually exclusive in webpack module factorization.
    // Rspack puts results that need to be shared in both error and ok in [ModuleFactoryCreateData].
    let mut create_data = ModuleFactoryCreateData {
      compiler_id: self.compiler_id,
      compilation_id: self.compilation_id,
      resolve_options: self.resolve_options,
      options: self.options.clone(),
      context,
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
    match self.module_factory.create(&mut create_data).await {
      Ok(result) => {
        if let Some(current_profile) = &factorize_result_task.current_profile {
          current_profile.mark_factory_end();
        }
        let diagnostics = create_data.diagnostics.drain(..).collect();
        Ok(vec![Box::new(
          factorize_result_task
            .with_dependencies(create_data.dependencies)
            .with_factory_result(Some(result))
            .with_diagnostics(diagnostics)
            .with_file_dependencies(create_data.file_dependencies.drain())
            .with_missing_dependencies(create_data.missing_dependencies.drain())
            .with_context_dependencies(create_data.context_dependencies.drain()),
        )])
      }
      Err(mut e) => {
        if let Some(current_profile) = &factorize_result_task.current_profile {
          current_profile.mark_factory_end();
        }
        // Wrap source code if available
        if let Some(s) = self.original_module_source {
          let has_source_code = e.source_code().is_some();
          if !has_source_code {
            e = e.with_source_code(s.source().to_string());
          }
        }
        // Bail out if `options.bail` set to `true`,
        // which means 'Fail out on the first error instead of tolerating it.'
        if self.options.bail {
          return Err(e);
        }
        let mut diagnostics = Vec::with_capacity(create_data.diagnostics.len() + 1);
        diagnostics.push(
          Into::<Diagnostic>::into(e)
            .with_loc(create_data.dependencies[0].loc().map(|loc| loc.to_string())),
        );
        diagnostics.append(&mut create_data.diagnostics);
        // Continue bundling if `options.bail` set to `false`.
        Ok(vec![Box::new(
          factorize_result_task
            .with_dependencies(create_data.dependencies)
            .with_diagnostics(diagnostics)
            .with_file_dependencies(create_data.file_dependencies.drain())
            .with_missing_dependencies(create_data.missing_dependencies.drain())
            .with_context_dependencies(create_data.context_dependencies.drain()),
        )])
      }
    }
  }
}

/// a struct temporarily used creating ExportsInfo
#[derive(Debug)]
pub struct ExportsInfoRelated {
  pub exports_info: ExportsInfoData,
  pub other_exports_info: ExportInfoData,
  pub side_effects_info: ExportInfoData,
}

#[derive(Debug)]
pub struct FactorizeResultTask {
  //  pub dependency: DependencyId,
  pub original_module_identifier: Option<ModuleIdentifier>,
  /// Result will be available if [crate::ModuleFactory::create] returns `Ok`.
  pub factory_result: Option<ModuleFactoryResult>,
  pub dependencies: Vec<BoxDependency>,
  pub current_profile: Option<Box<ModuleProfile>>,
  pub exports_info_related: ExportsInfoRelated,

  pub file_dependencies: HashSet<ArcPath>,
  pub context_dependencies: HashSet<ArcPath>,
  pub missing_dependencies: HashSet<ArcPath>,
  pub diagnostics: Vec<Diagnostic>,
}

impl FactorizeResultTask {
  fn with_dependencies(mut self, dependencies: Vec<BoxDependency>) -> Self {
    self.dependencies = dependencies;
    self
  }

  fn with_factory_result(mut self, factory_result: Option<ModuleFactoryResult>) -> Self {
    self.factory_result = factory_result;
    self
  }

  fn with_diagnostics(mut self, diagnostics: Vec<Diagnostic>) -> Self {
    self.diagnostics = diagnostics;
    self
  }

  fn with_file_dependencies(mut self, files: impl IntoIterator<Item = ArcPath>) -> Self {
    self.file_dependencies = files.into_iter().collect();
    self
  }

  fn with_context_dependencies(mut self, contexts: impl IntoIterator<Item = ArcPath>) -> Self {
    self.context_dependencies = contexts.into_iter().collect();
    self
  }

  fn with_missing_dependencies(mut self, missing: impl IntoIterator<Item = ArcPath>) -> Self {
    self.missing_dependencies = missing.into_iter().collect();
    self
  }
}

#[async_trait::async_trait]
impl Task<MakeTaskContext> for FactorizeResultTask {
  fn get_task_type(&self) -> TaskType {
    TaskType::Sync
  }
  async fn main_run(self: Box<Self>, context: &mut MakeTaskContext) -> TaskResult<MakeTaskContext> {
    let FactorizeResultTask {
      original_module_identifier,
      factory_result,
      dependencies,
      current_profile,
      exports_info_related,
      file_dependencies,
      context_dependencies,
      missing_dependencies,
      diagnostics,
    } = *self;
    let artifact = &mut context.artifact;
    if !diagnostics.is_empty() {
      if let Some(id) = original_module_identifier {
        artifact.make_failed_module.insert(id);
      } else {
        artifact
          .make_failed_dependencies
          .insert((*dependencies[0].id(), None));
      }
    }

    artifact.diagnostics.extend(
      diagnostics
        .into_iter()
        .map(|d| d.with_module_identifier(original_module_identifier)),
    );

    artifact
      .file_dependencies
      .add_batch_file(&file_dependencies);
    artifact
      .context_dependencies
      .add_batch_file(&context_dependencies);
    artifact
      .missing_dependencies
      .add_batch_file(&missing_dependencies);
    let module_graph =
      &mut MakeTaskContext::get_module_graph_mut(&mut artifact.module_graph_partial);
    let Some(factory_result) = factory_result else {
      let dep = &dependencies[0];
      tracing::trace!("Module created with failure, but without bailout: {dep:?}");
      return Ok(vec![]);
    };

    let Some(module) = factory_result.module else {
      let dep = &dependencies[0];
      tracing::trace!("Module ignored: {dep:?}");
      return Ok(vec![]);
    };
    let module_identifier = module.identifier();
    let mut mgm =
      ModuleGraphModule::new(module.identifier(), exports_info_related.exports_info.id());
    mgm.set_issuer_if_unset(original_module_identifier);

    module_graph.set_exports_info(
      exports_info_related.exports_info.id(),
      exports_info_related.exports_info,
    );
    module_graph.set_export_info(
      exports_info_related.side_effects_info.id(),
      exports_info_related.side_effects_info,
    );
    module_graph.set_export_info(
      exports_info_related.other_exports_info.id(),
      exports_info_related.other_exports_info,
    );
    tracing::trace!("Module created: {}", &module_identifier);

    Ok(vec![Box::new(AddTask {
      original_module_identifier,
      module,
      module_graph_module: Box::new(mgm),
      dependencies,
      current_profile,
    })])
  }
}
