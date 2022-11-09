mod compilation;
mod resolver;

use anyhow::Context;
pub use compilation::*;
pub use resolver::*;
use std::{
  path::{Path, PathBuf},
  sync::Arc,
};

use hashbrown::HashMap;
use rayon::prelude::*;
use rspack_error::{Error, Result, TWithDiagnosticArray};
use tokio::sync::RwLock;
use tracing::instrument;

use crate::{
  CompilerOptions, Dependency, LoaderRunnerRunner, ModuleGraphModule, ModuleIdentifier,
  NormalModule, Plugin, PluginDriver, SharedPluginDriver, Stats,
};

#[derive(Debug)]
pub struct Compiler {
  pub options: Arc<CompilerOptions>,
  pub compilation: Compilation,
  pub plugin_driver: SharedPluginDriver,
  pub loader_runner_runner: Arc<LoaderRunnerRunner>,
}

impl Compiler {
  #[instrument(skip_all)]
  pub fn new(options: CompilerOptions, plugins: Vec<Box<dyn Plugin>>) -> Self {
    let options = Arc::new(options);

    let resolver_factory = Arc::new(ResolverFactory::new(options.resolve.clone()));
    let resolver = resolver_factory.get(Default::default());
    let plugin_driver = Arc::new(RwLock::new(PluginDriver::new(
      options.clone(),
      plugins,
      Arc::new(resolver),
    )));
    let loader_runner_runner = Arc::new(LoaderRunnerRunner::new(
      options.clone(),
      resolver_factory,
      plugin_driver.clone(),
    ));

    Self {
      options: options.clone(),
      compilation: Compilation::new(
        options,
        Default::default(),
        Default::default(),
        Default::default(),
        plugin_driver.clone(),
        loader_runner_runner.clone(),
      ),
      plugin_driver,
      loader_runner_runner,
    }
  }

  pub async fn run(&mut self) -> Result<()> {
    self.build().await?;
    Ok(())
  }

  #[instrument(name = "build")]
  pub async fn build(&mut self) -> Result<Stats> {
    // TODO: clear the outdate cache entires in resolver,
    // TODO: maybe it's better to use external entries.
    self.plugin_driver.read().await.resolver.clear();

    self.compilation = Compilation::new(
      // TODO: use Arc<T> instead
      self.options.clone(),
      self.options.entry.clone(),
      Default::default(),
      Default::default(),
      self.plugin_driver.clone(),
      self.loader_runner_runner.clone(),
    );

    // Fake this compilation as *currently* rebuilding does not create a new compilation
    self
      .plugin_driver
      .write()
      .await
      .this_compilation(&mut self.compilation)?;

    self
      .plugin_driver
      .write()
      .await
      .compilation(&mut self.compilation)?;

    let deps = self.compilation.entry_dependencies();
    self.compile(deps).await?;
    Ok(self.stats())
  }

  #[instrument(name = "compile")]
  async fn compile(&mut self, deps: HashMap<String, Vec<Dependency>>) -> Result<()> {
    let option = self.options.clone();
    self.compilation.make(deps).await;
    if option.builtins.tree_shaking {
      let result = self.compilation.optimize_dependency().await?;
      self.compilation.used_symbol = result.0;
      // This is only used when testing
      #[cfg(debug_assertions)]
      {
        self.compilation.tree_shaking_result = result.1;
      }
    }
    self.compilation.seal(self.plugin_driver.clone()).await?;

    // Consume plugin driver diagnostic
    let plugin_driver_diagnostics = self.plugin_driver.read().await.take_diagnostic();
    self
      .compilation
      .push_batch_diagnostic(plugin_driver_diagnostics);

    self.emit_assets(&self.compilation)?;
    self.compilation.done(self.plugin_driver.clone()).await?;

    Ok(())
  }

  fn stats(&self) -> Stats {
    let stats = Stats::new(&self.compilation);
    if self.options.__emit_error {
      stats.emit_diagnostics().unwrap();
    }
    stats
  }

  pub fn emit_assets(&self, compilation: &Compilation) -> Result<()> {
    let output_path = self.options.context.join(&self.options.output.path);
    if !output_path.exists() {
      std::fs::create_dir_all(&output_path)
        .with_context(|| format!("failed to create dir: {:?}", &output_path))
        .map_err(|e| Error::Anyhow { source: e })?;
    }

    compilation
      .assets()
      .par_iter()
      .try_for_each(|(filename, asset)| {
        let file_path = Path::new(&output_path).join(filename);
        self.emit_asset(file_path, asset)
      })
      .map_err(|e| e.into())
  }

  fn emit_asset(&self, file_path: PathBuf, asset: &CompilationAsset) -> anyhow::Result<()> {
    std::fs::create_dir_all(
      file_path
        .parent()
        .unwrap_or_else(|| panic!("The parent of {} can't found", file_path.display())),
    )?;
    std::fs::write(file_path, asset.get_source().buffer()).map_err(|e| e.into())
  }

  // TODO: remove this function when we had hash in stats.
  pub async fn rebuild(
    &mut self,
    changed_files: std::collections::HashSet<String>,
    removed_files: std::collections::HashSet<String>,
  ) -> Result<(std::collections::HashMap<String, (u8, String)>, Stats)> {
    let collect_modules_from_stats = |s: &Stats<'_>| -> HashMap<String, String> {
      let modules = s.compilation.module_graph.module_graph_modules();
      // TODO: use hash;

      modules
        .filter_map(|item| {
          use crate::SourceType::*;

          s.compilation
            .module_graph
            .module_by_identifier(&item.module_identifier)
            .and_then(|module| {
              let resource_data = module.resource_resolved_data();
              let resource_path = &resource_data.resource_path;

              if !changed_files.contains(resource_path) && !removed_files.contains(resource_path) {
                None
              } else if item.module_type.is_js_like() {
                s.compilation
                  .module_graph
                  .module_by_identifier(&item.module_identifier)
                  .map(|module| {
                    // TODO: it soo slowly, should use cache to instead.
                    let code = module.code_generation(item, s.compilation).unwrap();
                    let code = if let Some(code) = code.get(JavaScript) {
                      code.ast_or_source.as_source().unwrap().source().to_string()
                    } else {
                      println!("expect get JavaScirpt code");
                      String::new()
                    };
                    (item.module_identifier.clone(), code)
                  })
              } else if item.module_type.is_css() {
                s.compilation
                  .module_graph
                  .module_by_identifier(&item.module_identifier)
                  .map(|module| {
                    // TODO: it soo slowly, should use cache to instead.
                    let code = module.code_generation(item, s.compilation).unwrap();
                    let code = if let Some(code) = code.get(Css) {
                      // only used for compare between two build
                      code.ast_or_source.as_source().unwrap().source().to_string()
                    } else {
                      println!("expect get CSS code");
                      String::new()
                    };
                    (item.module_identifier.clone(), code)
                  })
              } else {
                None
              }
            })
        })
        .collect()
    };

    let old = self.compilation.get_stats();
    let old = collect_modules_from_stats(&old);

    let new_stats = self.build().await?;
    let new = collect_modules_from_stats(&new_stats);

    let mut diff = std::collections::HashMap::new();

    for (new_uri, new_content) in &new {
      if !old.contains_key(new_uri) {
        // added
        diff.insert(new_uri.to_string(), (2, new_content.to_string()));
      }
    }

    for (old_uri, old_content) in old {
      if let Some(new_content) = new.get(&old_uri) {
        // changed
        // TODO: should use module hash.
        if *new_content != old_content {
          diff.insert(old_uri, (0, new_content.to_string()));
        }
      } else {
        // deleted
        diff.insert(old_uri, (1, String::new()));
      }
    }
    // return all diff modules is not a good idea,
    // maybe a connection between browser client and local server is a better choice.
    Ok((diff, new_stats))
  }
}

pub type ModuleCreatedData = TWithDiagnosticArray<
  Box<(
    ModuleGraphModule,
    NormalModule,
    Option<ModuleIdentifier>,
    u32,
    Dependency,
    bool,
  )>,
>;

pub type ModuleResolvedData = TWithDiagnosticArray<(
  Option<ModuleIdentifier>,
  u32,
  Box<NormalModule>,
  Box<Vec<Dependency>>,
)>;

#[derive(Debug)]
pub enum Msg {
  DependencyReference((Dependency, u32), String),
  ModuleCreated(ModuleCreatedData),
  ModuleReused(TWithDiagnosticArray<(Option<ModuleIdentifier>, u32, ModuleIdentifier)>),
  ModuleResolved(ModuleResolvedData),
  ModuleBuiltErrorEncountered(ModuleIdentifier, Error),
  ModuleCreationCanceled,
  ModuleCreationErrorEncountered(Error),
}
