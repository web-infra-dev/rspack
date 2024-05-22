use std::collections::{HashMap, HashSet};
use std::ops::Deref;
use std::sync::Arc;
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
use std::collections::hash_map::Entry;
use std::hash::Hash;

pub use drive::*;
pub use flag_dependency_exports_plugin::*;
pub use flag_dependency_usage_plugin::*;
use indoc::indoc;
pub use mangle_exports_plugin::*;
pub use module_concatenation_plugin::*;
use once_cell::sync::Lazy;
use rspack_ast::javascript::Ast;
use rspack_core::concatenated_module::find_new_name;
use rspack_core::reserved_names::RESERVED_NAMES;
use rspack_core::rspack_sources::{
  BoxSource, ConcatSource, RawSource, ReplaceSource, Source, SourceExt,
};
use rspack_core::{
  basic_function, render_init_fragments, ChunkGroupUkey, ChunkInitFragments, ChunkRenderContext,
  ChunkUkey, CodeGenerationDataTopLevelDeclarations, Compilation, CompilationId,
  ConcatenatedModuleIdent, ExportsArgument, Module, RuntimeGlobals, SourceType, SpanExt,
};
use rspack_core::{BoxModule, IdentCollector};
use rspack_error::Result;
use rspack_hash::RspackHash;
use rspack_hook::plugin;
use rspack_identifier::IdentifierLinkedMap;
use rspack_util::diff_mode;
use rspack_util::fx_hash::FxDashMap;
pub use side_effects_flag_plugin::*;
use swc_core::atoms::Atom;
use swc_core::common::{FileName, Spanned, SyntaxContext};
use swc_core::ecma::transforms::base::resolver;

use crate::runtime::{
  render_chunk_modules, render_module, render_runtime_modules, stringify_array,
};

static COMPILATION_HOOKS_MAP: Lazy<FxDashMap<CompilationId, Box<JavascriptModulesPluginHooks>>> =
  Lazy::new(Default::default);

#[plugin]
#[derive(Debug, Default)]
pub struct JsPlugin;

struct InlinedModuleInfo {
  source: Arc<dyn Source>,
  module_scope_idents: Vec<Arc<ConcatenatedModuleIdent>>,
  used_in_non_inlined: Vec<Arc<ConcatenatedModuleIdent>>,
}

impl JsPlugin {
  pub fn get_compilation_hooks(
    compilation: &Compilation,
  ) -> dashmap::mapref::one::Ref<'_, CompilationId, Box<JavascriptModulesPluginHooks>> {
    let id = compilation.id();
    if !COMPILATION_HOOKS_MAP.contains_key(&id) {
      COMPILATION_HOOKS_MAP.insert(id, Default::default());
    }
    COMPILATION_HOOKS_MAP
      .get(&id)
      .expect("should have js plugin drive")
  }

  pub fn get_compilation_hooks_mut(
    compilation: &Compilation,
  ) -> dashmap::mapref::one::RefMut<'_, CompilationId, Box<JavascriptModulesPluginHooks>> {
    COMPILATION_HOOKS_MAP.entry(compilation.id()).or_default()
  }

  pub fn render_require(&self, chunk_ukey: &ChunkUkey, compilation: &Compilation) -> Vec<Cow<str>> {
    let runtime_requirements = compilation
      .chunk_graph
      .get_chunk_runtime_requirements(chunk_ukey);

    let strict_module_error_handling = compilation.options.output.strict_module_error_handling;
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

    sources.push("exports: {}".into());
    sources.push("});\n// Execute the module function".into());

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

    sources
  }

  pub fn render_bootstrap(
    &self,
    chunk_ukey: &ChunkUkey,
    compilation: &Compilation,
  ) -> Result<RenderBootstrapResult> {
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
      header.extend(self.render_require(chunk_ukey, compilation));
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
          if allow_inline_startup && {
            let codegen = compilation
              .code_generation_results
              .get(module, Some(&chunk.runtime));
            let module_graph = compilation.get_module_graph();
            let top_level_decls = codegen
              .data
              .get::<CodeGenerationDataTopLevelDeclarations>()
              .map(|d| d.inner())
              .or_else(|| {
                module_graph
                  .module_by_identifier(module)
                  .and_then(|m| m.build_info())
                  .and_then(|build_info| build_info.top_level_declarations.as_ref())
              });
            top_level_decls.is_none()
          } {
            buf2.push("// This entry module doesn't tell about it's top-level declarations so it can't be inlined".into());
            allow_inline_startup = false;
          }
          let hooks = JsPlugin::get_compilation_hooks(compilation);
          let bailout = hooks.inline_in_runtime_bailout.call(compilation)?;
          if allow_inline_startup && let Some(bailout) = bailout {
            buf2.push(format!("// This entry module can't be inlined because {bailout}").into());
            allow_inline_startup = false;
          }
          if allow_inline_startup && diff_mode::is_diff_mode() {
            buf2.push("// This entry module can't be inlined in diff mode".into());
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
              "// the startup function\n{} = {};\n",
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
  ) -> Result<BoxSource> {
    let hooks = Self::get_compilation_hooks(compilation);
    let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);
    let supports_arrow_function = compilation
      .options
      .output
      .environment
      .supports_arrow_function();
    let runtime_requirements = compilation
      .chunk_graph
      .get_tree_runtime_requirements(chunk_ukey);
    let mut chunk_init_fragments = ChunkInitFragments::default();
    let iife = compilation.options.output.iife;
    let mut all_strict = compilation.options.output.module;
    let RenderBootstrapResult {
      header,
      startup,
      allow_inline_startup,
    } = self.render_bootstrap(chunk_ukey, compilation)?;
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
      sources.add(RawSource::from(if supports_arrow_function {
        "(() => { // webpackBootstrap\n"
      } else {
        "(function() { // webpackBootstrap\n"
      }));
    }
    if !all_strict
      && all_modules.iter().all(|m| {
        let build_info = m
          .build_info()
          .expect("should have build_info in render_main");
        build_info.strict
      })
    {
      if let Some(strict_bailout) = hooks.strict_runtime_bailout.call(compilation, chunk_ukey)? {
        sources.add(RawSource::from(format!(
          "// runtime can't be in strict mode because {strict_bailout}.\n"
        )));
      } else {
        all_strict = true;
        sources.add(RawSource::from("\"use strict\";\n"));
      }
    }

    let chunk_modules: Vec<&Box<dyn Module>> = if let Some(inlined_modules) = inlined_modules {
      all_modules
        .clone()
        .into_iter()
        .filter(|m| !inlined_modules.contains_key(&m.identifier()))
        .collect::<Vec<_>>()
    } else {
      all_modules.clone()
    };

    let chunk_modules_result =
      render_chunk_modules(compilation, chunk_ukey, &chunk_modules, all_strict)?;
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
          RawSource::from("{}").boxed()
        };
      sources.add(RawSource::from("var __webpack_modules__ = ("));
      sources.add(chunk_modules_source);
      sources.add(RawSource::from(");\n"));
      sources.add(RawSource::from(
        "/************************************************************************/\n",
      ));
    }
    if !header.is_empty() {
      let mut header = header.join("\n");
      header.push('\n');
      sources.add(RawSource::from(header));
      sources.add(RawSource::from(
        "/************************************************************************/\n",
      ));
    }

    if compilation
      .chunk_graph
      .has_chunk_runtime_modules(chunk_ukey)
    {
      sources.add(render_runtime_modules(compilation, chunk_ukey)?);
      sources.add(RawSource::from(
        "/************************************************************************/\n",
      ));
    }
    if let Some(inlined_modules) = inlined_modules {
      let last_entry_module = inlined_modules
        .keys()
        .last()
        .expect("should have last entry module");
      let mut startup_sources = ConcatSource::default();
      startup_sources.add(RawSource::from(format!(
        "var {} = {{}};\n",
        RuntimeGlobals::EXPORTS
      )));

      let renamed_inline_modules = self.rename_inline_modules(
        &all_modules,
        inlined_modules,
        compilation,
        chunk_ukey,
        all_strict,
      )?;

      for (m_identifier, _) in inlined_modules {
        let m = module_graph
          .module_by_identifier(m_identifier)
          .expect("should have module");
        let Some((mut rendered_module, fragments, additional_fragments)) =
          render_module(compilation, chunk_ukey, m, all_strict, false)?
        else {
          continue;
        };

        if renamed_inline_modules
          .get(&m_identifier.to_string())
          .is_some()
        {
          if let Some(source) = renamed_inline_modules.get(&m_identifier.to_string()) {
            rendered_module = source.clone();
          };
        }

        chunk_init_fragments.extend(fragments);
        chunk_init_fragments.extend(additional_fragments);
        let inner_strict = !all_strict && m.build_info().expect("should have build_info").strict;
        let module_runtime_requirements = compilation
          .chunk_graph
          .get_module_runtime_requirements(*m_identifier, &chunk.runtime);
        let exports = module_runtime_requirements
          .map(|r| r.contains(RuntimeGlobals::EXPORTS))
          .unwrap_or_default();
        let exports_argument = m.get_exports_argument();
        let webpack_exports_argument = matches!(exports_argument, ExportsArgument::WebpackExports);
        let webpack_exports = exports && webpack_exports_argument;
        let iife: Option<Cow<str>> = if inner_strict {
          Some("it need to be in strict mode.".into())
        } else if inlined_modules.len() > 1 {
          Some("it need to be isolated against other entry modules.".into())
        } else if exports && !webpack_exports {
          Some(format!("it uses a non-standard name for the exports ({exports_argument}).").into())
        } else {
          hooks
            .embed_in_runtime_bailout
            .call(compilation, m, chunk)?
            .map(|s| s.into())
        };
        let footer;
        if let Some(iife) = iife {
          startup_sources.add(RawSource::from(format!(
            "// This entry need to be wrapped in an IIFE because {iife}\n"
          )));
          if supports_arrow_function {
            startup_sources.add(RawSource::from("(() => {\n"));
            footer = "\n})();\n\n";
          } else {
            startup_sources.add(RawSource::from("!function() {\n"));
            footer = "\n}();\n";
          }
          if inner_strict {
            startup_sources.add(RawSource::from("\"use strict\";\n"));
          }
        } else {
          footer = "\n";
        }
        if exports {
          if m_identifier != last_entry_module {
            startup_sources.add(RawSource::from(format!("var {exports_argument} = {{}};\n")));
          } else if !webpack_exports_argument {
            startup_sources.add(RawSource::from(format!(
              "var {exports_argument} = {};\n",
              RuntimeGlobals::EXPORTS
            )));
          }
        }
        startup_sources.add(rendered_module);
        startup_sources.add(RawSource::from(footer));
      }
      if runtime_requirements.contains(RuntimeGlobals::ON_CHUNKS_LOADED) {
        startup_sources.add(RawSource::from(format!(
          "{} = {}({});\n",
          RuntimeGlobals::EXPORTS,
          RuntimeGlobals::ON_CHUNKS_LOADED,
          RuntimeGlobals::EXPORTS,
        )));
      }
      let mut render_source = RenderSource {
        source: startup_sources.boxed(),
      };
      hooks.render_startup.call(
        compilation,
        chunk_ukey,
        last_entry_module,
        &mut render_source,
      )?;
      sources.add(render_source.source);
    } else if let Some(last_entry_module) = compilation
      .chunk_graph
      .get_chunk_entry_modules_with_chunk_group_iterable(chunk_ukey)
      .keys()
      .last()
    {
      let mut render_source = RenderSource {
        source: RawSource::from(startup.join("\n") + "\n").boxed(),
      };
      hooks.render_startup.call(
        compilation,
        chunk_ukey,
        last_entry_module,
        &mut render_source,
      )?;
      sources.add(render_source.source);
    }
    if has_entry_modules
      && runtime_requirements.contains(RuntimeGlobals::RETURN_EXPORTS_FROM_RUNTIME)
    {
      sources.add(RawSource::from("return __webpack_exports__;\n"));
    }
    if iife {
      sources.add(RawSource::from("})()\n"));
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
      .call(compilation, chunk_ukey, &mut render_source)?;
    Ok(if iife {
      ConcatSource::new([render_source.source, RawSource::from(";").boxed()]).boxed()
    } else {
      render_source.source
    })
  }

  pub fn rename_inline_modules(
    &self,
    all_modules: &[&BoxModule],
    inlined_modules: &IdentifierLinkedMap<ChunkGroupUkey>,
    compilation: &Compilation,
    chunk_ukey: &ChunkUkey,
    all_strict: bool,
  ) -> Result<HashMap<String, Arc<dyn Source>>> {
    let mut inlined_modules_to_info: HashMap<String, InlinedModuleInfo> = HashMap::new();
    let mut non_inlined_module_through_idents: Vec<ConcatenatedModuleIdent> = Vec::new();
    let mut renamed_inline_modules: HashMap<String, Arc<dyn Source>> = HashMap::new();
    let mut all_used_names = HashSet::from_iter(RESERVED_NAMES.iter().map(|item| item.to_string()));

    for m in all_modules.iter() {
      let is_inlined_module = inlined_modules.contains_key(&m.identifier());

      let Some((rendered_module, ..)) =
        render_module(compilation, chunk_ukey, m, all_strict, false)?
      else {
        continue;
      };

      let code = rendered_module;
      let cm: Arc<swc_core::common::SourceMap> = Default::default();
      let fm = cm.new_source_file(
        FileName::Custom(m.identifier().to_string()),
        code.source().to_string(),
      );
      let comments = swc_node_comments::SwcComments::default();
      let mut errors = vec![];

      let Ok(program) = swc_core::ecma::parser::parse_file_as_program(
        &fm,
        swc_core::ecma::parser::Syntax::default(),
        swc_core::ecma::ast::EsVersion::EsNext,
        Some(&comments),
        &mut errors,
      ) else {
        continue;
      };

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
          if ident.id.span.ctxt == global_ctxt
            || ident.id.span.ctxt != module_ctxt
            || ident.is_class_expr_with_ident
          {
            all_used_names.insert(ident.id.sym.to_string());
          }

          if ident.id.span.ctxt == module_ctxt {
            all_used_names.insert(ident.id.sym.to_string());
            module_scope_idents.push(Arc::new(ident));
          }
        }

        let ident: String = m.identifier().to_string();
        inlined_modules_to_info.insert(
          ident,
          InlinedModuleInfo {
            source: code,
            module_scope_idents,
            used_in_non_inlined: Vec::new(),
          },
        );
      } else {
        for ident in collector.ids {
          if ident.id.span.ctxt == global_ctxt {
            all_used_names.insert(ident.id.sym.to_string());
            non_inlined_module_through_idents.push(ident);
          }
        }
      }
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
        .find(|m| m.identifier().to_string() == *module_id)
        .expect("should find the inlined module in all_modules");

      let mut source: Arc<dyn Source> = Arc::clone(_source);

      if used_in_non_inlined.is_empty() {
        renamed_inline_modules.insert(module_id.to_string(), source);
        continue;
      }

      let mut binding_to_ref: HashMap<(Atom, SyntaxContext), Vec<ConcatenatedModuleIdent>> =
        HashMap::default();

      for module_scope_ident in module_scope_idents.iter() {
        match binding_to_ref.entry((
          module_scope_ident.id.sym.clone(),
          module_scope_ident.id.span.ctxt,
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
          let new_name = find_new_name(name, &all_used_names, None, &readable_identifier);

          let mut replace_source = ReplaceSource::new(Arc::clone(&source));
          for identifier in refs.iter() {
            let span = identifier.id.span();
            let low = span.real_lo();
            let high = span.real_hi();

            if identifier.shorthand {
              replace_source.insert(high, &format!(": {}", new_name), None);
              continue;
            }

            replace_source.replace(low, high, &new_name, None);
          }

          source = Arc::new(replace_source);
          all_used_names.insert(new_name);
        }
      }

      renamed_inline_modules.insert(module_id.to_string(), Arc::clone(&source));
    }

    Ok(renamed_inline_modules)
  }

  pub async fn render_chunk(
    &self,
    compilation: &Compilation,
    chunk_ukey: &ChunkUkey,
  ) -> Result<BoxSource> {
    let hooks = Self::get_compilation_hooks(compilation);
    let module_graph = &compilation.get_module_graph();
    let is_module = compilation.options.output.module;
    let mut all_strict = compilation.options.output.module;
    let chunk_modules = compilation.chunk_graph.get_chunk_modules_by_source_type(
      chunk_ukey,
      SourceType::JavaScript,
      module_graph,
    );
    let mut sources = ConcatSource::default();
    if !all_strict
      && chunk_modules.iter().all(|m| {
        let build_info = m
          .build_info()
          .expect("should have build_info in render_main");
        build_info.strict
      })
    {
      if let Some(strict_bailout) = hooks.strict_runtime_bailout.call(compilation, chunk_ukey)? {
        sources.add(RawSource::from(format!(
          "// runtime can't be in strict mode because {strict_bailout}.\n"
        )));
      } else {
        sources.add(RawSource::from("\"use strict\";\n"));
        all_strict = true;
      }
    }
    let (chunk_modules_source, chunk_init_fragments) =
      render_chunk_modules(compilation, chunk_ukey, &chunk_modules, all_strict)?
        .unwrap_or_else(|| (RawSource::from("{}").boxed(), Vec::new()));
    let mut render_source = RenderSource {
      source: chunk_modules_source,
    };
    hooks
      .render_chunk
      .call(compilation, chunk_ukey, &mut render_source)?;
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
      .call(compilation, chunk_ukey, &mut render_source)?;
    sources.add(render_source.source);
    if !is_module {
      sources.add(RawSource::from(";"));
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
    let hooks = Self::get_compilation_hooks(compilation);
    hooks.chunk_hash.call(compilation, chunk_ukey, hasher).await
  }

  #[inline]
  pub fn update_hash_with_bootstrap(
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
    } = self.render_bootstrap(chunk_ukey, compilation)?;
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
  header: Vec<Cow<'a, str>>,
  startup: Vec<Cow<'a, str>>,
  allow_inline_startup: bool,
}
