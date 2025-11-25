use std::{
  borrow::Cow,
  collections::{HashMap, HashSet, hash_map::Entry},
  hash::Hash,
  ops::Deref,
  sync::{Arc, LazyLock, RwLock as SyncRwLock},
};

use rayon::prelude::*;
pub mod api_plugin;
mod drive;
mod flag_dependency_exports_plugin;
mod flag_dependency_usage_plugin;
pub mod impl_plugin_for_js_plugin;
pub mod infer_async_modules_plugin;
mod inline_exports_plugin;
mod mangle_exports_plugin;
pub mod module_concatenation_plugin;
mod side_effects_flag_plugin;
pub mod url_plugin;

pub use drive::*;
pub use flag_dependency_exports_plugin::*;
pub use flag_dependency_usage_plugin::*;
use indoc::indoc;
pub use inline_exports_plugin::*;
pub use mangle_exports_plugin::*;
pub use module_concatenation_plugin::*;
use rspack_collections::{Identifier, IdentifierDashMap, IdentifierLinkedMap, IdentifierMap};
use rspack_core::{
  ChunkGraph, ChunkGroupUkey, ChunkInitFragments, ChunkRenderContext, ChunkUkey,
  CodeGenerationDataTopLevelDeclarations, Compilation, CompilationId, ConcatenatedModuleIdent,
  ExportsArgument, IdentCollector, Module, RuntimeGlobals, SourceType,
  concatenated_module::find_new_name,
  render_init_fragments,
  reserved_names::RESERVED_NAMES,
  rspack_sources::{BoxSource, ConcatSource, RawStringSource, ReplaceSource, Source, SourceExt},
  split_readable_identifier,
};
use rspack_error::{Result, ToStringResultToRspackResultExt};
use rspack_hash::{RspackHash, RspackHashDigest};
use rspack_hook::plugin;
use rspack_javascript_compiler::ast::Ast;
use rspack_util::SpanExt;
#[cfg(allocative)]
use rspack_util::allocative;
use rustc_hash::FxHashMap;
pub use side_effects_flag_plugin::*;
use swc_core::{
  atoms::Atom,
  common::{FileName, Spanned, SyntaxContext},
  ecma::transforms::base::resolver,
};
use tokio::sync::RwLock;

use crate::runtime::{
  render_chunk_modules, render_module, render_runtime_modules, stringify_array,
};

#[cfg_attr(allocative, allocative::root)]
static COMPILATION_HOOKS_MAP: LazyLock<
  SyncRwLock<FxHashMap<CompilationId, Arc<RwLock<JavascriptModulesPluginHooks>>>>,
> = LazyLock::new(Default::default);

#[derive(Debug, Clone)]
struct WithHash<T> {
  hash: Option<RspackHashDigest>,
  value: T,
}

#[derive(Debug, Default)]
struct RenameModuleCache {
  inlined_modules_to_info: IdentifierDashMap<Arc<WithHash<InlinedModuleInfo>>>,
  non_inlined_modules_through_idents:
    IdentifierDashMap<Arc<WithHash<Vec<ConcatenatedModuleIdent>>>>,
}

impl RenameModuleCache {
  pub fn get_inlined_info(&self, ident: &Identifier) -> Option<Arc<WithHash<InlinedModuleInfo>>> {
    self
      .inlined_modules_to_info
      .get(ident)
      .map(|info| info.clone())
  }

  pub fn get_non_inlined_idents(
    &self,
    ident: &Identifier,
  ) -> Option<Arc<WithHash<Vec<ConcatenatedModuleIdent>>>> {
    self
      .non_inlined_modules_through_idents
      .get(ident)
      .map(|idents| idents.clone())
  }
}

#[derive(Debug, Clone)]
struct InlinedModuleInfo {
  source: Arc<dyn Source>,
  module_scope_idents: Vec<Arc<ConcatenatedModuleIdent>>,
  used_in_non_inlined: Vec<Arc<ConcatenatedModuleIdent>>,
}

#[derive(Debug)]
struct RenameInfoPatch {
  inlined_modules_to_info: IdentifierMap<InlinedModuleInfo>,
  non_inlined_module_through_idents: Vec<ConcatenatedModuleIdent>,
  all_used_names: HashSet<Atom>,
}

#[plugin]
#[derive(Debug, Default)]
pub struct JsPlugin {
  rename_module_cache: RenameModuleCache,
}

impl JsPlugin {
  pub fn get_compilation_hooks(id: CompilationId) -> Arc<RwLock<JavascriptModulesPluginHooks>> {
    COMPILATION_HOOKS_MAP
      .read()
      .expect("should have js plugin drive")
      .get(&id)
      .expect("should have js plugin drive")
      .clone()
  }

  pub fn get_compilation_hooks_mut(id: CompilationId) -> Arc<RwLock<JavascriptModulesPluginHooks>> {
    COMPILATION_HOOKS_MAP
      .write()
      .expect("should have js plugin drive")
      .entry(id)
      .or_default()
      .clone()
  }

  pub fn render_require<'me>(
    chunk_ukey: &ChunkUkey,
    compilation: &'me Compilation,
  ) -> Vec<Cow<'me, str>> {
    let runtime_requirements = ChunkGraph::get_chunk_runtime_requirements(compilation, chunk_ukey);

    let strict_module_error_handling = compilation.options.output.strict_module_error_handling;
    let need_module_defer =
      runtime_requirements.contains(RuntimeGlobals::MAKE_DEFERRED_NAMESPACE_OBJECT);
    let mut sources: Vec<Cow<str>> = Vec::new();

    sources.push(
      indoc! {r#"
        // Check if module is in cache
        var cachedModule = __webpack_module_cache__[moduleId];
        if (cachedModule !== undefined) {"#}
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
        var module = (__webpack_module_cache__[moduleId] = {"#}
      .into(),
    );

    if runtime_requirements.contains(RuntimeGlobals::MODULE_ID) {
      sources.push("id: moduleId,".into());
    }

    if runtime_requirements.contains(RuntimeGlobals::MODULE_LOADED) {
      sources.push("loaded: false,".into());
    }

    if need_module_defer {
      sources.push("exports: __webpack_module_deferred_exports__[moduleId] || {}".into());
    } else {
      sources.push("exports: {}".into());
    }
    sources.push("});\n// Execute the module function".into());

    let module_execution = if runtime_requirements
      .contains(RuntimeGlobals::INTERCEPT_MODULE_EXECUTION)
    {
      format!(r#"
        var execOptions = {{ id: moduleId, module: module, factory: __webpack_modules__[moduleId], require: {} }};
        {}.forEach(function(handler) {{ handler(execOptions); }});
        module = execOptions.module;
        if (!execOptions.factory) {{
          console.error("undefined factory", moduleId);
          throw Error("RuntimeError: factory is undefined (" + moduleId + ")");
        }}
        execOptions.factory.call(module.exports, module, module.exports, execOptions.require);
      "#,
        compilation.runtime_template.render_runtime_globals(&RuntimeGlobals::REQUIRE),
        compilation.runtime_template.render_runtime_globals(&RuntimeGlobals::INTERCEPT_MODULE_EXECUTION)
      ).into()
    } else if runtime_requirements.contains(RuntimeGlobals::THIS_AS_EXPORTS) {
      format!(
        "__webpack_modules__[moduleId].call(module.exports, module, module.exports, {});\n",
        compilation
          .runtime_template
          .render_runtime_globals(&RuntimeGlobals::REQUIRE)
      )
      .into()
    } else {
      format!(
        "__webpack_modules__[moduleId](module, module.exports, {});\n",
        compilation
          .runtime_template
          .render_runtime_globals(&RuntimeGlobals::REQUIRE)
      )
      .into()
    };

    if strict_module_error_handling {
      sources.push("try {\n".into());
      sources.push(module_execution);
      sources.push("} catch (e) {".into());
      if need_module_defer {
        sources.push("delete __webpack_module_deferred_exports__[moduleId];".into());
      }
      sources.push("module.error = e;\nthrow e;".into());
      sources.push("}".into());
    } else {
      sources.push(module_execution);
      if need_module_defer {
        sources.push("delete __webpack_module_deferred_exports__[moduleId];".into());
      }
    }

    if runtime_requirements.contains(RuntimeGlobals::MODULE_LOADED) {
      sources.push("// Flag the module as loaded\nmodule.loaded = true;".into());
    }

    sources.push("// Return the exports of the module\nreturn module.exports;".into());

    sources
  }

  pub async fn render_bootstrap<'me>(
    chunk_ukey: &ChunkUkey,
    compilation: &'me Compilation,
  ) -> Result<RenderBootstrapResult<'me>> {
    let runtime_requirements = ChunkGraph::get_chunk_runtime_requirements(compilation, chunk_ukey);
    let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);
    let runtime_template = &compilation.runtime_template;
    let module_factories = runtime_requirements.contains(RuntimeGlobals::MODULE_FACTORIES);
    let require_function = runtime_requirements.contains(RuntimeGlobals::REQUIRE);
    let module_cache = runtime_requirements.contains(RuntimeGlobals::MODULE_CACHE);
    let intercept_module_execution =
      runtime_requirements.contains(RuntimeGlobals::INTERCEPT_MODULE_EXECUTION);
    let module_used = runtime_requirements.contains(RuntimeGlobals::MODULE);
    let require_scope_used = runtime_requirements.contains(RuntimeGlobals::REQUIRE_SCOPE);
    let need_module_defer =
      runtime_requirements.contains(RuntimeGlobals::MAKE_DEFERRED_NAMESPACE_OBJECT);
    let use_require = require_function || intercept_module_execution || module_used;
    let mut header: Vec<Cow<str>> = Vec::new();
    let mut startup: Vec<Cow<str>> = Vec::new();
    let mut allow_inline_startup = true;
    let supports_arrow_function = compilation
      .options
      .output
      .environment
      .supports_arrow_function();

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

    if need_module_defer {
      // in order to optimize of DeferredNamespaceObject, we remove all proxy handlers after the module initialize
      // (see MakeDeferredNamespaceObjectRuntimeModule)
      // This requires all deferred imports to a module can get the module export object before the module
      // is evaluated.
      header.push(
        "// The deferred module cache\nvar __webpack_module_deferred_exports__ = {};\n".into(),
      );
    }

    if use_require {
      header.push(
        format!(
          "// The require function\nfunction {}(moduleId) {{\n",
          runtime_template.render_runtime_globals(&RuntimeGlobals::REQUIRE)
        )
        .into(),
      );
      header.extend(Self::render_require(chunk_ukey, compilation));
      header.push("\n}\n".into());
    } else if require_scope_used {
      header.push(
        format!(
          "// The require scope\nvar {} = {{}};\n",
          runtime_template.render_runtime_globals(&RuntimeGlobals::REQUIRE)
        )
        .into(),
      );
    }

    if module_factories || runtime_requirements.contains(RuntimeGlobals::MODULE_FACTORIES_ADD_ONLY)
    {
      header.push(
        format!(
          "// expose the modules object (__webpack_modules__)\n{} = __webpack_modules__;\n",
          runtime_template.render_runtime_globals(&RuntimeGlobals::MODULE_FACTORIES)
        )
        .into(),
      );
    }

    if runtime_requirements.contains(RuntimeGlobals::MODULE_CACHE) {
      header.push(
        format!(
          "// expose the module cache\n{} = __webpack_module_cache__;\n",
          runtime_template.render_runtime_globals(&RuntimeGlobals::MODULE_CACHE)
        )
        .into(),
      );
    }

    if intercept_module_execution {
      header.push(
        format!(
          "// expose the module execution interceptor\n{} = [];\n",
          runtime_template.render_runtime_globals(&RuntimeGlobals::INTERCEPT_MODULE_EXECUTION)
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
                .expect_id(&compilation.chunk_ids_artifact)
                .to_string()
            })
            .collect::<Vec<_>>();
          if allow_inline_startup && !chunk_ids.is_empty() {
            buf2.push("// This entry module depends on other loaded chunks and execution need to be delayed".into());
            allow_inline_startup = false;
          }
          if allow_inline_startup && {
            let module_graph = compilation.get_module_graph();
            let module_graph_cache = &compilation.module_graph_cache_artifact;
            module_graph
              .get_incoming_connections_by_origin_module(module)
              .iter()
              .any(|(origin_module, connections)| {
                if let Some(origin_module) = origin_module {
                  connections.iter().any(|c| {
                    c.is_target_active(&module_graph, Some(chunk.runtime()), module_graph_cache)
                  }) && compilation
                    .chunk_graph
                    .get_module_runtimes_iter(*origin_module, &compilation.chunk_by_ukey)
                    .any(|runtime| runtime.intersection(chunk.runtime()).count() > 0)
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
          if allow_inline_startup && {
            let codegen = compilation
              .code_generation_results
              .get(module, Some(chunk.runtime()));
            let module_graph = compilation.get_module_graph();
            let top_level_decls = codegen
              .data
              .get::<CodeGenerationDataTopLevelDeclarations>()
              .map(|d| d.inner())
              .or_else(|| {
                module_graph
                  .module_by_identifier(module)
                  .and_then(|m| m.build_info().top_level_declarations.as_ref())
              });
            top_level_decls.is_none()
          } {
            buf2.push("// This entry module doesn't tell about it's top-level declarations so it can't be inlined".into());
            allow_inline_startup = false;
          }
          let hooks = JsPlugin::get_compilation_hooks(compilation.id());
          let bailout = hooks
            .try_read()
            .expect("should have js plugin drive")
            .inline_in_runtime_bailout
            .call(compilation)
            .await?;
          if allow_inline_startup && let Some(bailout) = bailout {
            buf2.push(format!("// This entry module can't be inlined because {bailout}").into());
            allow_inline_startup = false;
          }
          let entry_runtime_requirements =
            ChunkGraph::get_module_runtime_requirements(compilation, *module, chunk.runtime());
          if allow_inline_startup
            && let Some(entry_runtime_requirements) = entry_runtime_requirements
            && entry_runtime_requirements.contains(RuntimeGlobals::MODULE)
          {
            allow_inline_startup = false;
            buf2.push("// This entry module used 'module' so it can't be inlined".into());
          }

          let module_id = ChunkGraph::get_module_id(&compilation.module_ids_artifact, *module)
            .expect("should have module id");
          let mut module_id_expr = serde_json::to_string(module_id).expect("invalid module_id");
          if runtime_requirements.contains(RuntimeGlobals::ENTRY_MODULE_ID) {
            module_id_expr = format!(
              "{} = {module_id_expr}",
              runtime_template.render_runtime_globals(&RuntimeGlobals::ENTRY_MODULE_ID)
            );
          }

          if !chunk_ids.is_empty() {
            let on_chunks_loaded_callback = if supports_arrow_function {
              format!(
                "() => {}({module_id_expr})",
                runtime_template.render_runtime_globals(&RuntimeGlobals::REQUIRE)
              )
            } else {
              format!(
                "function() {{ return {}({module_id_expr}) }}",
                runtime_template.render_runtime_globals(&RuntimeGlobals::REQUIRE)
              )
            };
            buf2.push(
              format!(
                "{}{}(undefined, {}, {});",
                if i + 1 == entries.len() {
                  format!(
                    "var {} = ",
                    runtime_template.render_runtime_globals(&RuntimeGlobals::EXPORTS)
                  )
                } else {
                  "".to_string()
                },
                runtime_template.render_runtime_globals(&RuntimeGlobals::ON_CHUNKS_LOADED),
                stringify_array(&chunk_ids),
                on_chunks_loaded_callback
              )
              .into(),
            );
          } else if use_require {
            buf2.push(
              format!(
                "{}{}({module_id_expr});",
                if i + 1 == entries.len() {
                  format!(
                    "var {} = ",
                    runtime_template.render_runtime_globals(&RuntimeGlobals::EXPORTS)
                  )
                } else {
                  "".to_string()
                },
                runtime_template.render_runtime_globals(&RuntimeGlobals::REQUIRE)
              )
              .into(),
            )
          } else {
            let should_exec = i + 1 == entries.len();
            if should_exec {
              buf2.push(
                format!(
                  "var {} = {{}}",
                  runtime_template.render_runtime_globals(&RuntimeGlobals::EXPORTS)
                )
                .into(),
              );
            }
            if require_scope_used {
              buf2.push(
                format!(
                  "__webpack_modules__[{module_id_expr}](0, {}, {});",
                  if should_exec {
                    runtime_template.render_runtime_globals(&RuntimeGlobals::EXPORTS)
                  } else {
                    "{}".to_string()
                  },
                  runtime_template.render_runtime_globals(&RuntimeGlobals::REQUIRE)
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
                    runtime_template.render_runtime_globals(&RuntimeGlobals::EXPORTS)
                  } else {
                    "{}".to_string()
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
              runtime_template.render_runtime_globals(&RuntimeGlobals::ON_CHUNKS_LOADED)
            )
            .into(),
          );
        }
        if runtime_requirements.contains(RuntimeGlobals::STARTUP) {
          allow_inline_startup = false;
          header.push(
            format!(
              "// the startup function\n{} = {};\n",
              runtime_template.render_runtime_globals(&RuntimeGlobals::STARTUP),
              compilation.runtime_template.basic_function(
                "",
                &format!(
                  "{}\nreturn {}",
                  buf2.join("\n"),
                  runtime_template.render_runtime_globals(&RuntimeGlobals::EXPORTS)
                )
              )
            )
            .into(),
          );
          startup.push("// run startup".into());
          startup.push(
            format!(
              "var {} = {}();",
              runtime_template.render_runtime_globals(&RuntimeGlobals::EXPORTS),
              runtime_template.render_runtime_globals(&RuntimeGlobals::STARTUP)
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
            runtime_template.render_runtime_globals(&RuntimeGlobals::STARTUP)
          )
          .into(),
        );
      }
    } else if runtime_requirements.contains(RuntimeGlobals::STARTUP) {
      header.push(
        format!(
          "// the startup function\n// It's empty as some runtime module handles the default behavior\n{} = function(){{}};",
          runtime_template.render_runtime_globals(&RuntimeGlobals::STARTUP)
        )
        .into(),
      );
      startup.push("// run startup".into());
      startup.push(
        format!(
          "var __webpack_exports__ = {}();",
          runtime_template.render_runtime_globals(&RuntimeGlobals::STARTUP)
        )
        .into(),
      );
    }

    Ok(RenderBootstrapResult {
      header,
      startup,
      allow_inline_startup,
    })
  }

  pub async fn render_main(
    &self,
    compilation: &Compilation,
    chunk_ukey: &ChunkUkey,
    output_path: &str,
  ) -> Result<BoxSource> {
    let runtime_template = &compilation.runtime_template;
    let js_plugin_hooks = Self::get_compilation_hooks(compilation.id());
    let hooks = js_plugin_hooks
      .try_read()
      .expect("should have js plugin drive");
    let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);
    let supports_arrow_function = compilation
      .options
      .output
      .environment
      .supports_arrow_function();
    let runtime_requirements = ChunkGraph::get_tree_runtime_requirements(compilation, chunk_ukey);
    let mut chunk_init_fragments = ChunkInitFragments::default();
    let iife = compilation.options.output.iife;
    let mut all_strict = compilation.options.output.module;
    let RenderBootstrapResult {
      header,
      startup,
      allow_inline_startup,
    } = Self::render_bootstrap(chunk_ukey, compilation).await?;
    let module_graph = &compilation.get_module_graph();
    let all_modules = compilation.chunk_graph.get_chunk_modules_by_source_type(
      chunk_ukey,
      SourceType::JavaScript,
      module_graph,
    );
    let has_entry_modules = chunk.has_entry_module(&compilation.chunk_graph);
    let inlined_modules = if allow_inline_startup && has_entry_modules {
      Some(
        compilation
          .chunk_graph
          .get_chunk_entry_modules_with_chunk_group_iterable(chunk_ukey),
      )
    } else {
      None
    };
    let mut sources = ConcatSource::default();
    if iife {
      sources.add(RawStringSource::from(if supports_arrow_function {
        "(() => {\n"
      } else {
        "(function() {\n"
      }));
    }
    if !all_strict && all_modules.iter().all(|m| m.build_info().strict) {
      if let Some(strict_bailout) = hooks
        .strict_runtime_bailout
        .call(compilation, chunk_ukey)
        .await?
      {
        sources.add(RawStringSource::from(format!(
          "// runtime can't be in strict mode because {strict_bailout}.\n"
        )));
      } else {
        all_strict = true;
        sources.add(RawStringSource::from_static("\"use strict\";\n"));
      }
    }

    let chunk_modules: Vec<&dyn Module> = if let Some(inlined_modules) = inlined_modules {
      all_modules
        .clone()
        .into_iter()
        .filter(|m| !inlined_modules.contains_key(&m.identifier()))
        .collect::<Vec<_>>()
    } else {
      all_modules.clone()
    };

    let chunk_modules_result = render_chunk_modules(
      compilation,
      chunk_ukey,
      &chunk_modules,
      all_strict,
      output_path,
      &hooks,
    )
    .await?;
    let has_chunk_modules_result = chunk_modules_result.is_some();
    if has_chunk_modules_result
      || runtime_requirements.contains(RuntimeGlobals::MODULE_FACTORIES)
      || runtime_requirements.contains(RuntimeGlobals::MODULE_FACTORIES_ADD_ONLY)
      || runtime_requirements.contains(RuntimeGlobals::REQUIRE)
    {
      let chunk_modules_source =
        if let Some((chunk_modules_source, fragments)) = chunk_modules_result {
          chunk_init_fragments.extend(fragments);
          chunk_modules_source
        } else {
          RawStringSource::from_static("{}").boxed()
        };
      sources.add(RawStringSource::from_static("var __webpack_modules__ = ("));
      sources.add(chunk_modules_source);
      sources.add(RawStringSource::from_static(");\n"));
    }
    if !header.is_empty() {
      let mut header = header.join("\n");
      header.push('\n');
      sources.add(RawStringSource::from(header));
    }

    if compilation
      .chunk_graph
      .has_chunk_runtime_modules(chunk_ukey)
    {
      sources.add(render_runtime_modules(compilation, chunk_ukey).await?);
    }
    if let Some(inlined_modules) = inlined_modules {
      let last_entry_module = inlined_modules
        .keys()
        .next_back()
        .expect("should have last entry module");
      let mut startup_sources = ConcatSource::default();

      if runtime_requirements.contains(RuntimeGlobals::EXPORTS) {
        startup_sources.add(RawStringSource::from(format!(
          "var {} = {{}};\n",
          runtime_template.render_runtime_globals(&RuntimeGlobals::EXPORTS)
        )));
      }

      let renamed_inline_modules = if compilation.options.optimization.avoid_entry_iife {
        self
          .get_renamed_inline_module(
            &all_modules,
            inlined_modules,
            compilation,
            chunk_ukey,
            all_strict,
            has_chunk_modules_result,
            output_path,
            &hooks,
          )
          .await?
      } else {
        None
      };

      for (m_identifier, _) in inlined_modules {
        let m = module_graph
          .module_by_identifier(m_identifier)
          .expect("should have module");
        let Some((mut rendered_module, fragments, additional_fragments)) = render_module(
          compilation,
          chunk_ukey,
          m.as_ref(),
          all_strict,
          false,
          output_path,
          &hooks,
        )
        .await?
        else {
          continue;
        };

        if let Some(renamed_inline_modules) = &renamed_inline_modules
          && renamed_inline_modules.contains_key(m_identifier)
          && let Some(source) = renamed_inline_modules.get(m_identifier)
        {
          rendered_module = source.clone();
        };

        chunk_init_fragments.extend(fragments);
        chunk_init_fragments.extend(additional_fragments);
        let inner_strict = !all_strict && m.build_info().strict;
        let module_runtime_requirements =
          ChunkGraph::get_module_runtime_requirements(compilation, *m_identifier, chunk.runtime());
        let exports = module_runtime_requirements
          .map(|r| r.contains(RuntimeGlobals::EXPORTS))
          .unwrap_or_default();
        let exports_argument = m.get_exports_argument();
        let rspack_exports_argument = matches!(exports_argument, ExportsArgument::RspackExports);
        let rspack_exports = exports && rspack_exports_argument;
        let iife: Option<Cow<str>> = if inner_strict {
          Some("it needs to be in strict mode.".into())
        } else if inlined_modules.len() > 1 {
          Some("it needs to be isolated against other entry modules.".into())
        } else if has_chunk_modules_result && renamed_inline_modules.is_none() {
          Some("it needs to be isolated against other modules in the chunk.".into())
        } else if exports && !rspack_exports {
          Some(format!("it uses a non-standard name for the exports ({exports_argument}).").into())
        } else {
          hooks
            .embed_in_runtime_bailout
            .call(compilation, m, chunk)
            .await?
            .map(|s| s.into())
        };
        let footer;
        if let Some(iife) = iife {
          startup_sources.add(RawStringSource::from(format!(
            "// This entry needs to be wrapped in an IIFE because {iife}\n"
          )));
          if supports_arrow_function {
            startup_sources.add(RawStringSource::from_static("(() => {\n"));
            footer = "\n})();\n\n";
          } else {
            startup_sources.add(RawStringSource::from_static("!function() {\n"));
            footer = "\n}();\n";
          }
          if inner_strict {
            startup_sources.add(RawStringSource::from_static("\"use strict\";\n"));
          }
        } else {
          footer = "\n";
        }
        if exports {
          if m_identifier != last_entry_module {
            startup_sources.add(RawStringSource::from(format!(
              "var {exports_argument} = {{}};\n"
            )));
          } else if !rspack_exports_argument {
            startup_sources.add(RawStringSource::from(format!(
              "var {exports_argument} = {};\n",
              runtime_template.render_runtime_globals(&RuntimeGlobals::EXPORTS)
            )));
          }
        }
        startup_sources.add(rendered_module);
        startup_sources.add(RawStringSource::from(footer));
      }
      if runtime_requirements.contains(RuntimeGlobals::ON_CHUNKS_LOADED) {
        startup_sources.add(RawStringSource::from(format!(
          "{} = {}({});\n",
          runtime_template.render_runtime_globals(&RuntimeGlobals::EXPORTS),
          runtime_template.render_runtime_globals(&RuntimeGlobals::ON_CHUNKS_LOADED),
          runtime_template.render_runtime_globals(&RuntimeGlobals::EXPORTS),
        )));
      }
      let mut render_source = RenderSource {
        source: startup_sources.boxed(),
      };
      hooks
        .render_startup
        .call(
          compilation,
          chunk_ukey,
          last_entry_module,
          &mut render_source,
        )
        .await?;
      sources.add(render_source.source);
    } else if let Some(last_entry_module) = compilation
      .chunk_graph
      .get_chunk_entry_modules_with_chunk_group_iterable(chunk_ukey)
      .keys()
      .next_back()
    {
      let mut render_source = RenderSource {
        source: RawStringSource::from(startup.join("\n") + "\n").boxed(),
      };
      hooks
        .render_startup
        .call(
          compilation,
          chunk_ukey,
          last_entry_module,
          &mut render_source,
        )
        .await?;
      sources.add(render_source.source);
    }
    if has_entry_modules
      && runtime_requirements.contains(RuntimeGlobals::RETURN_EXPORTS_FROM_RUNTIME)
    {
      sources.add(RawStringSource::from_static(
        "return __webpack_exports__;\n",
      ));
    }
    if iife {
      sources.add(RawStringSource::from_static("})()\n"));
    }
    let final_source = render_init_fragments(
      sources.boxed(),
      chunk_init_fragments,
      &mut ChunkRenderContext {},
    )?;
    let mut render_source = RenderSource {
      source: final_source,
    };
    hooks
      .render
      .call(compilation, chunk_ukey, &mut render_source)
      .await?;
    Ok(if iife {
      ConcatSource::new([
        render_source.source,
        RawStringSource::from_static(";").boxed(),
      ])
      .boxed()
    } else {
      render_source.source
    })
  }

  #[allow(clippy::too_many_arguments)]
  pub async fn get_renamed_inline_module(
    &self,
    all_modules: &[&dyn Module],
    inlined_modules: &IdentifierLinkedMap<ChunkGroupUkey>,
    compilation: &Compilation,
    chunk_ukey: &ChunkUkey,
    all_strict: bool,
    has_chunk_modules_result: bool,
    output_path: &str,
    hooks: &JavascriptModulesPluginHooks,
  ) -> Result<Option<IdentifierMap<Arc<dyn Source>>>> {
    let inner_strict = !all_strict && all_modules.iter().all(|m| m.build_info().strict);
    let is_multiple_entries = inlined_modules.len() > 1;
    let single_entry_with_modules = inlined_modules.len() == 1 && has_chunk_modules_result;

    // TODO:
    // This step is before the IIFE reason calculation. Ideally, it should only be executed when this function can optimize the
    // IIFE reason. Otherwise, it should directly return false. There are four reasons now, we have skipped two already, the left
    // one is 'it uses a non-standard name for the exports'.
    if is_multiple_entries || inner_strict || !single_entry_with_modules {
      return Ok(None);
    }

    let mut inlined_modules_to_info: IdentifierMap<InlinedModuleInfo> = IdentifierMap::default();
    let mut non_inlined_module_through_idents: Vec<ConcatenatedModuleIdent> = Vec::new();
    let mut all_used_names = HashSet::from_iter(RESERVED_NAMES.iter().map(|item| Atom::new(*item)));
    let mut renamed_inline_modules: IdentifierMap<Arc<dyn Source>> = IdentifierMap::default();

    let render_module_results = rspack_futures::scope::<_, _>(|token| {
      all_modules.iter().for_each(|module| {
        let s = unsafe {
          token.used((
            compilation,
            chunk_ukey,
            module,
            all_strict,
            output_path,
            &hooks,
          ))
        };
        s.spawn(
          move |(compilation, chunk_ukey, module, all_strict, output_path, hooks)| async move {
            render_module(
              compilation,
              chunk_ukey,
              *module,
              all_strict,
              false,
              output_path,
              hooks,
            )
            .await
          },
        )
      });
    })
    .await
    .into_iter()
    .map(|r| r.to_rspack_result())
    .collect::<Result<Vec<_>>>()?;

    let mut render_module_sources = vec![];
    for (render_module_result, module) in render_module_results.into_iter().zip(all_modules.iter())
    {
      if let Some((rendered_module, ..)) = render_module_result? {
        render_module_sources.push((rendered_module, module));
      }
    }

    // make patch in parallel iteration
    let rename_info_patch = render_module_sources
      .par_iter()
      .fold(
        || {
          Ok(RenameInfoPatch {
            inlined_modules_to_info: IdentifierMap::default(),
            non_inlined_module_through_idents: Vec::new(),
            all_used_names: HashSet::from_iter(RESERVED_NAMES.iter().map(|item| Atom::new(*item))),
          })
        },
        |mut acc, (rendered_module, m)| {
          let is_inlined_module = inlined_modules.contains_key(&m.identifier());

          if let Ok(acc) = acc.as_mut() {
            let code = rendered_module;
            let mut use_cache = false;

            if is_inlined_module {
              if let Some(ident_info_with_hash) =
                self.rename_module_cache.get_inlined_info(&m.identifier())
                && let (Some(hash_current), Some(hash_cache)) = (
                  m.build_info().hash.as_ref(),
                  ident_info_with_hash.hash.as_ref(),
                )
                && *hash_current == *hash_cache
              {
                let WithHash { value, .. } = (*ident_info_with_hash).clone();
                acc.inlined_modules_to_info.insert(m.identifier(), value);
                use_cache = true;
              }
            } else if let Some(idents_with_hash) = self
              .rename_module_cache
              .get_non_inlined_idents(&m.identifier())
              && let (Some(hash_current), Some(hash_cache)) =
                (m.build_info().hash.as_ref(), idents_with_hash.hash.as_ref())
              && *hash_current == *hash_cache
            {
              acc
                .all_used_names
                .extend(idents_with_hash.value.iter().map(|v| v.id.sym.clone()));
              acc
                .non_inlined_module_through_idents
                .extend(idents_with_hash.value.clone());
              use_cache = true;
            }

            if !use_cache {
              let cm: Arc<swc_core::common::SourceMap> = Default::default();
              let fm = cm.new_source_file(
                Arc::new(FileName::Custom(m.identifier().to_string())),
                code.source().into_string_lossy().into_owned(),
              );
              let comments = swc_node_comments::SwcComments::default();
              let mut errors = vec![];

              if let Ok(program) = swc_core::ecma::parser::parse_file_as_program(
                &fm,
                swc_core::ecma::parser::Syntax::default(),
                swc_core::ecma::ast::EsVersion::EsNext,
                Some(&comments),
                &mut errors,
              ) {
                let mut ast: Ast = Ast::new(program, cm, Some(comments));
                let mut global_ctxt = SyntaxContext::empty();
                let mut module_ctxt = SyntaxContext::empty();

                ast.transform(|program, context| {
                  global_ctxt = global_ctxt.apply_mark(context.unresolved_mark);
                  module_ctxt = module_ctxt.apply_mark(context.top_level_mark);
                  program.visit_mut_with(&mut resolver(
                    context.unresolved_mark,
                    context.top_level_mark,
                    false,
                  ));
                });

                let mut collector = IdentCollector::default();
                ast.visit(|program, _ctxt| {
                  program.visit_with(&mut collector);
                });

                if is_inlined_module {
                  let mut module_scope_idents = Vec::new();

                  for ident in collector.ids {
                    if ident.id.ctxt == global_ctxt
                      || ident.id.ctxt != module_ctxt
                      || ident.is_class_expr_with_ident
                    {
                      acc.all_used_names.insert(ident.id.sym.clone());
                    }

                    if ident.id.ctxt == module_ctxt {
                      acc.all_used_names.insert(ident.id.sym.clone());
                      module_scope_idents.push(Arc::new(ident));
                    }
                  }

                  let ident = m.identifier();

                  let info = InlinedModuleInfo {
                    source: code.clone(),
                    module_scope_idents,
                    used_in_non_inlined: Vec::new(),
                  };
                  let runtime = compilation.chunk_by_ukey.expect_get(chunk_ukey).runtime();

                  self.rename_module_cache.inlined_modules_to_info.insert(
                    ident,
                    Arc::new(WithHash {
                      hash: ChunkGraph::get_module_hash(compilation, ident, runtime).cloned(),
                      value: info.clone(),
                    }),
                  );

                  acc.inlined_modules_to_info.insert(ident, info);
                } else {
                  let mut idents_vec = vec![];
                  let module_ident = m.identifier();
                  let runtime = compilation.chunk_by_ukey.expect_get(chunk_ukey).runtime();

                  for ident in collector.ids {
                    if ident.id.ctxt == global_ctxt {
                      acc.all_used_names.insert(ident.clone().id.sym.clone());
                      idents_vec.push(ident.clone());
                      acc.non_inlined_module_through_idents.push(ident);
                    }
                  }

                  self
                    .rename_module_cache
                    .non_inlined_modules_through_idents
                    .insert(
                      module_ident,
                      Arc::new(WithHash {
                        hash: ChunkGraph::get_module_hash(compilation, module_ident, runtime)
                          .cloned(),
                        value: idents_vec.clone(),
                      }),
                    );
                }
              }
            }
          }

          acc
        },
      )
      .reduce(
        || {
          Ok(RenameInfoPatch {
            inlined_modules_to_info: IdentifierMap::default(),
            non_inlined_module_through_idents: Vec::new(),
            all_used_names: HashSet::from_iter(RESERVED_NAMES.iter().map(|item| Atom::new(*item))),
          })
        },
        |acc, chunk| match acc {
          Ok(mut acc) => match chunk {
            Ok(chunk) => {
              acc
                .inlined_modules_to_info
                .extend(chunk.inlined_modules_to_info);
              acc
                .non_inlined_module_through_idents
                .extend(chunk.non_inlined_module_through_idents);
              acc.all_used_names.extend(chunk.all_used_names);
              Ok(acc)
            }
            Err(e) => Err(e),
          },
          Err(e) => Err(e),
        },
      );

    match rename_info_patch {
      Ok(rename_info_patch) => {
        // update patches
        inlined_modules_to_info.extend(rename_info_patch.inlined_modules_to_info);
        non_inlined_module_through_idents
          .extend(rename_info_patch.non_inlined_module_through_idents);
        all_used_names.extend(rename_info_patch.all_used_names);
      }
      Err(e) => return Err(e),
    }

    for (_ident, info) in inlined_modules_to_info.iter_mut() {
      for module_scope_ident in info.module_scope_idents.iter() {
        for non_inlined_module_through_ident in non_inlined_module_through_idents.iter() {
          if module_scope_ident.id.sym == non_inlined_module_through_ident.id.sym {
            info
              .used_in_non_inlined
              .push(Arc::clone(module_scope_ident));
          }
        }
      }
    }

    for (module_id, inlined_module_info) in inlined_modules_to_info.iter() {
      let InlinedModuleInfo {
        source: _source,
        module_scope_idents,
        used_in_non_inlined,
      } = inlined_module_info;

      let module = all_modules
        .iter()
        .find(|m| m.identifier() == *module_id)
        .unwrap_or_else(|| panic!("should find inlined module id \"{module_id}\" in all_modules"));

      let source: Arc<dyn Source> = _source.clone();
      let mut replace_source = ReplaceSource::new(_source.clone());

      if used_in_non_inlined.is_empty() {
        renamed_inline_modules.insert(*module_id, source);
        continue;
      }

      let mut binding_to_ref: HashMap<(Atom, SyntaxContext), Vec<ConcatenatedModuleIdent>> =
        HashMap::default();

      for module_scope_ident in module_scope_idents.iter() {
        match binding_to_ref.entry((
          module_scope_ident.id.sym.clone(),
          module_scope_ident.id.ctxt,
        )) {
          Entry::Occupied(mut occ) => {
            occ.get_mut().push(module_scope_ident.deref().clone());
          }
          Entry::Vacant(vac) => {
            vac.insert(vec![module_scope_ident.deref().clone()]);
          }
        };
      }

      for (id, refs) in binding_to_ref.iter() {
        let name = &id.0;
        let ident_used = !used_in_non_inlined
          .iter()
          .filter(|v| v.id.sym == *name)
          .collect::<Vec<_>>()
          .is_empty();

        if ident_used {
          let context = compilation.options.context.clone();
          let readable_identifier = module.readable_identifier(&context).to_string();
          let splitted_readable_identifier = split_readable_identifier(&readable_identifier);
          let new_name = find_new_name(name, &all_used_names, &splitted_readable_identifier);

          for identifier in refs.iter() {
            let span = identifier.id.span();
            let low = span.real_lo();
            let high = span.real_hi();

            if identifier.shorthand {
              replace_source.insert(high, &format!(": {new_name}"), None);
              continue;
            }

            replace_source.replace(low, high, &new_name, None);
          }

          all_used_names.insert(new_name);
        }
      }

      let source: Arc<dyn Source> = Arc::new(replace_source);
      renamed_inline_modules.insert(*module_id, Arc::clone(&source));
    }

    Ok(Some(renamed_inline_modules))
  }

  pub async fn render_chunk(
    &self,
    compilation: &Compilation,
    chunk_ukey: &ChunkUkey,
    output_path: &str,
  ) -> Result<BoxSource> {
    let js_plugin_hooks = Self::get_compilation_hooks(compilation.id());
    let hooks = js_plugin_hooks
      .try_read()
      .expect("should have js plugin drive");
    let module_graph = &compilation.get_module_graph();
    let is_module = compilation.options.output.module;
    let mut all_strict = compilation.options.output.module;
    let chunk_modules = compilation.chunk_graph.get_chunk_modules_by_source_type(
      chunk_ukey,
      SourceType::JavaScript,
      module_graph,
    );
    let mut sources = ConcatSource::default();
    if !all_strict && chunk_modules.iter().all(|m| m.build_info().strict) {
      if let Some(strict_bailout) = hooks
        .strict_runtime_bailout
        .call(compilation, chunk_ukey)
        .await?
      {
        sources.add(RawStringSource::from(format!(
          "// runtime can't be in strict mode because {strict_bailout}.\n"
        )));
      } else {
        sources.add(RawStringSource::from_static("\"use strict\";\n"));
        all_strict = true;
      }
    }
    let (chunk_modules_source, chunk_init_fragments) = render_chunk_modules(
      compilation,
      chunk_ukey,
      &chunk_modules,
      all_strict,
      output_path,
      &hooks,
    )
    .await?
    .unwrap_or_else(|| (RawStringSource::from_static("{}").boxed(), Vec::new()));
    let mut render_source = RenderSource {
      source: chunk_modules_source,
    };
    hooks
      .render_chunk
      .call(compilation, chunk_ukey, &mut render_source)
      .await?;
    let source_with_fragments = render_init_fragments(
      render_source.source,
      chunk_init_fragments,
      &mut ChunkRenderContext {},
    )?;
    let mut render_source = RenderSource {
      source: source_with_fragments,
    };
    hooks
      .render
      .call(compilation, chunk_ukey, &mut render_source)
      .await?;
    sources.add(render_source.source);
    if !is_module {
      sources.add(RawStringSource::from_static(";"));
    }
    Ok(sources.boxed())
  }

  #[inline]
  pub async fn get_chunk_hash(
    &self,
    chunk_ukey: &ChunkUkey,
    compilation: &Compilation,
    hasher: &mut RspackHash,
  ) -> Result<()> {
    let hooks = Self::get_compilation_hooks(compilation.id());
    hooks
      .try_read()
      .expect("should have js plugin drive")
      .chunk_hash
      .call(compilation, chunk_ukey, hasher)
      .await?;
    Ok(())
  }

  #[inline]
  pub async fn update_hash_with_bootstrap(
    &self,
    chunk_ukey: &ChunkUkey,
    compilation: &Compilation,
    hasher: &mut RspackHash,
  ) -> Result<()> {
    // sample hash use content
    let RenderBootstrapResult {
      header,
      startup,
      allow_inline_startup,
    } = Self::render_bootstrap(chunk_ukey, compilation).await?;
    header.hash(hasher);
    startup.hash(hasher);
    allow_inline_startup.hash(hasher);
    Ok(())
  }
}

#[derive(Debug, Clone)]
pub struct ExtractedCommentsInfo {
  pub source: BoxSource,
  pub comments_file_name: String,
}

#[derive(Debug)]
pub struct RenderBootstrapResult<'a> {
  pub header: Vec<Cow<'a, str>>,
  pub startup: Vec<Cow<'a, str>>,
  pub allow_inline_startup: bool,
}
