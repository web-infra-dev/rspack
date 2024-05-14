pub mod api_plugin;
mod drive;
mod flag_dependency_exports_plugin;
mod flag_dependency_usage_plugin;
pub mod impl_plugin_for_js_plugin;
pub mod infer_async_modules_plugin;
pub mod inner_graph_plugin;
mod mangle_exports_plugin;
pub mod module_concatenation_plugin;
mod side_effects_flag_plugin;

use std::hash::Hash;

pub use drive::*;
pub use flag_dependency_exports_plugin::*;
pub use flag_dependency_usage_plugin::*;
pub use mangle_exports_plugin::*;
pub use module_concatenation_plugin::*;
use once_cell::sync::Lazy;
use rspack_core::rspack_sources::{BoxSource, ConcatSource, RawSource, SourceExt};
use rspack_core::{
  render_init_fragments, ChunkRenderContext, ChunkUkey, Compilation, CompilationId, RuntimeGlobals,
};
use rspack_error::Result;
use rspack_hash::RspackHash;
use rspack_hook::plugin;
use rspack_util::diff_mode::is_diff_mode;
use rspack_util::fx_hash::{BuildFxHasher, FxDashMap};
pub use side_effects_flag_plugin::*;

use crate::runtime::{render_chunk_modules, render_iife, render_runtime_modules, stringify_array};

static COMPILATION_DRIVES_MAP: Lazy<FxDashMap<CompilationId, JavascriptModulesPluginPluginDrive>> =
  Lazy::new(Default::default);

#[plugin]
#[derive(Debug, Default)]
pub struct JsPlugin;

impl JsPlugin {
  pub fn get_compilation_drives(
    compilation: &Compilation,
  ) -> dashmap::mapref::one::Ref<'_, CompilationId, JavascriptModulesPluginPluginDrive, BuildFxHasher>
  {
    let id = compilation.id();
    if !COMPILATION_DRIVES_MAP.contains_key(&id) {
      COMPILATION_DRIVES_MAP.insert(id, Default::default());
    }
    COMPILATION_DRIVES_MAP
      .get(&id)
      .expect("should have js plugin drive")
  }

  pub fn get_compilation_drives_mut(
    compilation: &Compilation,
  ) -> dashmap::mapref::one::RefMut<
    '_,
    CompilationId,
    JavascriptModulesPluginPluginDrive,
    BuildFxHasher,
  > {
    COMPILATION_DRIVES_MAP.entry(compilation.id()).or_default()
  }

  pub fn render_require(&self, chunk_ukey: &ChunkUkey, compilation: &Compilation) -> BoxSource {
    let runtime_requirements = compilation
      .chunk_graph
      .get_chunk_runtime_requirements(chunk_ukey);

    let strict_module_error_handling = compilation.options.output.strict_module_error_handling;
    let mut sources = ConcatSource::default();

    sources.add(RawSource::from(
      r#"// Check if module is in cache
        var cachedModule = __webpack_module_cache__[moduleId];
        if (cachedModule !== undefined) {
      "#,
    ));

    if strict_module_error_handling {
      sources.add(RawSource::from(
        "if (cachedModule.error !== undefined) throw cachedModule.error;",
      ));
    }

    sources.add(RawSource::from(
      r#"return cachedModule.exports;
      }
      // Create a new module (and put it into the cache)
      var module = (__webpack_module_cache__[moduleId] = {
      "#,
    ));

    if runtime_requirements.contains(RuntimeGlobals::MODULE_ID) {
      sources.add(RawSource::from("id: moduleId,\n"));
    }

    if runtime_requirements.contains(RuntimeGlobals::MODULE_LOADED) {
      sources.add(RawSource::from("loaded: false,\n"));
    }

    sources.add(RawSource::from(
      r#" exports: {}
      });
      // Execute the module function
      "#,
    ));

    let module_execution = match runtime_requirements
      .contains(RuntimeGlobals::INTERCEPT_MODULE_EXECUTION)
    {
      true => RawSource::from(
        r#"var execOptions = { id: moduleId, module: module, factory: __webpack_modules__[moduleId], require: __webpack_require__ };
            __webpack_require__.i.forEach(function(handler) { handler(execOptions); });
            module = execOptions.module;
            if (!execOptions.factory) {
              console.error("undefined factory", moduleId)
            }
            execOptions.factory.call(module.exports, module, module.exports, execOptions.require);
            "#,
      ),
      false => {
        if runtime_requirements.contains(RuntimeGlobals::THIS_AS_EXPORTS) {
          RawSource::from(
            "__webpack_modules__[moduleId].call(module.exports, module, module.exports, __webpack_require__);\n",
          )
        } else {
          RawSource::from(
            "__webpack_modules__[moduleId](module, module.exports, __webpack_require__);\n",
          )
        }
      }
    };

    if strict_module_error_handling {
      sources.add(RawSource::from("try {\n"));
      sources.add(module_execution);
      sources.add(RawSource::from(
        r#"} catch (e) {
            module.error = e;
            throw e;
          }
          "#,
      ));
    } else {
      sources.add(module_execution);
    }

    if runtime_requirements.contains(RuntimeGlobals::MODULE_LOADED) {
      sources.add(RawSource::from(
        "// Flag the module as loaded \n module.loaded = true;\n",
      ));
    }

    sources.add(RawSource::from(
      "// Return the exports of the module\n return module.exports;\n",
    ));

    sources.boxed()
  }

  pub fn render_bootstrap(
    &self,
    chunk_ukey: &ChunkUkey,
    compilation: &Compilation,
  ) -> (BoxSource, BoxSource) {
    let runtime_requirements = compilation
      .chunk_graph
      .get_chunk_runtime_requirements(chunk_ukey);
    let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);
    let module_factories = runtime_requirements.contains(RuntimeGlobals::MODULE_FACTORIES);
    // let require_function = runtime_requirements.contains(RuntimeGlobals::REQUIRE);
    let intercept_module_execution =
      runtime_requirements.contains(RuntimeGlobals::INTERCEPT_MODULE_EXECUTION);
    // let module_used = runtime_requirements.contains(RuntimeGlobals::MODULE);
    // let use_require = require_function || intercept_module_execution || module_used;
    let mut header = ConcatSource::default();

    if is_diff_mode() {
      header.add(RawSource::from(
        "\n/************************************************************************/\n",
      ));
    }

    header.add(RawSource::from(
      "// The module cache\n var __webpack_module_cache__ = {};\n",
    ));
    header.add(RawSource::from(
      "function __webpack_require__(moduleId) {\n",
    ));
    header.add(self.render_require(chunk_ukey, compilation));
    header.add(RawSource::from("\n}\n"));

    if module_factories || runtime_requirements.contains(RuntimeGlobals::MODULE_FACTORIES_ADD_ONLY)
    {
      header.add(RawSource::from(
        "// expose the modules object (__webpack_modules__)\n __webpack_require__.m = __webpack_modules__;\n",
      ));
    }

    if runtime_requirements.contains(RuntimeGlobals::MODULE_CACHE) {
      header.add(RawSource::from(
        "// expose the module cache\n __webpack_require__.c = __webpack_module_cache__;\n",
      ));
    }

    if intercept_module_execution {
      header.add(RawSource::from(
        "// expose the module execution interceptor\n __webpack_require__.i = [];\n",
      ));
    }

    let mut startup = vec![];

    if !runtime_requirements.contains(RuntimeGlobals::STARTUP_NO_DEFAULT) {
      if chunk.has_entry_module(&compilation.chunk_graph) {
        let entries = compilation
          .chunk_graph
          .get_chunk_entry_modules_with_chunk_group_iterable(chunk_ukey);
        for (i, (module, entry)) in entries.iter().enumerate() {
          let chunk_group = compilation.chunk_group_by_ukey.expect_get(entry);
          let chunk_ids = chunk_group
            .chunks
            .iter()
            .filter(|c| *c != chunk_ukey)
            .map(|chunk_ukey| {
              compilation
                .chunk_by_ukey
                .expect_get(chunk_ukey)
                .expect_id()
                .to_string()
            })
            .collect::<Vec<_>>();
          let module_id = compilation
            .get_module_graph()
            .module_graph_module_by_identifier(module)
            .map(|module| module.id(&compilation.chunk_graph))
            .expect("should have module id");
          let mut module_id_expr = serde_json::to_string(module_id).expect("invalid module_id");
          if runtime_requirements.contains(RuntimeGlobals::ENTRY_MODULE_ID) {
            module_id_expr = format!("{} = {module_id_expr}", RuntimeGlobals::ENTRY_MODULE_ID);
          }

          if !chunk_ids.is_empty() {
            startup.push(format!(
              "{}{}(undefined, {} , function() {{ return __webpack_require__({module_id_expr}) }});",
              if i + 1 == entries.len() {
                "var __webpack_exports__ = "
              } else {
                ""
              },
              RuntimeGlobals::ON_CHUNKS_LOADED,
              stringify_array(&chunk_ids)
            ));
          }
          /* if use_require */
          else {
            startup.push(format!(
              "{}__webpack_require__({module_id_expr});",
              if i + 1 == entries.len() {
                "var __webpack_exports__ = "
              } else {
                ""
              },
            ))
          }
          // else {
          //   startup.push(format!("__webpack_modules__[{module_id_expr}]();"))
          // }
        }
        if runtime_requirements.contains(RuntimeGlobals::ON_CHUNKS_LOADED) {
          startup.push(format!(
            "__webpack_exports__ = {}(__webpack_exports__);",
            RuntimeGlobals::ON_CHUNKS_LOADED
          ));
        }
        if runtime_requirements.contains(RuntimeGlobals::STARTUP) {
          header.add(RawSource::from(format!(
            r#"//  the startup function
            {} = function(){{
              {}
              return __webpack_exports__;
            }};
          "#,
            RuntimeGlobals::STARTUP,
            std::mem::take(&mut startup).join("\n")
          )));
          startup.push("// run startup".to_string());
          startup.push(format!(
            "var __webpack_exports__ = {}();",
            RuntimeGlobals::STARTUP
          ));
        }
      } else if runtime_requirements.contains(RuntimeGlobals::STARTUP) {
        header.add(RawSource::from(format!(
          r#"// the startup function
          // It's empty as no entry modules are in this chunk
            {} = function(){{}};
          "#,
          RuntimeGlobals::STARTUP
        )));
      }
    } else if runtime_requirements.contains(RuntimeGlobals::STARTUP) {
      header.add(RawSource::from(format!(
        r#"// the startup function
        // It's empty as some runtime module handles the default behavior
          {} = function(){{}};
        "#,
        RuntimeGlobals::STARTUP
      )));
      startup.push("// run startup".to_string());
      startup.push(format!(
        "var __webpack_exports__ = {}();",
        RuntimeGlobals::STARTUP
      ));
    }
    if is_diff_mode() {
      header.add(RawSource::from(
        "\n/************************************************************************/\n",
      ));
    }
    (header.boxed(), RawSource::from(startup.join("\n")).boxed())
  }

  pub async fn render_main(
    &self,
    compilation: &Compilation,
    chunk_ukey: &ChunkUkey,
  ) -> Result<BoxSource> {
    let drive = Self::get_compilation_drives(compilation);
    let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);
    let runtime_requirements = compilation
      .chunk_graph
      .get_tree_runtime_requirements(chunk_ukey);
    let (module_source, chunk_init_fragments) = render_chunk_modules(compilation, chunk_ukey)?;
    let (header, startup) = self.render_bootstrap(chunk_ukey, compilation);
    let mut sources = ConcatSource::default();
    sources.add(RawSource::from("var __webpack_modules__ = "));
    sources.add(module_source);
    sources.add(RawSource::from("\n"));
    sources.add(header);
    sources.add(render_runtime_modules(compilation, chunk_ukey)?);
    if chunk.has_entry_module(&compilation.chunk_graph) {
      let last_entry_module = compilation
        .chunk_graph
        .get_chunk_entry_modules_with_chunk_group_iterable(chunk_ukey)
        .keys()
        .last()
        .expect("should have last entry module");
      if let Some(source) = drive.render_startup(RenderJsStartupArgs {
        compilation,
        chunk: chunk_ukey,
        module: *last_entry_module,
        source: startup,
      })? {
        sources.add(source);
      }
      if runtime_requirements.contains(RuntimeGlobals::RETURN_EXPORTS_FROM_RUNTIME) {
        sources.add(RawSource::from("return __webpack_exports__;\n"));
      }
    }
    let mut final_source = if compilation.options.output.iife {
      render_iife(sources.boxed())
    } else {
      sources.boxed()
    };
    final_source = render_init_fragments(
      final_source,
      chunk_init_fragments,
      &mut ChunkRenderContext {},
    )?;
    if let Some(source) = drive.render(RenderJsArgs {
      compilation,
      chunk: chunk_ukey,
      source: &final_source,
    })? {
      return Ok(source);
    }
    Ok(final_source)
  }

  #[inline]
  pub async fn render_chunk_impl(
    &self,
    compilation: &Compilation,
    chunk_ukey: &ChunkUkey,
  ) -> Result<BoxSource> {
    let drive = Self::get_compilation_drives(compilation);
    let (module_source, chunk_init_fragments) = render_chunk_modules(compilation, chunk_ukey)?;
    let source = drive
      .render_chunk(RenderJsChunkArgs {
        compilation,
        chunk_ukey,
        module_source,
      })
      .await?
      .expect("should run render_chunk hook");
    let source_with_fragments =
      render_init_fragments(source, chunk_init_fragments, &mut ChunkRenderContext {})?;
    Ok(
      ConcatSource::new([
        if let Some(source) = drive.render(RenderJsArgs {
          compilation,
          chunk: chunk_ukey,
          source: &source_with_fragments,
        })? {
          source
        } else {
          source_with_fragments
        },
        RawSource::from(";").boxed(),
      ])
      .boxed(),
    )
  }

  #[inline]
  pub fn get_chunk_hash(
    &self,
    chunk_ukey: &ChunkUkey,
    compilation: &Compilation,
    hasher: &mut RspackHash,
  ) -> PluginJsChunkHashHookOutput {
    let drive = Self::get_compilation_drives(compilation);
    drive.js_chunk_hash(JsChunkHashArgs {
      compilation,
      chunk_ukey,
      hasher,
    })
  }

  #[inline]
  pub fn update_hash_with_bootstrap(
    &self,
    chunk_ukey: &ChunkUkey,
    compilation: &Compilation,
    hasher: &mut RspackHash,
  ) {
    // sample hash use content
    let (header, startup) = self.render_bootstrap(chunk_ukey, compilation);
    header.hash(hasher);
    startup.hash(hasher);
  }
}

#[derive(Debug, Clone)]
pub struct ExtractedCommentsInfo {
  pub source: BoxSource,
  pub comments_file_name: String,
}
