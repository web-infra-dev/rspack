use std::{
  collections::{HashMap, HashSet},
  path::Path,
};

use rspack_error::Result;
use rspack_sources::{RawSource, SourceExt};

use crate::{AssetInfo, Chunk, Compilation, CompilationAsset, Compiler, RenderManifestArgs, Stats};

const HOT_UPDATE_MAIN_FILENAME: &str = "hot-update.json";

fn get_hot_update_main_filename(chunk_name: &str) -> String {
  format!("{}.{}", chunk_name, HOT_UPDATE_MAIN_FILENAME)
}

#[derive(Default)]
struct HotUpdateContent {
  updated_chunk_ids: HashSet<String>,
  removed_chunk_ids: HashSet<String>,
  _removed_modules: HashSet<String>,
  // TODO: should [chunk-name].[hash].hot-update.json
  filename: String,
}

impl HotUpdateContent {
  fn new(chunk_name: &str) -> Self {
    Self {
      filename: get_hot_update_main_filename(chunk_name),
      ..Default::default()
    }
  }
}

impl Compiler {
  // TODO: remove this function when we had `record` in compiler.
  pub async fn rebuild(
    &mut self,
    changed_files: std::collections::HashSet<String>,
    removed_files: std::collections::HashSet<String>,
  ) -> Result<Stats> {
    let old = self.compilation.get_stats();
    let collect_changed_modules = |compilation: &Compilation| -> HashMap<String, String> {
      let modules = compilation.module_graph.module_graph_modules();
      // TODO: use hash;

      modules
        .filter_map(|item| {
          use crate::SourceType::*;

          compilation
            .module_graph
            .module_by_identifier(&item.module_identifier)
            .and_then(|module| {
              let resource_data = module.resource_resolved_data();
              let resource_path = &resource_data.resource_path;

              if !changed_files.contains(resource_path) && !removed_files.contains(resource_path) {
                None
              } else if item.module_type.is_js_like() {
                // TODO: it soo slowly, should use cache to instead.
                let code = module.code_generation(compilation).unwrap();
                let code = if let Some(code) = code.get(&JavaScript) {
                  code.ast_or_source.as_source().unwrap().source().to_string()
                } else {
                  println!("expect get JavaScirpt code");
                  String::new()
                };
                Some((item.module_identifier.clone(), code))
              } else if item.module_type.is_css() {
                // TODO: it soo slowly, should use cache to instead.
                let code = module.code_generation(compilation).unwrap();
                let code = if let Some(code) = code.get(&Css) {
                  // only used for compare between two build
                  code.ast_or_source.as_source().unwrap().source().to_string()
                } else {
                  println!("expect get CSS code");
                  String::new()
                };
                Some((item.module_identifier.clone(), code))
              } else {
                None
              }
            })
        })
        .collect()
    };
    let old_modules = collect_changed_modules(old.compilation);
    // TODO: should use `records`
    let all_old_runtime = old
      .compilation
      .chunk_by_ukey
      .iter()
      .map(|(_, chunk)| chunk.id.clone())
      .collect::<HashSet<_>>();
    let mut hot_update_main_content_by_runtime = all_old_runtime
      .iter()
      .map(|id| (id.clone(), HotUpdateContent::new(id)))
      .collect::<HashMap<String, HotUpdateContent>>();

    let mut old_chunks: Vec<(String, hashbrown::HashSet<String>)> = vec![];
    for (ukey, chunk) in &old.compilation.chunk_by_ukey {
      let modules = old
        .compilation
        .chunk_graph
        .get_chunk_graph_chunk(ukey)
        .modules
        .clone();
      old_chunks.push((chunk.id.clone(), modules));
    }

    // ----
    {
      // build without stats
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
    }
    // ----
    if hot_update_main_content_by_runtime.is_empty() {
      return Ok(self.stats());
    }

    let now = &mut self.compilation;

    let now_modules = collect_changed_modules(now);

    let mut updated_modules: HashMap<String, String> = Default::default();
    let mut completely_removed_modules: HashSet<String> = Default::default();

    for (old_uri, old_content) in &old_modules {
      if let Some(now_content) = now_modules.get(old_uri) {
        // updated
        if now_content != old_content {
          updated_modules.insert(old_uri.to_string(), now_content.to_string());
        }
      } else {
        // deleted
        completely_removed_modules.insert(old_uri.to_string());
      }
    }

    // ----
    let output_path = now.options.context.join(&now.options.output.path);

    // TODO: hash
    // if old.hash == now.hash { return  } else { // xxxx}

    for (chunk_id, _old_chunk_modules) in &old_chunks {
      let mut new_modules = vec![];
      let mut chunk_id = chunk_id.to_string();
      let mut _new_runtime = chunk_id.to_string();
      let mut removed_from_runtime = Some(chunk_id.to_string());
      let current_chunk = now
        .chunk_by_ukey
        .iter()
        .find(|(_, chunk)| chunk.id.eq(&chunk_id))
        .map(|(_, chunk)| chunk);

      if let Some(current_chunk) = current_chunk {
        chunk_id = current_chunk.id.to_string();
        _new_runtime = if all_old_runtime.contains(&current_chunk.id) {
          current_chunk.id.to_string()
        } else {
          continue;
        };
        new_modules = now
          .chunk_graph
          .get_chunk_graph_chunk(&current_chunk.ukey)
          .modules
          .iter()
          .filter_map(|module| {
            updated_modules
              .contains_key(module)
              .then_some(module.to_string())
          })
          .collect::<Vec<_>>();

        if current_chunk.id == chunk_id {
          removed_from_runtime = None
        }
      }

      if let Some(removed) = &removed_from_runtime {
        if let Some(info) = hot_update_main_content_by_runtime.get_mut(&chunk_id) {
          info.removed_chunk_ids.insert(removed.to_string());
        }
      }

      if !new_modules.is_empty() {
        let mut hot_update_chunk =
          Chunk::new(Some("hot_update_chunk".to_string()), chunk_id.to_string());
        let ukey = hot_update_chunk.ukey;
        if let Some(current_chunk) = current_chunk {
          current_chunk
            .groups
            .iter()
            .for_each(|group| hot_update_chunk.add_group(*group))
        }

        now.chunk_by_ukey.insert(ukey, hot_update_chunk);
        now.chunk_graph.add_chunk(ukey);

        for module in new_modules {
          now
            .chunk_graph
            .connect_chunk_and_module(ukey, module.to_string());
        }

        let render_manifest = now
          .plugin_driver
          .read()
          .await
          .render_manifest(RenderManifestArgs {
            compilation: now,
            chunk_ukey: ukey,
          })
          .unwrap();

        for entry in render_manifest {
          // remove extensions;
          let filename = Path::new(entry.filename())
            .with_extension("")
            .display()
            .to_string();
          let path = output_path.join(format!("{}.hot-update.js", filename));
          // TODO: should put into `renderManifest`.
          let source = format!(
            "self['hotUpdate'](\n'{}',\n{{ {} }})\n",
            filename,
            entry.source.source()
          );
          let asset = CompilationAsset::new(RawSource::from(source).boxed(), AssetInfo::default());

          Compiler::emit_asset(path, &asset)?;
        }

        if let Some(info) = hot_update_main_content_by_runtime.get_mut(&chunk_id) {
          info.updated_chunk_ids.insert(chunk_id.to_string());
        }
      }
    }

    let completely_removed_modules_array: Vec<String> =
      completely_removed_modules.into_iter().collect();

    for (_, content) in hot_update_main_content_by_runtime {
      let file_path = output_path.join(&content.filename);
      let c: Vec<String> = content.updated_chunk_ids.into_iter().collect();
      let r: Vec<String> = content.removed_chunk_ids.into_iter().collect();
      let m: Vec<String> = completely_removed_modules_array.clone();
      Compiler::emit_asset(
        file_path,
        &CompilationAsset::new(
          RawSource::Source(
            serde_json::json!({
              "c": c,
              "r": r,
              "m": m,
            })
            .to_string(),
          )
          .boxed(),
          AssetInfo::default(),
        ),
      )?;
    }

    Ok(self.stats())
  }
}
