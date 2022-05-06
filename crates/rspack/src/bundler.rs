use std::collections::HashMap;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;

use rspack_core::ModuleGraph;
use smol_str::SmolStr;
use sugar_path::PathSugar;
use swc::config::Options;
use swc_common::FileName;
use swc_common::Mark;
use swc_ecma_transforms_base::pass::noop;
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
  pub options: Arc<BundleOptions>,
  pub plugin_driver: Arc<PluginDriver>,
  module_graph: Option<ModuleGraph>,
}

impl Bundler {
  pub fn new(options: BundleOptions, plugins: Vec<Box<dyn Plugin>>) -> Self {
    enable_tracing_by_env();
    let injected_plugins = inject_built_in_plugins(plugins);
    let ctx: Arc<BundleContext> = Default::default();
    Self {
      options: Arc::new(options),
      ctx: ctx.clone(),
      plugin_driver: Arc::new(PluginDriver {
        plugins: injected_plugins,
        ctx,
      }),
      module_graph: None,
    }
  }

  #[instrument(skip(self))]
  pub async fn build(&mut self) {
    let mut bundle = rspack_core::Bundle::new(
      self.options.clone(),
      self.plugin_driver.clone(),
      self.ctx.clone(),
    );

    bundle.build_graph().await;

    let mut chunk_spliter = ChunkSpliter::new(self.options.clone());
    let output = chunk_spliter.generate(&self.plugin_driver, bundle.module_graph.as_mut().unwrap());
    output.into_iter().for_each(|(_, chunk)| {
      self.ctx.assets.lock().unwrap().push(Asset {
        source: chunk.code,
        filename: chunk.file_name,
      })
    });

    self.module_graph = Some(bundle.module_graph.take().unwrap());
  }

  #[instrument(skip(self))]
  pub async fn rebuild(&mut self, changed_file: String) -> HashMap<String, String> {
    tracing::trace!("rebuld bacause of {:?}", changed_file);
    let mut old_modules_id = self
      .module_graph
      .as_ref()
      .unwrap()
      .module_by_id
      .keys()
      .cloned()
      .collect::<HashSet<_>>();
    let changed_file: SmolStr = changed_file.into();
    old_modules_id.remove(&changed_file);
    tracing::trace!("old_modules_id {:?}", old_modules_id);
    let mut module_graph = {
      // TODO: We need to reuse some cache. Rebuild is fake now.
      let mut bundle = rspack_core::Bundle::new(
        self.options.clone(),
        self.plugin_driver.clone(),
        self.ctx.clone(),
      );

      bundle.build_graph().await;
      bundle.module_graph.take().unwrap()
    };
    let mut bundle = ChunkSpliter::new(self.options.clone());
    let output = bundle.generate(&self.plugin_driver, &mut module_graph);
    output.into_iter().for_each(|(_, chunk)| {
      self.ctx.assets.lock().unwrap().push(Asset {
        source: chunk.code,
        filename: chunk.file_name,
      })
    });
    self.module_graph = Some(module_graph);

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
        let top_level_mark = Mark::from_u32(1);

        let transoform_output =
          swc::try_with_handler(compiler.cm.clone(), Default::default(), |handler| {
            let fm = compiler.cm.new_source_file(
              FileName::Custom(module.id.to_string()),
              module.id.to_string(),
            );
            Ok(
              compiler
                .process_js_with_custom_pass(
                  fm,
                  Some(swc_ecma_ast::Program::Module(module.ast.clone())),
                  handler,
                  &Options {
                    global_mark: Some(top_level_mark),
                    ..Default::default()
                  },
                  |_, _| noop(),
                  |_, _| {
                    hmr_module(
                      module.id.to_string(),
                      top_level_mark,
                      &module.resolved_ids,
                      module.is_user_defined_entry_point,
                    )
                  },
                )
                .unwrap(),
            )
          })
          .unwrap();
        (module_id.to_string(), transoform_output.code)
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
