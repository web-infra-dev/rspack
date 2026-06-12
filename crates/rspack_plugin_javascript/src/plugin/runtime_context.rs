use std::borrow::Cow;

use rspack_core::{
  ChunkCodeTemplate, ChunkGraph, ChunkInitFragments, ChunkRenderContext, ChunkUkey,
  CodeGenerationDataTopLevelDeclarations, Compilation, ExportsArgument, Module, RuntimeGlobals,
  RuntimeVariable, SourceType, property_access, render_init_fragments,
  rspack_sources::{BoxSource, ConcatSource, RawStringSource, SourceExt},
};
use rspack_error::Result;

use super::{JsPlugin, RenderBootstrapResult};
use crate::{
  RenderSource,
  runtime::{
    render_chunk_modules, render_module, render_runtime_context_declaration,
    render_runtime_context_require_assignment, render_runtime_modules, stringify_array,
  },
};

impl JsPlugin {
  pub fn render_rspack_require<'me>(
    chunk_ukey: &ChunkUkey,
    compilation: &'me Compilation,
    runtime_template: &ChunkCodeTemplate,
  ) -> Vec<Cow<'me, str>> {
    let runtime_requirements = compilation
      .cgc_runtime_requirements_artifact
      .get(chunk_ukey)
      .copied()
      .unwrap_or_default();

    let strict_module_error_handling = compilation.options.output.strict_module_error_handling;
    let need_module_defer =
      runtime_requirements.contains(RuntimeGlobals::MAKE_DEFERRED_NAMESPACE_OBJECT);
    let callable_require = runtime_template.render_runtime_variable(&RuntimeVariable::Require);
    let require_argument = runtime_template.render_runtime_argument();
    let runtime_context = runtime_template.render_runtime_variable(&RuntimeVariable::Context);
    let module_factories = runtime_template.render_runtime_variable(&RuntimeVariable::Modules);
    let module_cache = runtime_template.render_runtime_variable(&RuntimeVariable::ModuleCache);
    let mut sources: Vec<Cow<str>> = Vec::new();

    sources.push(
      format!(
        r#"// Check if module is in cache
var cachedModule = {module_cache}[moduleId];
if (cachedModule !== undefined) {{"#,
      )
      .into(),
    );

    if strict_module_error_handling {
      sources.push("if (cachedModule.error !== undefined) throw cachedModule.error;".into());
    }

    sources.push(
      format!(
        r#"return cachedModule.exports;
}}
// Create a new module (and put it into the cache)
var module = ({module_cache}[moduleId] = {{"#,
      )
      .into(),
    );

    if runtime_requirements.contains(RuntimeGlobals::MODULE_ID) {
      sources.push("id: moduleId,".into());
    }

    if runtime_requirements.contains(RuntimeGlobals::MODULE_LOADED) {
      sources.push("loaded: false,".into());
    }

    if need_module_defer {
      sources.push("exports: __rspack_deferred_exports[moduleId] || {}".into());
    } else {
      sources.push("exports: {}".into());
    }
    sources.push("});\n// Execute the module function".into());

    let module_execution = if runtime_requirements
      .contains(RuntimeGlobals::INTERCEPT_MODULE_EXECUTION)
    {
      format!(
        r#"
        var execOptions = {{ id: moduleId, module: module, factory: {module_factories}[moduleId], require: {callable_require}, context: Object.create({runtime_context}) }};
        {}.forEach(function(handler) {{ handler(execOptions); }});
        module = execOptions.module;
        execOptions.factory.call(module.exports, module, module.exports, execOptions.context);
      "#,
        runtime_template.render_runtime_globals(&RuntimeGlobals::INTERCEPT_MODULE_EXECUTION)
      )
      .into()
    } else if runtime_requirements.contains(RuntimeGlobals::THIS_AS_EXPORTS) {
      format!(
        "{module_factories}[moduleId].call(module.exports, module, module.exports, {require_argument});\n"
      )
      .into()
    } else {
      format!("{module_factories}[moduleId](module, module.exports, {require_argument});\n").into()
    };

    if strict_module_error_handling {
      sources.push("try {\n".into());
      sources.push(module_execution);
      sources.push("} catch (e) {".into());
      if need_module_defer {
        sources.push("delete __rspack_deferred_exports[moduleId];".into());
      }
      sources.push("module.error = e;\nthrow e;".into());
      sources.push("}".into());
    } else {
      sources.push(module_execution);
      if need_module_defer {
        sources.push("delete __rspack_deferred_exports[moduleId];".into());
      }
    }

    if runtime_requirements.contains(RuntimeGlobals::MODULE_LOADED) {
      sources.push("// Flag the module as loaded\nmodule.loaded = true;".into());
    }

    sources.push("// Return the exports of the module\nreturn module.exports;".into());

    sources
  }

  pub async fn render_rspack_bootstrap<'me>(
    chunk_ukey: &ChunkUkey,
    compilation: &'me Compilation,
    runtime_template: &ChunkCodeTemplate,
  ) -> Result<RenderBootstrapResult<'me>> {
    let runtime_requirements = compilation
      .cgc_runtime_requirements_artifact
      .get(chunk_ukey)
      .copied()
      .unwrap_or_default();
    let chunk = compilation
      .build_chunk_graph_artifact
      .chunk_by_ukey
      .expect_get(chunk_ukey);
    let module_factories = runtime_requirements.contains(RuntimeGlobals::MODULE_FACTORIES);
    let require_function = runtime_requirements.contains(RuntimeGlobals::REQUIRE);
    let module_cache = runtime_requirements.contains(RuntimeGlobals::MODULE_CACHE);
    let intercept_module_execution =
      runtime_requirements.contains(RuntimeGlobals::INTERCEPT_MODULE_EXECUTION);
    let module_used = runtime_requirements.contains(RuntimeGlobals::MODULE);
    let has_custom_runtime_module = compilation
      .build_chunk_graph_artifact
      .chunk_graph
      .get_chunk_runtime_modules_iterable(chunk_ukey)
      .any(|runtime_module_identifier| {
        let runtime_module = &compilation.runtime_modules[runtime_module_identifier];
        runtime_module.get_custom_source().is_some()
          || runtime_module.get_constructor_name() == "RuntimeModuleFromJs"
      });
    let require_scope_used = runtime_requirements.contains(RuntimeGlobals::REQUIRE_SCOPE)
      || !runtime_requirements.renderable_require_scope().is_empty()
      || has_custom_runtime_module;
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
    let has_bootstrap_runtime_context = runtime_requirements.needs_bootstrap_runtime_context();

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
      header.push(
        format!(
          r#"// The module cache
var {} = {{}};
"#,
          runtime_template.render_runtime_variable(&RuntimeVariable::ModuleCache)
        )
        .into(),
      );
    }

    if need_module_defer {
      header.push(
        r#"// The deferred module cache
var __rspack_deferred_exports = {};
"#
        .into(),
      );
    }

    if has_bootstrap_runtime_context {
      header.push(render_runtime_context_declaration(runtime_template).into());
    }

    if use_require {
      header.push(
        format!(
          r#"// The require function
function {}(moduleId) {{
"#,
          runtime_template.render_runtime_variable(&RuntimeVariable::Require)
        )
        .into(),
      );
      header.extend(Self::render_rspack_require(
        chunk_ukey,
        compilation,
        runtime_template,
      ));
      header.push(
        r#"
}
"#
        .into(),
      );
      header.push(render_runtime_context_require_assignment(runtime_template).into());
    } else if require_scope_used && !has_bootstrap_runtime_context {
      header.push(render_runtime_context_declaration(runtime_template).into());
    }

    if module_factories || runtime_requirements.contains(RuntimeGlobals::MODULE_FACTORIES_ADD_ONLY)
    {
      let runtime_context = runtime_template.render_runtime_variable(&RuntimeVariable::Context);
      let name = RuntimeGlobals::MODULE_FACTORIES
        .rspack_context_property_name()
        .expect("module factories should have context property name");
      let module_factories = format!("{runtime_context}{}", property_access([name], 0));
      header.push(
        format!(
          r#"// expose the modules object ({modules})
{module_factories} = {modules};
"#,
          modules = runtime_template.render_runtime_variable(&RuntimeVariable::Modules),
          module_factories = module_factories
        )
        .into(),
      );
    }

    if runtime_requirements.contains(RuntimeGlobals::MODULE_CACHE) {
      let runtime_context = runtime_template.render_runtime_variable(&RuntimeVariable::Context);
      let name = RuntimeGlobals::MODULE_CACHE
        .rspack_context_property_name()
        .expect("module cache should have context property name");
      let module_cache_runtime_global = format!("{runtime_context}{}", property_access([name], 0));
      header.push(
        format!(
          r#"// expose the module cache
{} = {};
"#,
          module_cache_runtime_global,
          runtime_template.render_runtime_variable(&RuntimeVariable::ModuleCache),
        )
        .into(),
      );
    }

    if intercept_module_execution {
      let runtime_context = runtime_template.render_runtime_variable(&RuntimeVariable::Context);
      let name = RuntimeGlobals::INTERCEPT_MODULE_EXECUTION
        .rspack_context_property_name()
        .expect("intercept module execution should have context property name");
      let intercept_module_execution = format!("{runtime_context}{}", property_access([name], 0));
      header.push(
        format!(
          r#"// expose the module execution interceptor
{intercept_module_execution} = [];
"#,
        )
        .into(),
      );
    }

    if !runtime_requirements.contains(RuntimeGlobals::STARTUP_NO_DEFAULT) {
      if chunk.has_entry_module(&compilation.build_chunk_graph_artifact.chunk_graph) {
        let mut buf2: Vec<Cow<str>> = Vec::new();
        buf2.push("// Load entry module and return exports".into());
        let entries = compilation
          .build_chunk_graph_artifact
          .chunk_graph
          .get_chunk_entry_modules_with_chunk_group_iterable(chunk_ukey);
        let module_graph = compilation.get_module_graph();
        for (i, (module, entry)) in entries.iter().enumerate() {
          let chunk_group = compilation
            .build_chunk_graph_artifact
            .chunk_group_by_ukey
            .expect_get(entry);
          let chunk_ids = chunk_group
            .chunks
            .iter()
            .filter(|c| *c != chunk_ukey)
            .map(|chunk_ukey| {
              compilation
                .build_chunk_graph_artifact
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
            let module_graph_cache = &compilation.module_graph_cache_artifact;
            module_graph
              .get_incoming_connections_by_origin_module(module)
              .modules()
              .iter()
              .any(|(origin_module, connections)| {
                connections.iter().any(|c| {
                  c.is_target_active(
                    module_graph,
                    Some(chunk.runtime()),
                    module_graph_cache,
                    &compilation
                      .build_module_graph_artifact
                      .side_effects_state_artifact,
                    &compilation.exports_info_artifact,
                  )
                }) && compilation
                  .build_chunk_graph_artifact
                  .chunk_graph
                  .get_module_runtimes_iter(
                    *origin_module,
                    &compilation.build_chunk_graph_artifact.chunk_by_ukey,
                  )
                  .any(|runtime| runtime.intersection(chunk.runtime()).count() > 0)
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
          let mut module_id_expr = rspack_util::json_stringify(module_id);
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
                  String::new()
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
                  String::new()
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
                  "{}[{module_id_expr}](0, {}, {});",
                  runtime_template.render_runtime_variable(&RuntimeVariable::Modules),
                  if should_exec {
                    runtime_template.render_runtime_globals(&RuntimeGlobals::EXPORTS)
                  } else {
                    "{}".to_string()
                  },
                  runtime_template.render_runtime_argument()
                )
                .into(),
              );
            } else if let Some(entry_runtime_requirements) = entry_runtime_requirements
              && entry_runtime_requirements.contains(RuntimeGlobals::EXPORTS)
            {
              buf2.push(
                format!(
                  "{}[{module_id_expr}](0, {});",
                  runtime_template.render_runtime_variable(&RuntimeVariable::Modules),
                  if should_exec {
                    runtime_template.render_runtime_globals(&RuntimeGlobals::EXPORTS)
                  } else {
                    "{}".to_string()
                  }
                )
                .into(),
              );
            } else {
              buf2.push(
                format!(
                  "{}[{module_id_expr}]();",
                  runtime_template.render_runtime_variable(&RuntimeVariable::Modules)
                )
                .into(),
              );
            }
          }
        }
        if runtime_requirements.contains(RuntimeGlobals::ON_CHUNKS_LOADED) {
          buf2.push(
            format!(
              "{exports} = {on_chunks_loaded}({exports});",
              exports = runtime_template.render_runtime_variable(&RuntimeVariable::Exports),
              on_chunks_loaded =
                runtime_template.render_runtime_globals(&RuntimeGlobals::ON_CHUNKS_LOADED)
            )
            .into(),
          );
        }
        if runtime_requirements.contains(RuntimeGlobals::STARTUP) {
          let exports = runtime_template.render_runtime_globals(&RuntimeGlobals::EXPORTS);
          allow_inline_startup = false;
          header.push(
            format!(
              r#"// the startup function
{} = {};
"#,
              runtime_template.render_runtime_globals(&RuntimeGlobals::STARTUP),
              runtime_template
                .basic_function("", &format!("{}\nreturn {}", buf2.join("\n"), exports))
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
            r#"// the startup function
// It's empty as no entry modules are in this chunk
{} = function(){{}};"#,
            runtime_template.render_runtime_globals(&RuntimeGlobals::STARTUP)
          )
          .into(),
        );
      }
    } else if runtime_requirements.contains(RuntimeGlobals::STARTUP) {
      header.push(
        format!(
          r#"// the startup function
// It's empty as some runtime module handles the default behavior
{} = function(){{}};"#,
          runtime_template.render_runtime_globals(&RuntimeGlobals::STARTUP)
        )
        .into(),
      );
      startup.push("// run startup".into());
      startup.push(
        format!(
          "var {} = {}();",
          runtime_template.render_runtime_variable(&RuntimeVariable::Exports),
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
}

impl JsPlugin {
  pub async fn render_rspack_main(
    &self,
    compilation: &Compilation,
    chunk_ukey: &ChunkUkey,
    output_path: &str,
    runtime_template: &ChunkCodeTemplate,
  ) -> Result<BoxSource> {
    let js_plugin_hooks = Self::get_compilation_hooks(compilation.id());
    let hooks = js_plugin_hooks
      .try_read()
      .expect("should have js plugin drive");
    let chunk = compilation
      .build_chunk_graph_artifact
      .chunk_by_ukey
      .expect_get(chunk_ukey);
    let supports_arrow_function = compilation
      .options
      .output
      .environment
      .supports_arrow_function();
    let runtime_requirements = compilation
      .cgc_runtime_requirements_artifact
      .get(chunk_ukey)
      .copied()
      .unwrap_or_default();
    let has_bootstrap_runtime_context = runtime_requirements.needs_bootstrap_runtime_context();
    let mut chunk_init_fragments = ChunkInitFragments::default();
    let iife = compilation.options.output.iife;
    let mut all_strict = compilation.options.output.module;
    let RenderBootstrapResult {
      header,
      startup,
      allow_inline_startup,
    } = Self::render_rspack_bootstrap(chunk_ukey, compilation, runtime_template).await?;
    let module_graph = &compilation.get_module_graph();
    let all_modules = compilation
      .build_chunk_graph_artifact
      .chunk_graph
      .get_chunk_modules_by_source_type(chunk_ukey, SourceType::JavaScript, module_graph);
    let has_entry_modules =
      chunk.has_entry_module(&compilation.build_chunk_graph_artifact.chunk_graph);
    let inlined_modules = if allow_inline_startup && has_entry_modules {
      Some(
        compilation
          .build_chunk_graph_artifact
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
      runtime_template,
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
      sources.add(RawStringSource::from(format!(
        "var {} = (",
        runtime_template.render_runtime_variable(&RuntimeVariable::Modules)
      )));
      sources.add(chunk_modules_source);
      sources.add(RawStringSource::from_static(");\n"));
    }
    if !header.is_empty() {
      let mut header = header.join("\n");
      header.push('\n');
      sources.add(RawStringSource::from(header));
    }
    if compilation
      .build_chunk_graph_artifact
      .chunk_graph
      .has_chunk_runtime_modules(chunk_ukey)
    {
      sources.add(render_runtime_modules(compilation, chunk_ukey, runtime_template).await?);
    } else if runtime_template.uses_runtime_context() && !has_bootstrap_runtime_context {
      sources.add(RawStringSource::from(render_runtime_context_declaration(
        runtime_template,
      )));
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
            runtime_template,
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
          runtime_template,
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
          Some(
            format!(
              "it uses a non-standard name for the exports ({}).",
              runtime_template.render_exports_argument(exports_argument)
            )
            .into(),
          )
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
          let exports_argument = runtime_template.render_exports_argument(exports_argument);
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
          runtime_template,
        )
        .await?;
      sources.add(render_source.source);
    } else if let Some(last_entry_module) = compilation
      .build_chunk_graph_artifact
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
          runtime_template,
        )
        .await?;
      sources.add(render_source.source);
    }
    if has_entry_modules
      && runtime_requirements.contains(RuntimeGlobals::RETURN_EXPORTS_FROM_RUNTIME)
    {
      sources.add(RawStringSource::from(format!(
        "return {};\n",
        runtime_template.render_runtime_variable(&RuntimeVariable::Exports)
      )));
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
      .call(
        compilation,
        chunk_ukey,
        &mut render_source,
        runtime_template,
      )
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
}
