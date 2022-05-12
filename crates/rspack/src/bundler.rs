use std::collections::HashMap;
use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use rspack_core::normalize_bundle_options;
use rspack_core::plugin_hook::get_resolver;
use rspack_core::ModuleGraph;
use rspack_core::NormalizedBundleOptions;
use rspack_core::SWC_GLOBALS;
use sugar_path::PathSugar;
use swc_common::Globals;
use swc_common::Mark;
use swc_common::GLOBALS;
use tracing::instrument;

use crate::chunk_spliter::ChunkSpliter;
use crate::utils::inject_built_in_plugins;
use crate::utils::log::enable_tracing_by_env;
use rspack_core::get_swc_compiler;
pub use rspack_core::hmr::hmr_module;
use rspack_core::Plugin;
use rspack_core::PluginDriver;
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum InternalModuleFormat {
  ES,
  CJS,
  AMD,
  UMD,
}

impl Default for InternalModuleFormat {
  fn default() -> Self {
    InternalModuleFormat::ES
  }
}

pub use rspack_core::Asset;
pub use rspack_core::BundleContext;
pub use rspack_core::BundleMode;
pub use rspack_core::BundleOptions;

#[derive(Debug)]
pub struct Bundler {
  pub ctx: Arc<BundleContext>,
  pub options: Arc<NormalizedBundleOptions>,
  pub plugin_driver: Arc<PluginDriver>,
  pub module_graph: Option<ModuleGraph>,
  _noop: (),
}

impl Bundler {
  pub fn new(mut options: BundleOptions, plugins: Vec<Box<dyn Plugin>>) -> Self {
    enable_tracing_by_env();
    println!(
      "create bundler with options:\n {:#?} \nplugins:\n {:#?}\n",
      options, plugins
    );
    let injected_plugins = inject_built_in_plugins(plugins, &mut options);
    let normalized_options = Arc::new(normalize_bundle_options(options));
    let top_level_mark = GLOBALS.set(&SWC_GLOBALS, || Mark::fresh(Mark::root()));
    let ctx: Arc<BundleContext> = Arc::new(BundleContext::new(
      get_swc_compiler(),
      normalized_options.clone(),
      top_level_mark,
    ));
    Self {
      options: normalized_options,
      ctx: ctx.clone(),
      plugin_driver: Arc::new(PluginDriver {
        plugins: injected_plugins,
        ctx,
      }),
      module_graph: None,
      _noop: (),
    }
  }

  #[instrument(skip(self))]
  pub async fn build(&mut self) {
    self.plugin_driver.build_start().await;
    tracing::trace!("start build");
    let mut bundle = rspack_core::Bundle::new(
      self.options.clone(),
      self.plugin_driver.clone(),
      self.ctx.clone(),
    );

    bundle.build_graph().await;

    let mut chunk_spliter = ChunkSpliter::new(self.options.clone());
    let output = chunk_spliter.generate(&self.plugin_driver, &mut bundle);
    output.into_iter().for_each(|(_, chunk)| {
      self.ctx.assets.lock().unwrap().push(Asset {
        source: chunk.code,
        filename: chunk.file_name,
      })
    });

    self.module_graph = Some(bundle.module_graph.take().unwrap());
  }
  pub fn resolve(
    &mut self,
    id: String,
    dir: String,
  ) -> Result<nodejs_resolver::ResolveResult, std::string::String> {
    let resolver = get_resolver(self.options.as_ref());
    let base = Path::new(&dir);
    let res = resolver.resolve(base, &id);
    res
  }
  #[instrument(skip(self))]
  pub async fn rebuild(&mut self, changed_file: String) -> HashMap<String, String> {
    tracing::debug!("rebuld bacause of {:?}", changed_file);
    let mut old_modules_id = self
      .module_graph
      .as_ref()
      .unwrap()
      .module_by_id
      .keys()
      .cloned()
      .collect::<HashSet<_>>();
    let changed_file: String = changed_file.into();
    old_modules_id.remove(&changed_file);
    tracing::trace!("old_modules_id {:?}", old_modules_id);
    let mut bundle = {
      // TODO: We need to reuse some cache. Rebuild is fake now.
      let mut bundle = rspack_core::Bundle::new(
        self.options.clone(),
        self.plugin_driver.clone(),
        self.ctx.clone(),
      );

      bundle.build_graph().await;
      bundle
    };
    let mut chunk_spliter = ChunkSpliter::new(self.options.clone());
    let output = chunk_spliter.generate(&self.plugin_driver, &mut bundle);
    output.into_iter().for_each(|(_, chunk)| {
      self.ctx.assets.lock().unwrap().push(Asset {
        source: chunk.code,
        filename: chunk.file_name,
      })
    });
    self.module_graph = Some(bundle.module_graph.take().unwrap());

    let new_modules_id = self
      .module_graph
      .as_ref()
      .unwrap()
      .module_by_id
      .keys()
      .cloned()
      .collect::<HashSet<_>>();
    let diff_rendered = new_modules_id
      .into_iter()
      .filter(|module_id| !old_modules_id.contains(module_id))
      .map(|module_id| {
        tracing::trace!("render new added module {:?}", module_id);
        let module = self
          .module_graph
          .as_ref()
          .unwrap()
          .module_by_id
          .get(&module_id)
          .unwrap();
        let compiler = get_swc_compiler();

        let transform_output =
          swc::try_with_handler(compiler.cm.clone(), Default::default(), |handler| {
            module.render(
              &compiler,
              handler,
              &self.module_graph.as_ref().unwrap().module_by_id,
              &self.options,
              &bundle.context,
            )
          })
          .unwrap();
        (module_id.to_string(), transform_output.code)
      })
      .collect();
    diff_rendered
  }

  pub fn write_assets_to_disk(&self) {
    self.ctx.assets.lock().unwrap().iter().for_each(|asset| {
      let mut path = PathBuf::from(self.options.outdir.clone());
      // .map(PathBuf::from)
      // .unwrap_or_else(|| std::env::current_dir().unwrap());
      path.push(&asset.filename);
      std::fs::create_dir_all(path.resolve().parent().unwrap()).unwrap();
      std::fs::write(path.resolve(), &asset.source).unwrap();
    });
  }
}
