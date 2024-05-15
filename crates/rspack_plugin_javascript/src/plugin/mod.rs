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

use std::borrow::Cow;
use std::hash::Hash;

pub use drive::*;
pub use flag_dependency_exports_plugin::*;
pub use flag_dependency_usage_plugin::*;
use indoc::indoc;
pub use mangle_exports_plugin::*;
pub use module_concatenation_plugin::*;
use once_cell::sync::Lazy;
use rspack_core::rspack_sources::{BoxSource, ConcatSource, RawSource, SourceExt};
use rspack_core::{
  basic_function, render_init_fragments, ChunkRenderContext, ChunkUkey, Compilation, CompilationId,
  RuntimeGlobals,
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

  pub fn render_require(&self, chunk_ukey: &ChunkUkey, compilation: &Compilation) -> String {
    let runtime_requirements = compilation
      .chunk_graph
      .get_chunk_runtime_requirements(chunk_ukey);

    let strict_module_error_handling = compilation.options.output.strict_module_error_handling;
    let mut sources: Vec<Cow<str>> = Vec::new();

    sources.push(
      indoc! {r#"
        // Check if module is in cache
        var cachedModule = __webpack_module_cache__[moduleId];
        if (cachedModule !== undefined) {
      "#}
      .into(),
    );

    if strict_module_error_handling {
      sources.push("if (cachedModule.error !== undefined) throw cachedModule.error;".into());
    }

    sources.push(
      indoc! {r#"
        return cachedModule.exports;
        }
        // Create a new module (and put it into the cache)
        var module = (__webpack_module_cache__[moduleId] = {
      "#}
      .into(),
    );

    if runtime_requirements.contains(RuntimeGlobals::MODULE_ID) {
      sources.push("id: moduleId,\n".into());
    }

    if runtime_requirements.contains(RuntimeGlobals::MODULE_LOADED) {
      sources.push("loaded: false,\n".into());
    }

    sources.push("exports: {}\n".into());
    sources.push(
      indoc! {r#"
        });
        // Execute the module function
      "#}
      .into(),
    );

    let module_execution = if runtime_requirements
      .contains(RuntimeGlobals::INTERCEPT_MODULE_EXECUTION)
    {
      indoc!{r#"
        var execOptions = { id: moduleId, module: module, factory: __webpack_modules__[moduleId], require: __webpack_require__ };
        __webpack_require__.i.forEach(function(handler) { handler(execOptions); });
        module = execOptions.module;
        if (!execOptions.factory) {
          console.error("undefined factory", moduleId)
        }
        execOptions.factory.call(module.exports, module, module.exports, execOptions.require);
      "#}.into()
    } else if runtime_requirements.contains(RuntimeGlobals::THIS_AS_EXPORTS) {
      "__webpack_modules__[moduleId].call(module.exports, module, module.exports, __webpack_require__);\n".into()
    } else {
      "__webpack_modules__[moduleId](module, module.exports, __webpack_require__);\n".into()
    };

    if strict_module_error_handling {
      sources.push("try {\n".into());
      sources.push(module_execution);
      sources.push("} catch (e) {".into());
      sources.push("module.error = e;\nthrow e;".into());
      sources.push("}".into());
    } else {
      sources.push(module_execution);
    }

    if runtime_requirements.contains(RuntimeGlobals::MODULE_LOADED) {
      sources.push("// Flag the module as loaded\nmodule.loaded = true;".into());
    }

    sources.push("// Return the exports of the module\nreturn module.exports;".into());

    sources.join("")
  }

  pub fn render_bootstrap(
    &self,
    chunk_ukey: &ChunkUkey,
    compilation: &Compilation,
  ) -> RenderBootstrapResult {
    let runtime_requirements = compilation
      .chunk_graph
      .get_chunk_runtime_requirements(chunk_ukey);
    let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);
    let module_factories = runtime_requirements.contains(RuntimeGlobals::MODULE_FACTORIES);
    let require_function = runtime_requirements.contains(RuntimeGlobals::REQUIRE);
    let module_cache = runtime_requirements.contains(RuntimeGlobals::MODULE_CACHE);
    let intercept_module_execution =
      runtime_requirements.contains(RuntimeGlobals::INTERCEPT_MODULE_EXECUTION);
    let module_used = runtime_requirements.contains(RuntimeGlobals::MODULE);
    let require_scope_used = runtime_requirements.contains(RuntimeGlobals::REQUIRE_SCOPE);
    let use_require = require_function || intercept_module_execution || module_used;
    let mut header: Vec<Cow<str>> = Vec::new();
    let mut startup: Vec<Cow<str>> = Vec::new();
    let mut allow_inline_startup = true;

    if is_diff_mode() {
      header.push(
        "\n/************************************************************************/\n".into(),
      );
    }

    if allow_inline_startup && module_factories {
      startup.push("// module factories are used so entry inlining is disabled".into());
      allow_inline_startup = false;
    }
    if allow_inline_startup && module_cache {
      startup.push("// module cache are used so entry inlining is disabled".into());
      allow_inline_startup = false;
    }
    if allow_inline_startup && intercept_module_execution {
      startup.push("// module execution is intercepted so entry inlining is disabled".into());
      allow_inline_startup = false;
    }

    if use_require || module_cache {
      header.push("// The module cache\nvar __webpack_module_cache__ = {};\n".into());
    }

    if use_require {
      header.push(
        format!(
          "// The require function\nfunction {}(moduleId) {{\n",
          RuntimeGlobals::REQUIRE
        )
        .into(),
      );
      header.push(self.render_require(chunk_ukey, compilation).into());
      header.push("\n}\n".into());
    } else if require_scope_used {
      header.push(
        format!(
          "// The require scope\nvar {} = {{}};\n",
          RuntimeGlobals::REQUIRE
        )
        .into(),
      );
    }

    if module_factories || runtime_requirements.contains(RuntimeGlobals::MODULE_FACTORIES_ADD_ONLY)
    {
      header.push(
        format!(
          "// expose the modules object (__webpack_modules__)\n{} = __webpack_modules__;\n",
          RuntimeGlobals::MODULE_FACTORIES
        )
        .into(),
      );
    }

    if runtime_requirements.contains(RuntimeGlobals::MODULE_CACHE) {
      header.push(
        format!(
          "// expose the module cache\n{} = __webpack_module_cache__;\n",
          RuntimeGlobals::MODULE_CACHE
        )
        .into(),
      );
    }

    if intercept_module_execution {
      header.push(
        format!(
          "// expose the module execution interceptor\n{} = [];\n",
          RuntimeGlobals::INTERCEPT_MODULE_EXECUTION
        )
        .into(),
      );
    }

    if !runtime_requirements.contains(RuntimeGlobals::STARTUP_NO_DEFAULT) {
      if chunk.has_entry_module(&compilation.chunk_graph) {
        let mut buf2: Vec<Cow<str>> = Vec::new();
        buf2.push("// Load entry module and return exports".into());
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
          if allow_inline_startup && !chunk_ids.is_empty() {
            buf2.push("// This entry module depends on other loaded chunks and execution need to be delayed".into());
            allow_inline_startup = false;
          }
          if allow_inline_startup && {
            let module_graph = compilation.get_module_graph();
            module_graph
              .get_incoming_connections_by_origin_module(module)
              .iter()
              .any(|(origin_module, connections)| {
                if let Some(origin_module) = origin_module {
                  connections
                    .iter()
                    .any(|c| c.is_target_active(&module_graph, Some(&chunk.runtime)))
                    && compilation
                      .chunk_graph
                      .get_module_runtimes(*origin_module, &compilation.chunk_by_ukey)
                      .into_values()
                      .any(|runtime| runtime.intersection(&chunk.runtime).count() > 0)
                } else {
                  false
                }
              })
          } {
            buf2.push(
              "// This entry module is referenced by other modules so it can't be inlined".into(),
            );
            allow_inline_startup = false;
          }
          if allow_inline_startup {
            // TODO: topLevelDeclarations and inlineInRuntimeBailout
            allow_inline_startup = false;
          }
          let entry_runtime_requirements = compilation
            .chunk_graph
            .get_module_runtime_requirements(*module, &chunk.runtime);
          if allow_inline_startup
            && let Some(entry_runtime_requirements) = entry_runtime_requirements
            && entry_runtime_requirements.contains(RuntimeGlobals::MODULE)
          {
            allow_inline_startup = false;
            buf2.push("// This entry module used 'module' so it can't be inlined".into());
          }

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
            buf2.push(
              format!(
                "{}{}(undefined, {}, function() {{ return {}({module_id_expr}) }});",
                if i + 1 == entries.len() {
                  format!("var {} = ", RuntimeGlobals::EXPORTS)
                } else {
                  "".to_string()
                },
                RuntimeGlobals::ON_CHUNKS_LOADED,
                stringify_array(&chunk_ids),
                RuntimeGlobals::REQUIRE
              )
              .into(),
            );
          } else if use_require {
            buf2.push(
              format!(
                "{}{}({module_id_expr});",
                if i + 1 == entries.len() {
                  format!("var {} = ", RuntimeGlobals::EXPORTS)
                } else {
                  "".to_string()
                },
                RuntimeGlobals::REQUIRE
              )
              .into(),
            )
          } else {
            let should_exec = i + 1 == entries.len();
            if should_exec {
              buf2.push(format!("var {} = {{}}", RuntimeGlobals::EXPORTS).into());
            }
            if require_scope_used {
              buf2.push(
                format!(
                  "__webpack_modules__[{module_id_expr}](0, {}, {});",
                  if should_exec {
                    RuntimeGlobals::EXPORTS.name()
                  } else {
                    "{}"
                  },
                  RuntimeGlobals::REQUIRE
                )
                .into(),
              );
            } else if let Some(entry_runtime_requirements) = entry_runtime_requirements
              && entry_runtime_requirements.contains(RuntimeGlobals::EXPORTS)
            {
              buf2.push(
                format!(
                  "__webpack_modules__[{module_id_expr}](0, {});",
                  if should_exec {
                    RuntimeGlobals::EXPORTS.name()
                  } else {
                    "{}"
                  }
                )
                .into(),
              );
            } else {
              buf2.push(format!("__webpack_modules__[{module_id_expr}]();").into());
            }
          }
        }
        if runtime_requirements.contains(RuntimeGlobals::ON_CHUNKS_LOADED) {
          buf2.push(
            format!(
              "__webpack_exports__ = {}(__webpack_exports__);",
              RuntimeGlobals::ON_CHUNKS_LOADED
            )
            .into(),
          );
        }
        if runtime_requirements.contains(RuntimeGlobals::STARTUP) {
          allow_inline_startup = false;
          header.push(
            format!(
              "// the startup function\n{} = {}\n",
              RuntimeGlobals::STARTUP,
              basic_function(
                "",
                &format!("{}\nreturn {}", buf2.join("\n"), RuntimeGlobals::EXPORTS)
              )
            )
            .into(),
          );
          startup.push("// run startup".into());
          startup.push(
            format!(
              "var {} = {}();",
              RuntimeGlobals::EXPORTS,
              RuntimeGlobals::STARTUP
            )
            .into(),
          );
        } else {
          startup.push("// startup".into());
          startup.push(buf2.join("\n").into());
        }
      } else if runtime_requirements.contains(RuntimeGlobals::STARTUP) {
        header.push(
          format!(
            "// the startup function\n// It's empty as no entry modules are in this chunk\n{} = function(){{}};",
            RuntimeGlobals::STARTUP
          )
          .into(),
        );
      }
    } else if runtime_requirements.contains(RuntimeGlobals::STARTUP) {
      header.push(
        format!(
          "// the startup function\n// It's empty as some runtime module handles the default behavior\n{} = function(){{}};",
          RuntimeGlobals::STARTUP
        )
        .into(),
      );
      startup.push("// run startup".into());
      startup.push(format!("var __webpack_exports__ = {}();", RuntimeGlobals::STARTUP).into());
    }
    if is_diff_mode() {
      header.push(
        "\n/************************************************************************/\n".into(),
      );
    }
    RenderBootstrapResult {
      header: RawSource::from(header.join("\n")).boxed(),
      startup: RawSource::from(startup.join("\n")).boxed(),
      allow_inline_startup,
    }
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
    let RenderBootstrapResult {
      header,
      startup,
      allow_inline_startup: _,
    } = self.render_bootstrap(chunk_ukey, compilation);
    let (module_source, chunk_init_fragments) = render_chunk_modules(compilation, chunk_ukey)?;
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
    let RenderBootstrapResult {
      header,
      startup,
      allow_inline_startup,
    } = self.render_bootstrap(chunk_ukey, compilation);
    header.hash(hasher);
    startup.hash(hasher);
    allow_inline_startup.hash(hasher);
  }
}

#[derive(Debug, Clone)]
pub struct ExtractedCommentsInfo {
  pub source: BoxSource,
  pub comments_file_name: String,
}

#[derive(Debug)]
pub struct RenderBootstrapResult {
  header: BoxSource,
  startup: BoxSource,
  allow_inline_startup: bool,
}
