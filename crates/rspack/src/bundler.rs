use std::collections::HashMap;
use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use nodejs_resolver::Resolver;
use rayon::prelude::*;
use rspack_core::inject_options;
use rspack_core::Bundle;
use rspack_core::NormalizedBundleOptions;
use rspack_core::{OutputChunk, OutputChunkSourceMap};
use rspack_swc::swc_common;
use sugar_path::PathSugar;
use swc_common::Mark;
use tracing::instrument;

use crate::chunk_spliter::generate_chunks;
use crate::stats::Stats;
use crate::utils::inject_built_in_plugins;
use crate::utils::log::enable_tracing_by_env;
use crate::utils::rayon::init_rayon_thread_poll;
use md4::{Digest, Md4};
pub use rspack_core::finalize::finalize;
use rspack_core::get_swc_compiler;
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
#[allow(clippy::manual_non_exhaustive)]
pub struct Bundler {
  pub options: Arc<NormalizedBundleOptions>,
  pub plugin_driver: Arc<PluginDriver>,
  pub bundle: Bundle,
  pub resolver: Arc<Resolver>,
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
    let normalized_options = Arc::new(inject_options(options));
    let injected_plugins = inject_built_in_plugins(plugins, &normalized_options);
    let (top_level_mark, unresolved_mark) =
      get_swc_compiler().run(|| (Mark::fresh(Mark::root()), Mark::fresh(Mark::root())));

    let ctx: Arc<BundleContext> = Arc::new(BundleContext::new(
      get_swc_compiler(),
      normalized_options.clone(),
      top_level_mark,
      unresolved_mark,
    ));

    let resolver = Arc::new(Resolver::new(normalized_options.resolve.clone()));

    let plugin_driver = Arc::new(PluginDriver::new(
      injected_plugins,
      ctx.clone(),
      resolver.clone(),
    ));

    Self {
      options: normalized_options.clone(),
      plugin_driver: plugin_driver.clone(),
      resolver: resolver.clone(),
      bundle: Bundle::new(normalized_options.clone(), plugin_driver, ctx, resolver),
      _noop: (),
    }
  }

  #[instrument(skip(self))]
  pub async fn build(&mut self, changed_files: Option<Vec<String>>) -> Result<Stats> {
    let start_time = std::time::Instant::now();
    self.plugin_driver.build_start().await?;
    tracing::trace!("start build");

    self.bundle.build_graph(changed_files).await?;

    let chunks = generate_chunks(&mut self.bundle)?;
    self.bundle.chunk_graph = chunks;

    let output = self.render_chunks();

    let mut map = HashMap::default();

    output.into_iter().for_each(|(_, chunk)| {
      map.insert(chunk.file_name.clone(), chunk.entry.clone());
      let mut code = chunk.code;

      match chunk.map {
        OutputChunkSourceMap::Inline(encoded_map) => {
          let comment = "\n//# sourceMappingURL=".to_owned() + &encoded_map;
          code += &comment;
        }
        OutputChunkSourceMap::External(map) => {
          let map_filename = chunk.file_name.clone() + ".map";
          self.bundle.context.assets.lock().unwrap().push(Asset {
            source: map,
            filename: map_filename,
          });
        }
        OutputChunkSourceMap::Linked(map) => {
          let map_filename = chunk.file_name.clone() + ".map";
          self.bundle.context.assets.lock().unwrap().push(Asset {
            source: map,
            filename: map_filename.clone(),
          });
          let comment = "\n//# sourceMappingURL=".to_owned() + &map_filename;
          code += &comment;
        }
        OutputChunkSourceMap::None => (),
      };

      self.bundle.context.assets.lock().unwrap().push(Asset {
        source: code,
        filename: chunk.file_name,
      });
    });

    self.plugin_driver.build_end().await?;
    let end_time = std::time::Instant::now();

    Ok(Stats {
      map,
      start_time,
      end_time,
    })
  }

  pub fn resolve(
    &mut self,
    id: String,
    dir: String,
  ) -> Result<nodejs_resolver::ResolveResult, std::string::String> {
    let base = Path::new(&dir);
    self.resolver.resolve(base, &id)
  }

  #[instrument(skip(self))]
  pub async fn rebuild(
    &mut self,
    changed_file: Vec<String>,
  ) -> Result<Vec<HashMap<String, String>>> {
    tracing::debug!("rebuild because of {:?}", changed_file);
    let changed_files = changed_file;
    let old_modules_uri = self
      .bundle
      .module_graph_container
      .module_graph
      .uris()
      .cloned()
      .filter(|id| !changed_files.contains(id))
      .collect::<HashSet<_>>();

    tracing::trace!("old_modules_id {:?}", old_modules_uri);

    self.bundle.context.assets.lock().unwrap().clear();
    changed_files.iter().for_each(|rd| {
      self
        .bundle
        .module_graph_container
        .module_graph
        .remove_by_uri(rd);
      self.bundle.visited_module_id.remove(rd);
    });

    let Stats { map, .. } = self.build(Some(changed_files)).await?;

    let new_modules_id = self
      .bundle
      .module_graph_container
      .module_graph
      .uris()
      .cloned()
      .collect::<HashSet<_>>();
    let diff_rendered = new_modules_id
      .into_iter()
      .filter(|module_id| !old_modules_uri.contains(module_id))
      .map(|module_id| {
        tracing::trace!("render new added module {:?}", module_id);
        (
          module_id.to_string(),
          self
            .bundle
            .module_graph_container
            .module_graph
            .module_by_uri(&module_id)
            .unwrap()
            .cached_output
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .code
            .clone(),
        )
      })
      .collect();
    Ok(vec![diff_rendered, map])
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
        path.push(&asset.filename);
        std::fs::create_dir_all(path.resolve().parent().unwrap()).unwrap();
        std::fs::write(path.resolve(), &asset.source).unwrap();
      });
  }

  pub fn render_chunks(&self) -> HashMap<String, OutputChunk> {
    self
      .bundle
      .chunk_graph
      .id_to_chunk()
      .par_iter()
      .map(|(_chunk_id, chunk)| {
        let mut chunk = chunk.render(&self.bundle);
        if chunk.file_name.contains("[contenthash]") {
          let mut hasher = Md4::new();
          hasher.update(&chunk.code);
          let content_hash = hasher.finalize();
          chunk.file_name = chunk
            .file_name
            .replace("[contenthash]", &format!("{:x}", content_hash));
        }

        (chunk.file_name.clone(), chunk)
      })
      .collect()
  }
}
