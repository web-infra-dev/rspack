use std::collections::HashMap;
use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use rspack_core::normalize_bundle_options;
use rspack_core::plugin_hook::get_resolver;
use rspack_core::Bundle;
use rspack_core::NormalizedBundleOptions;
use rspack_swc::{swc, swc_common};
use sugar_path::PathSugar;
use swc_common::Mark;
use tracing::instrument;

use crate::chunk_spliter::ChunkSpliter;
use crate::utils::inject_built_in_plugins;
use crate::utils::log::enable_tracing_by_env;
use crate::utils::rayon::init_rayon_thread_poll;
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
  pub options: Arc<NormalizedBundleOptions>,
  pub plugin_driver: Arc<PluginDriver>,
  pub bundle: Bundle,
  pub chunk_spliter: ChunkSpliter,
  _noop: (),
}

impl Bundler {
  pub fn new(options: BundleOptions, plugins: Vec<Box<dyn Plugin>>) -> Self {
    enable_tracing_by_env();
    init_rayon_thread_poll();
    println!(
      "create bundler with options:\n {:#?} \nplugins:\n {:#?}\n",
      options, plugins
    );
    let normalized_options = Arc::new(normalize_bundle_options(options));
    let injected_plugins = inject_built_in_plugins(plugins, &normalized_options);
    let (top_level_mark, unresolved_mark) =
      get_swc_compiler().run(|| (Mark::fresh(Mark::root()), Mark::fresh(Mark::root())));

    let ctx: Arc<BundleContext> = Arc::new(BundleContext::new(
      get_swc_compiler(),
      normalized_options.clone(),
      top_level_mark,
      unresolved_mark,
    ));
    let plugin_driver = Arc::new(PluginDriver {
      plugins: injected_plugins,
      ctx: ctx.clone(),
    });
    Self {
      options: normalized_options.clone(),
      plugin_driver: plugin_driver.clone(),
      bundle: Bundle::new(normalized_options.clone(), plugin_driver, ctx),
      chunk_spliter: ChunkSpliter::new(normalized_options.clone()),
      _noop: (),
    }
  }

  #[instrument(skip(self))]
  pub async fn build(&mut self, changed_files: Option<Vec<String>>) {
    self.plugin_driver.build_start().await;
    tracing::trace!("start build");

    self.bundle.build_graph(changed_files).await;

    let output = self
      .chunk_spliter
      .generate(&self.plugin_driver, &mut self.bundle);
    output.into_iter().for_each(|(_, chunk)| {
      self.bundle.context.assets.lock().unwrap().push(Asset {
        source: chunk.code,
        filename: chunk.file_name,
      })
    });

    self.plugin_driver.build_end().await;
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
      .bundle
      .module_graph
      .module_by_id
      .keys()
      .cloned()
      .collect::<HashSet<_>>();
    let changed_file: String = changed_file.into();
    old_modules_id.remove(&changed_file);
    tracing::trace!("old_modules_id {:?}", old_modules_id);

    self.bundle.context.assets.lock().unwrap().clear();

    self.build(Some(vec![changed_file])).await;

    let new_modules_id = self
      .bundle
      .module_graph
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
          .bundle
          .module_graph
          .module_by_id
          .get(&module_id)
          .unwrap();
        let compiler = get_swc_compiler();

        let transform_output =
          swc::try_with_handler(compiler.cm.clone(), Default::default(), |handler| {
            module.render(
              &compiler,
              handler,
              &self.bundle.module_graph.module_by_id,
              &self.options,
              &self.bundle.context,
            )
          })
          .unwrap();
        (module_id.to_string(), transform_output.code)
      })
      .collect();
    diff_rendered
  }

  pub fn write_assets_to_disk(&self) {
    self
      .bundle
      .context
      .assets
      .lock()
      .unwrap()
      .iter()
      .for_each(|asset| {
        let mut path = PathBuf::from(self.options.outdir.clone());
        // .map(PathBuf::from)
        // .unwrap_or_else(|| std::env::current_dir().unwrap());
        path.push(&asset.filename);
        std::fs::create_dir_all(path.resolve().parent().unwrap()).unwrap();
        std::fs::write(path.resolve(), &asset.source).unwrap();
      });
  }
}
