use std::{
  fmt::Debug,
  sync::atomic::{AtomicU32, Ordering},
  sync::Arc,
};

use futures::{stream::FuturesUnordered, StreamExt};
use hashbrown::HashMap;
use tokio::sync::mpsc::UnboundedSender;
use tracing::instrument;

use rspack_error::{Diagnostic, IntoTWithDiagnosticArray, Result};
use rspack_sources::BoxSource;

use crate::{
  split_chunks::code_splitting, BuildContext, Chunk, ChunkByUkey, ChunkGraph, ChunkGroup,
  ChunkGroupUkey, ChunkKind, ChunkUkey, CompilerOptions, Dependency, EntryItem, Entrypoint,
  LoaderRunnerRunner, ModuleDependency, ModuleGraph, ModuleIdentifier, ModuleRule, Msg,
  NormalModule, NormalModuleFactory, NormalModuleFactoryContext, ProcessAssetsArgs,
  RenderManifestArgs, RenderRuntimeArgs, ResolveKind, Runtime, SharedPluginDriver,
  VisitedModuleIdentity,
};

#[derive(Debug)]
pub struct Compilation {
  pub options: Arc<CompilerOptions>,
  entries: HashMap<String, EntryItem>,
  pub(crate) visited_module_id: VisitedModuleIdentity,
  pub module_graph: ModuleGraph,
  pub chunk_graph: ChunkGraph,
  pub chunk_by_ukey: HashMap<ChunkUkey, Chunk>,
  pub chunk_group_by_ukey: HashMap<ChunkGroupUkey, ChunkGroup>,
  pub runtime: Runtime,
  pub entrypoints: HashMap<String, ChunkGroupUkey>,
  pub assets: CompilationAssets,
  pub diagnostic: Vec<Diagnostic>,
  pub(crate) plugin_driver: SharedPluginDriver,
  pub(crate) loader_runner_runner: Arc<LoaderRunnerRunner>,
  pub(crate) _named_chunk: HashMap<String, ChunkUkey>,
  pub(crate) named_chunk_groups: HashMap<String, ChunkGroupUkey>,
}
impl Compilation {
  pub fn new(
    options: Arc<CompilerOptions>,
    entries: std::collections::HashMap<String, EntryItem>,
    visited_module_id: VisitedModuleIdentity,
    module_graph: ModuleGraph,
    plugin_driver: SharedPluginDriver,
    loader_runner_runner: Arc<LoaderRunnerRunner>,
  ) -> Self {
    Self {
      options,
      visited_module_id,
      module_graph,
      chunk_by_ukey: Default::default(),
      chunk_group_by_ukey: Default::default(),
      entries: HashMap::from_iter(entries),
      chunk_graph: Default::default(),
      runtime: Default::default(),
      entrypoints: Default::default(),
      assets: Default::default(),
      diagnostic: vec![],
      plugin_driver,
      loader_runner_runner,
      _named_chunk: Default::default(),
      named_chunk_groups: Default::default(),
    }
  }
  pub fn add_entry(&mut self, name: String, detail: EntryItem) {
    self.entries.insert(name, detail);
  }

  pub fn emit_asset(&mut self, filename: String, asset: BoxSource) {
    self.assets.insert(filename, asset);
  }

  pub fn assets(&self) -> &CompilationAssets {
    &self.assets
  }

  pub fn push_diagnostic(&mut self, diagnostic: Diagnostic) {
    self.diagnostic.push(diagnostic);
  }

  pub fn push_batch_diagnostic(&mut self, mut diagnostic: Vec<Diagnostic>) {
    self.diagnostic.append(&mut diagnostic);
  }

  pub(crate) fn add_chunk(
    chunk_by_ukey: &mut ChunkByUkey,
    name: Option<String>,
    id: String,
    kind: ChunkKind,
  ) -> &mut Chunk {
    let chunk = Chunk::new(name, id, kind);
    let ukey = chunk.ukey;
    chunk_by_ukey.insert(chunk.ukey, chunk);
    chunk_by_ukey.get_mut(&ukey).expect("chunk not found")
  }
  #[instrument(name = "entry_deps")]
  pub fn entry_dependencies(&self) -> HashMap<String, Dependency> {
    self
      .entries
      .iter()
      .map(|(name, detail)| {
        (
          name.clone(),
          Dependency {
            importer: None,
            parent_module_identifier: None,
            detail: ModuleDependency {
              specifier: detail.path.clone(),
              kind: ResolveKind::Import,
              span: None,
            },
          },
        )
      })
      .collect()
  }

  #[instrument(name = "compilation:make")]
  pub async fn make(&mut self, entry_deps: HashMap<String, Dependency>) {
    let active_task_count: Arc<AtomicU32> = Arc::new(AtomicU32::new(0));
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Msg>();

    entry_deps.into_iter().for_each(|(name, dep)| {
      let normal_module_factory = NormalModuleFactory::new(
        NormalModuleFactoryContext {
          module_name: Some(name),
          active_task_count: active_task_count.clone(),
          module_type: None,
          side_effects: None,
          options: self.options.clone(),
        },
        dep,
        tx.clone(),
        self.plugin_driver.clone(),
      );

      tokio::task::spawn(async move { normal_module_factory.create().await });
    });

    while active_task_count.load(Ordering::SeqCst) != 0 {
      match rx.recv().await {
        Some(job) => match job {
          Msg::ModuleCreated(module_with_diagnostic) => {
            let (mgm, module, original_module_identifier, dependency_id, dependency) =
              *module_with_diagnostic.inner;

            let module_identifier = module.identifier();

            if self
              .visited_module_id
              .contains(&(module_identifier.clone(), dependency.detail.clone()))
            {
              if let Err(err) = tx.send(Msg::ModuleReused(
                (original_module_identifier, dependency_id, module_identifier)
                  .with_diagnostic(module_with_diagnostic.diagnostic),
              )) {
                tracing::trace!("fail to send msg {:?}", err)
              }
              continue;
            }

            self
              .visited_module_id
              .insert((module_identifier.clone(), dependency.detail.clone()));

            if let Err(err) = tx.send(Msg::ModuleGraphModuleCreated(mgm)) {
              tracing::trace!("fail to send msg {:?}", err)
            }

            self.handle_module_build_and_dependencies(
              original_module_identifier,
              module,
              dependency_id,
              active_task_count.clone(),
              tx.clone(),
            );
          }
          Msg::ModuleGraphModuleCreated(mgm) => {
            self.module_graph.add_module_graph_module(mgm);
          }
          Msg::ModuleReused(result_with_diagnostics) => {
            active_task_count.fetch_sub(1, Ordering::SeqCst);

            let (original_module_identifier, dependency_id, module_identifier) =
              result_with_diagnostics.inner;
            self.push_batch_diagnostic(result_with_diagnostics.diagnostic);

            if let Err(err) = self.module_graph.set_resolved_module(
              original_module_identifier,
              dependency_id,
              module_identifier.clone(),
            ) {
              if let Err(err) = tx.send(Msg::ModuleBuiltErrorEncountered(module_identifier, err)) {
                tracing::trace!("fail to send msg {:?}", err)
              }
            };
          }
          Msg::ModuleResolved(result_with_diagnostics) => {
            active_task_count.fetch_sub(1, Ordering::SeqCst);

            let (original_module_identifier, dependency_id, module, deps) =
              result_with_diagnostics.inner;
            self.push_batch_diagnostic(result_with_diagnostics.diagnostic);

            {
              let mut module_graph_module = self
                .module_graph
                .module_graph_module_by_identifier_mut(&module.identifier())
                .unwrap();
              module_graph_module.all_dependencies = *deps;
            }
            if let Err(err) = self.module_graph.set_resolved_module(
              original_module_identifier,
              dependency_id,
              module.identifier(),
            ) {
              if let Err(err) = tx.send(Msg::ModuleBuiltErrorEncountered(module.identifier(), err))
              {
                tracing::trace!("fail to send msg {:?}", err)
              }
            };
            self.module_graph.add_module(*module);
          }
          Msg::ModuleBuiltErrorEncountered(module_identifier, err) => {
            active_task_count.fetch_sub(1, Ordering::SeqCst);
            self
              .module_graph
              .module_identifier_to_module
              .remove(&module_identifier);
            self
              .module_graph
              .module_identifier_to_module_graph_module
              .remove(&module_identifier);
            self.push_batch_diagnostic(err.into());
          }
          Msg::ModuleCreationCanceled => {
            active_task_count.fetch_sub(1, Ordering::SeqCst);
          }
          Msg::DependencyReference(dep, resolved_uri) => {
            self.module_graph.add_dependency(dep, resolved_uri);
          }
          Msg::ModuleCreationErrorEncountered(err) => {
            active_task_count.fetch_sub(1, Ordering::SeqCst);
            self.push_batch_diagnostic(err.into());
          }
        },
        None => {
          tracing::trace!("All sender is dropped");
        }
      }
    }
    tracing::debug!("module graph {:#?}", self.module_graph);
  }

  fn handle_module_build_and_dependencies(
    &self,
    original_module_identifier: Option<ModuleIdentifier>,
    mut module: NormalModule,
    dependency_id: u32,
    active_task_count: Arc<AtomicU32>,
    tx: UnboundedSender<Msg>,
  ) {
    let compiler_options = self.options.clone();
    let loader_runner_runner = self.loader_runner_runner.clone();
    let plugin_driver = self.plugin_driver.clone();

    let module_identifier = module.identifier();

    tokio::spawn(async move {
      let resource_data = module.resource_resolved_data();
      let resolved_loaders = match compiler_options
        .module
        .rules
        .iter()
        .filter_map(|module_rule| -> Option<Result<&ModuleRule>> {
          if let Some(func) = &module_rule.func__ {
            match func(resource_data) {
              Ok(result) => {
                if result {
                  return Some(Ok(module_rule));
                }

                return None
              },
              Err(e) => {
                return Some(Err(e.into()))
              }
            }
          }

          // Include all modules that pass test assertion. If you supply a Rule.test option, you cannot also supply a `Rule.resource`.
          // See: https://webpack.js.org/configuration/module/#ruletest
          if let Some(test_rule) = &module_rule.test && test_rule.is_match(&resource_data.resource) {
            return Some(Ok(module_rule));
          } else if let Some(resource_rule) = &module_rule.resource && resource_rule.is_match(&resource_data.resource) {
            return Some(Ok(module_rule));
          }

          if let Some(resource_query_rule) = &module_rule.resource_query && let Some(resource_query) = &resource_data.resource_query && resource_query_rule.is_match(resource_query) {
            return Some(Ok(module_rule));
          }

          None
        })
        .collect::<Result<Vec<_>>>() {
          Ok(result) => result,
          Err(err) => {
            if let Err(err) =
              tx.send(Msg::ModuleBuiltErrorEncountered(module_identifier, err))
            {
              tracing::trace!("fail to send msg {:?}", err)
            }
            return
          }
        };

      let resolved_loaders = resolved_loaders
        .into_iter()
        .flat_map(|module_rule| module_rule.uses.iter().map(Box::as_ref).rev())
        .collect::<Vec<_>>();

      match module
        .build(BuildContext {
          resolved_loaders,
          loader_runner_runner: &loader_runner_runner,
          compiler_options: &compiler_options,
        })
        .await
      {
        Ok(build_result) => {
          let module_identifier = module.identifier();

          let (build_result, diagnostics) = build_result.split_into_parts();

          let deps = build_result
            .dependencies
            .into_iter()
            .map(|dep| Dependency {
              importer: Some(module_identifier.clone()),
              parent_module_identifier: Some(module_identifier.clone()),
              detail: dep,
            })
            .collect::<Vec<_>>();

          if let Err(err) = tx.send(Msg::ModuleResolved(
            (
              original_module_identifier.clone(),
              dependency_id,
              Box::new(module),
              Box::new(deps.clone()),
            )
              .with_diagnostic(diagnostics),
          )) {
            tracing::trace!("fail to send msg {:?}", err);
            return;
          };

          Compilation::process_module_dependencies(
            deps,
            active_task_count.clone(),
            tx.clone(),
            plugin_driver.clone(),
            compiler_options.clone(),
          );
        }
        Err(err) => {
          if let Err(err) = tx.send(Msg::ModuleBuiltErrorEncountered(module_identifier, err)) {
            tracing::trace!("fail to send msg {:?}", err)
          }
        }
      }
    });
  }

  fn process_module_dependencies(
    dependencies: Vec<Dependency>,
    active_task_count: Arc<AtomicU32>,
    tx: UnboundedSender<Msg>,
    plugin_driver: SharedPluginDriver,
    compiler_options: Arc<CompilerOptions>,
  ) {
    dependencies.into_iter().for_each(|dep| {
      let normal_module_factory = NormalModuleFactory::new(
        NormalModuleFactoryContext {
          module_name: None,
          module_type: None,
          active_task_count: active_task_count.clone(),
          side_effects: None,
          options: compiler_options.clone(),
        },
        dep,
        tx.clone(),
        plugin_driver.clone(),
      );

      tokio::task::spawn(async move {
        normal_module_factory.create().await;
      });
    })
  }

  #[instrument()]
  async fn create_chunk_assets(&mut self, plugin_driver: SharedPluginDriver) {
    let chunk_ukey_and_manifest = (self
      .chunk_by_ukey
      .values()
      .map(|chunk| async {
        let manifest = plugin_driver
          .read()
          .await
          .render_manifest(RenderManifestArgs {
            chunk_ukey: chunk.ukey,
            compilation: self,
          });
        (chunk.ukey, manifest)
      })
      .collect::<FuturesUnordered<_>>())
    .collect::<Vec<_>>()
    .await;

    chunk_ukey_and_manifest
      .into_iter()
      .for_each(|(chunk_ukey, manifest)| {
        manifest.unwrap().into_iter().for_each(|file_manifest| {
          let current_chunk = self.chunk_by_ukey.get_mut(&chunk_ukey).unwrap();
          current_chunk
            .files
            .insert(file_manifest.filename().to_string());

          self.emit_asset(file_manifest.filename().to_string(), file_manifest.source);
        });
      })
  }
  #[instrument(name = "compilation:process_asssets")]
  async fn process_assets(&mut self, plugin_driver: SharedPluginDriver) {
    plugin_driver
      .write()
      .await
      .process_assets(ProcessAssetsArgs { compilation: self })
      .await
      .map_err(|e| {
        eprintln!("process_assets is not ok, err {:#?}", e);
        e
      })
      .ok();
  }
  pub async fn done(&mut self, plugin_driver: SharedPluginDriver) -> Result<()> {
    plugin_driver.write().await.done().await?;
    Ok(())
  }
  #[instrument(name = "compilation:render_runtime")]
  pub async fn render_runtime(&self, plugin_driver: SharedPluginDriver) -> Runtime {
    if let Ok(sources) = plugin_driver
      .read()
      .await
      .render_runtime(RenderRuntimeArgs {
        sources: vec![],
        compilation: self,
      })
    {
      Runtime {
        context_indent: self.runtime.context_indent.clone(),
        sources,
      }
    } else {
      self.runtime.clone()
    }
  }

  pub fn entry_modules(&self) {
    // self.
    todo!()
  }

  pub fn entrypoint_by_name(&self, name: &str) -> &Entrypoint {
    let ukey = self.entrypoints.get(name).expect("entrypoint not found");
    self
      .chunk_group_by_ukey
      .get(ukey)
      .expect("entrypoint not found by ukey")
  }
  #[instrument(name = "compilation:seal")]
  pub async fn seal(&mut self, plugin_driver: SharedPluginDriver) -> Result<()> {
    code_splitting(self)?;

    plugin_driver.write().await.optimize_chunks(self)?;

    tracing::debug!("chunk graph {:#?}", self.chunk_graph);

    let context_indent = if matches!(
      self.options.target.platform,
      crate::TargetPlatform::Web | crate::TargetPlatform::None
    ) {
      String::from("self")
    } else {
      String::from("this")
    };

    self.runtime = Runtime {
      sources: vec![],
      context_indent,
    };

    self.create_chunk_assets(plugin_driver.clone()).await;
    // generate runtime
    self.runtime = self.render_runtime(plugin_driver.clone()).await;

    self.process_assets(plugin_driver).await;
    Ok(())
  }
}

pub type CompilationAssets = HashMap<String, BoxSource>;
