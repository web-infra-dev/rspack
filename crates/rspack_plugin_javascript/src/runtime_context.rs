use std::sync::LazyLock;

use rspack_core::{
  ChunkKind, ChunkUkey, Compilation, RuntimeCodeTemplate, RuntimeGlobals, RuntimeGlobalsRenderMode,
  RuntimeProxyMetadata, RuntimeVariable, SourceType, property_access, render_lexical_declarations,
  rspack_sources::{BoxSource, ConcatSource, RawStringSource, SourceExt},
  runtime_module_owned_define_fields,
};
use rspack_error::Result;

use crate::{
  runtime::{RuntimeModuleSourceItem, render_runtime_module_sources},
  runtime_bridge::{
    render_runtime_bridge, render_runtime_bridge_aliases, render_runtime_bridge_fields,
  },
};

static HMR_RUNTIME_STATE_GLOBALS: LazyLock<RuntimeGlobals> = LazyLock::new(|| {
  RuntimeGlobals::HMR_DOWNLOAD_UPDATE_HANDLERS
    | RuntimeGlobals::HMR_INVALIDATE_MODULE_HANDLERS
    | RuntimeGlobals::HMR_MODULE_DATA
    | RuntimeGlobals::HMR_RUNTIME_STATE_PREFIX
});

static LIVE_BINDING_CONTEXT_GLOBALS: LazyLock<RuntimeGlobals> = LazyLock::new(|| {
  RuntimeGlobals::PUBLIC_PATH
    | RuntimeGlobals::SCRIPT_NONCE
    | RuntimeGlobals::GET_CHUNK_SCRIPT_FILENAME
    | RuntimeGlobals::SHARE_SCOPE_MAP
    | RuntimeGlobals::INITIALIZE_SHARING
    | RuntimeGlobals::CURRENT_REMOTE_GET_SCOPE
});

static BOOTSTRAP_EXPORT_GLOBALS: LazyLock<RuntimeGlobals> = LazyLock::new(|| {
  RuntimeGlobals::REQUIRE
    | RuntimeGlobals::REQUIRE_SCOPE
    | RuntimeGlobals::MODULE_FACTORIES
    | RuntimeGlobals::MODULE_CACHE
    | RuntimeGlobals::DEFERRED_MODULE_EXPORTS
    | RuntimeGlobals::INTERCEPT_MODULE_EXECUTION
});

fn runtime_iife_end(runtime_requirements: RuntimeGlobals) -> &'static str {
  if runtime_requirements.contains(RuntimeGlobals::GLOBAL) {
    "\n}).call(this);\n"
  } else {
    "\n})();\n"
  }
}

pub fn should_export_rspack_runtime_globals(
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
) -> bool {
  let chunk_graph = &compilation.build_chunk_graph_artifact.chunk_graph;
  if chunk_graph.get_chunk_entry_modules(chunk_ukey).is_empty() {
    return true;
  }

  let chunk = compilation
    .build_chunk_graph_artifact
    .chunk_by_ukey
    .expect_get(chunk_ukey);
  let module_graph = compilation.get_module_graph();
  chunk
    .get_all_referenced_chunks(&compilation.build_chunk_graph_artifact.chunk_group_by_ukey)
    .into_iter()
    .any(|referenced_chunk| {
      referenced_chunk != *chunk_ukey
        && chunk_graph.has_chunk_module_by_source_type(
          &referenced_chunk,
          SourceType::JavaScript,
          module_graph,
        )
    })
}

fn filter_unused_module_runtime_bindings(
  mut fields: RuntimeGlobals,
  tree_runtime_requirements: RuntimeGlobals,
  bootstrap_runtime_requirements: RuntimeGlobals,
) -> RuntimeGlobals {
  fields = fields.renderable_require_scope();

  let use_require = bootstrap_runtime_requirements.intersects(
    RuntimeGlobals::REQUIRE | RuntimeGlobals::INTERCEPT_MODULE_EXECUTION | RuntimeGlobals::MODULE,
  );
  let module_cache = tree_runtime_requirements.contains(RuntimeGlobals::MODULE_CACHE);
  let render_module_cache =
    use_require || bootstrap_runtime_requirements.contains(RuntimeGlobals::MODULE_CACHE);
  if !module_cache || !render_module_cache {
    fields.remove(RuntimeGlobals::MODULE_CACHE);
  }

  let uses_module_factories = tree_runtime_requirements
    .intersects(RuntimeGlobals::MODULE_FACTORIES | RuntimeGlobals::MODULE_FACTORIES_ADD_ONLY);
  let renders_module_factories = bootstrap_runtime_requirements.intersects(
    RuntimeGlobals::MODULE_FACTORIES
      | RuntimeGlobals::MODULE_FACTORIES_ADD_ONLY
      | RuntimeGlobals::REQUIRE,
  );
  if !uses_module_factories || !renders_module_factories {
    fields.remove(RuntimeGlobals::MODULE_FACTORIES);
  } else if tree_runtime_requirements.contains(RuntimeGlobals::MODULE_FACTORIES_ADD_ONLY) {
    fields.insert(RuntimeGlobals::MODULE_FACTORIES);
  }

  fields
}

fn render_rspack_runtime_exports(
  fields: RuntimeGlobals,
  setter_fields: RuntimeGlobals,
  should_export: bool,
  runtime_template: &RuntimeCodeTemplate,
  emitted_fields: &mut RuntimeGlobals,
  emitted_setters: &mut RuntimeGlobals,
) -> String {
  let mut source = String::new();

  if should_export {
    for (_, runtime_global) in fields
      .renderable_require_scope()
      .difference(*BOOTSTRAP_EXPORT_GLOBALS)
      .iter_names()
    {
      if emitted_fields.contains(runtime_global) {
        continue;
      }
      emitted_fields.insert(runtime_global);
      let specifier = runtime_template.render_runtime_globals(&runtime_global);
      source.push_str(&format!("export {{ {specifier} }};\n"));
    }
  }

  for (_, runtime_global) in setter_fields.renderable_require_scope().iter_names() {
    if emitted_setters.contains(runtime_global) {
      continue;
    }
    let Some(setter_name) = runtime_global.to_rspack_export_setter_name() else {
      continue;
    };
    emitted_setters.insert(runtime_global);
    let lexical_name = runtime_template.render_runtime_globals(&runtime_global);
    source.push_str(&format!(
      "function {setter_name}(value) {{ return {lexical_name} = value; }}\n"
    ));
    if should_export {
      source.push_str(&format!("export {{ {setter_name} }};\n"));
    }
  }

  source
}

fn render_runtime_context_accessors(
  fields: RuntimeGlobals,
  runtime_context: &str,
  runtime_template: &RuntimeCodeTemplate,
) -> String {
  let mut source = String::new();
  for (_, runtime_global) in fields.renderable_require_scope().iter_names() {
    let Some(key) = runtime_global.rspack_context_property_name() else {
      continue;
    };
    let lexical_name = runtime_template.render_runtime_globals(&runtime_global);
    source.push_str(&format!(
      "Object.defineProperty({runtime_context}, {}, {{ configurable: true, get: function() {{ return {lexical_name}; }}, set: function(value) {{ {lexical_name} = value; }} }});\n",
      rspack_util::json_stringify(key),
    ));
  }
  source
}

pub fn render_runtime_context_declaration(runtime_template: &RuntimeCodeTemplate) -> String {
  let runtime_context = runtime_template.render_runtime_variable(&RuntimeVariable::Context);
  format!("var {runtime_context}={{}};\n")
}

pub fn render_runtime_context_require_assignment(runtime_template: &RuntimeCodeTemplate) -> String {
  format!(
    "{} = {};\n",
    runtime_template.render_runtime_globals(&RuntimeGlobals::REQUIRE),
    runtime_template.render_runtime_variable(&RuntimeVariable::Require)
  )
}

pub async fn render_runtime_chunk_runtime_modules(
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  runtime_template: &RuntimeCodeTemplate,
) -> Result<BoxSource> {
  let runtime_module_sources = render_runtime_module_sources(compilation, chunk_ukey).await?;
  Ok(render_runtime_chunk_runtime_modules_sync(
    compilation,
    chunk_ukey,
    runtime_template,
    runtime_module_sources,
  ))
}

#[inline(never)]
fn render_runtime_chunk_runtime_modules_sync(
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  runtime_template: &RuntimeCodeTemplate,
  runtime_module_sources: Vec<RuntimeModuleSourceItem>,
) -> BoxSource {
  let mut sources = ConcatSource::default();
  let metadata = compilation
    .runtime_proxy_metadata_artifact
    .get(chunk_ukey)
    .expect("should generate runtime metadata");
  if runtime_module_sources.is_empty() && !metadata.runtime_module_bridge.owner {
    return sources.boxed();
  }
  let runtime_context = runtime_template.render_runtime_variable(&RuntimeVariable::Context);
  let module_graph = compilation.get_module_graph();
  let has_modules = !compilation
    .build_chunk_graph_artifact
    .chunk_graph
    .get_chunk_modules_identifier_by_source_type(chunk_ukey, SourceType::JavaScript, module_graph)
    .is_empty();
  let is_hmr_runtime = metadata
    .tree_runtime_requirements
    .contains(RuntimeGlobals::HMR_DOWNLOAD_MANIFEST);
  let should_export_runtime_globals = should_export_rspack_runtime_globals(compilation, chunk_ukey);
  let mut hmr_state_keys = Vec::new();
  for runtime_module_id in compilation
    .build_chunk_graph_artifact
    .chunk_graph
    .get_chunk_runtime_modules_iterable(chunk_ukey)
  {
    let runtime_module = compilation
      .runtime_modules
      .get(runtime_module_id)
      .expect("should have runtime module");
    let Some(key) = (match runtime_module.get_constructor_name().as_str() {
      "JsonpChunkLoadingRuntimeModule" => Some("jsonp"),
      "ModuleChunkLoadingRuntimeModule" => Some("module"),
      "ImportScriptsChunkLoadingRuntimeModule" => Some("importScripts"),
      "ReadFileChunkLoadingRuntimeModule" => Some("readFileVm"),
      "RequireChunkLoadingRuntimeModule" => Some("require"),
      _ => None,
    }) else {
      continue;
    };
    hmr_state_keys.push(key);
  }

  let isolate = has_modules;
  let render_runtime_global = |runtime_global: RuntimeGlobals| {
    if runtime_global == RuntimeGlobals::REQUIRE {
      Some(runtime_template.render_runtime_variable(&RuntimeVariable::Require))
    } else if runtime_global == RuntimeGlobals::MODULE_FACTORIES {
      Some(runtime_template.render_runtime_variable(&RuntimeVariable::Modules))
    } else if runtime_global == RuntimeGlobals::MODULE_CACHE {
      let module_cache = runtime_template.render_runtime_variable(&RuntimeVariable::ModuleCache);
      Some(format!(
        "typeof {module_cache} !== \"undefined\" ? {module_cache} : {{}}"
      ))
    } else if runtime_global == RuntimeGlobals::DEFERRED_MODULE_EXPORTS {
      Some("__rspack_deferred_exports".to_string())
    } else if runtime_template.render_mode() == RuntimeGlobalsRenderMode::RspackContext
      && runtime_global.intersects(RuntimeGlobals::STARTUP | RuntimeGlobals::STARTUP_ENTRYPOINT)
    {
      runtime_global
        .rspack_context_property_name()
        .map(|property_name| format!("{runtime_context}{}", property_access([property_name], 0)))
    } else {
      None
    }
  };
  let mut wrapped_sources = ConcatSource::default();
  let bootstrap_runtime_requirements = compilation
    .cgc_runtime_requirements_artifact
    .get(chunk_ukey)
    .copied()
    .unwrap_or_default();
  let lexical_fields = filter_unused_module_runtime_bindings(
    metadata.lexical_fields() | metadata.context_setter_fields(),
    metadata.tree_runtime_requirements,
    bootstrap_runtime_requirements,
  );
  let mut pending_bridge_fields = RuntimeGlobals::default();
  let mut bridge_prelude = String::new();
  if metadata.runtime_module_bridge.owner {
    let provided = runtime_module_sources
      .iter()
      .fold(RuntimeGlobals::default(), |fields, item| {
        fields | item.requirements.define
      });
    // Only publish installed helpers, never an optional weak dependency.
    let initial = lexical_fields
      .intersection(bootstrap_runtime_requirements)
      .difference(provided.difference(*BOOTSTRAP_EXPORT_GLOBALS));
    pending_bridge_fields = lexical_fields.intersection(provided).difference(initial);
    bridge_prelude = render_runtime_bridge(runtime_template);
    bridge_prelude.push_str(&render_runtime_bridge_fields(initial, runtime_template));
  }
  if runtime_template.render_mode() == RuntimeGlobalsRenderMode::RspackExport {
    sources.add(RawStringSource::from(render_lexical_declarations(
      lexical_fields
        .difference(runtime_module_owned_define_fields(compilation, chunk_ukey))
        .difference(*BOOTSTRAP_EXPORT_GLOBALS),
      Some(&render_runtime_global),
    )));
    if metadata.runtime_module_bridge.owner {
      sources.add(RawStringSource::from(bridge_prelude));
    }
    if is_hmr_runtime {
      let require = runtime_template.render_runtime_variable(&RuntimeVariable::Require);
      let modules = runtime_template.render_runtime_globals(&RuntimeGlobals::MODULE_FACTORIES);
      let module_cache = runtime_template.render_runtime_variable(&RuntimeVariable::ModuleCache);
      if metadata.runtime_module_bridge.owner {
        sources.add(RawStringSource::from(format!(
          "var {runtime_context}=__rspack_context;\n"
        )));
      } else {
        sources.add(RawStringSource::from(format!(
        "var {runtime_context}={require};{runtime_context}.r={require};{runtime_context}.m={modules};{runtime_context}.c={module_cache};\n"
      )));
        sources.add(RawStringSource::from(render_runtime_context_accessors(
          (metadata.context_fields() | metadata.context_setter_fields()).difference(
            RuntimeGlobals::REQUIRE
              | RuntimeGlobals::REQUIRE_SCOPE
              | RuntimeGlobals::MODULE_FACTORIES
              | RuntimeGlobals::MODULE_CACHE,
          ),
          &runtime_context,
          runtime_template,
        )));
      }
    }
    if metadata
      .lexical_fields()
      .intersects(*HMR_RUNTIME_STATE_GLOBALS)
    {
      for key in &hmr_state_keys {
        let state_expression = runtime_template.render_hmr_runtime_state_expression(key);
        sources.add(RawStringSource::from(format!("var {state_expression};\n")));
      }
    }

    let mut emitted_fields = RuntimeGlobals::default();
    let mut emitted_setters = RuntimeGlobals::default();
    let setters = metadata.context_setter_fields();
    let mut remaining_setters = setters;
    for item in runtime_module_sources {
      let install = pending_bridge_fields.intersection(item.requirements.define);
      if !install.is_empty() {
        sources.add(RawStringSource::from(render_runtime_bridge_fields(
          install,
          runtime_template,
        )));
        pending_bridge_fields.remove(install);
      }
      sources.add(item.source);
      let mut context_fields = metadata
        .context_fields()
        .intersection(item.requirements.define | item.requirements.force_context);
      if is_hmr_runtime {
        context_fields.insert(
          item
            .requirements
            .lexical_requirements()
            .renderable_require_scope(),
        );
        context_fields.remove(RuntimeGlobals::REQUIRE | RuntimeGlobals::REQUIRE_SCOPE);
      }
      let context_setters = context_fields.intersection(setters);
      remaining_setters.remove(context_setters);
      sources.add(RawStringSource::from(render_rspack_runtime_exports(
        context_fields,
        context_setters,
        should_export_runtime_globals,
        runtime_template,
        &mut emitted_fields,
        &mut emitted_setters,
      )));
    }
    sources.add(RawStringSource::from(render_rspack_runtime_exports(
      remaining_setters,
      remaining_setters,
      should_export_runtime_globals,
      runtime_template,
      &mut emitted_fields,
      &mut emitted_setters,
    )));

    if metadata.runtime_module_bridge.owner {
      let require = runtime_template.render_runtime_variable(&RuntimeVariable::Require);
      sources.add(RawStringSource::from(format!(
        "if ({require} !== __webpack_require__) {{ {require} = __webpack_require__; if ({require}.__rspack_compat_context__ !== __rspack_context) Object.defineProperty({require}, '__rspack_compat_context__', {{value: __rspack_context}}); }}\n"
      )));
    }
    return sources.boxed();
  }
  wrapped_sources.add(RawStringSource::from(render_lexical_declarations(
    lexical_fields.difference(runtime_module_owned_define_fields(compilation, chunk_ukey)),
    Some(&render_runtime_global),
  )));
  if metadata.runtime_module_bridge.owner {
    wrapped_sources.add(RawStringSource::from(bridge_prelude));
  }
  if metadata
    .lexical_fields()
    .intersects(*HMR_RUNTIME_STATE_GLOBALS)
  {
    for key in &hmr_state_keys {
      wrapped_sources.add(RawStringSource::from(format!("var hmrS_{key};\n")));
    }
  }
  for item in runtime_module_sources {
    let install = pending_bridge_fields.intersection(item.requirements.define);
    if !install.is_empty() {
      wrapped_sources.add(RawStringSource::from(render_runtime_bridge_fields(
        install,
        runtime_template,
      )));
      pending_bridge_fields.remove(install);
    }
    if isolate && item.needs_top_level {
      sources.add(item.source);
    } else {
      wrapped_sources.add(item.source);
    }
    let mut context_fields = metadata
      .context_fields()
      .intersection(item.requirements.define | item.requirements.force_context);
    if is_hmr_runtime {
      context_fields.insert(
        item
          .requirements
          .lexical_requirements()
          .renderable_require_scope(),
      );
      context_fields.remove(RuntimeGlobals::REQUIRE | RuntimeGlobals::REQUIRE_SCOPE);
    }
    let setters = metadata.context_setter_fields();
    if metadata.runtime_module_bridge.owner {
      continue;
    }
    if context_fields.is_empty() {
      continue;
    }
    for (_, runtime_global) in context_fields.iter_names() {
      let (Some(key), Some(lexical_name)) = (
        runtime_global.rspack_context_property_name(),
        runtime_global.to_lexical_name(),
      ) else {
        continue;
      };
      if setters.contains(runtime_global)
        && (is_hmr_runtime || LIVE_BINDING_CONTEXT_GLOBALS.contains(runtime_global))
      {
        wrapped_sources.add(RawStringSource::from(format!(
          "Object.defineProperty({}, {}, {{ configurable: true, get: function() {{ return {}; }}, set: function(value) {{ {} = value; }} }});\n",
          runtime_context,
          rspack_util::json_stringify(key),
          lexical_name,
          lexical_name
        )));
      } else {
        wrapped_sources.add(RawStringSource::from(format!(
          "{}{} = {};\n",
          runtime_context,
          property_access([key], 0),
          lexical_name
        )));
      }
    }
  }
  // A runtime extension can wrap the loader itself. Native runtime modules and
  // later chunks must observe the same callable as the extension after startup.
  if metadata.runtime_module_bridge.owner {
    let require = runtime_template.render_runtime_variable(&RuntimeVariable::Require);
    wrapped_sources.add(RawStringSource::from(format!(
      "{require} = __webpack_require__;\n"
    )));
  }
  if metadata.runtime_module_bridge.owner
    && bootstrap_runtime_requirements.contains(RuntimeGlobals::INTERCEPT_MODULE_EXECUTION)
  {
    wrapped_sources.add(RawStringSource::from_static(include_str!(
      "runtime/webpack_runtime_bridge_interceptors.js"
    )));
  }
  if isolate {
    sources.add(RawStringSource::from_static("(function() {\n"));
    sources.add(wrapped_sources);
    sources.add(RawStringSource::from_static(runtime_iife_end(
      metadata.tree_runtime_requirements,
    )));
  } else {
    sources.add(wrapped_sources);
  }

  sources.boxed()
}

pub async fn render_chunk_runtime_modules(
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  runtime_template: &RuntimeCodeTemplate,
) -> Result<BoxSource> {
  let runtime_module_sources = render_runtime_module_sources(compilation, chunk_ukey).await?;
  let mut sources = ConcatSource::default();
  if runtime_module_sources.is_empty() {
    return Ok(sources.boxed());
  }
  let metadata = compilation
    .runtime_proxy_metadata_artifact
    .get(chunk_ukey)
    .expect("should generate runtime metadata");
  let runtime_context = runtime_template.render_runtime_variable(&RuntimeVariable::Context);
  let is_hmr_runtime = metadata
    .tree_runtime_requirements
    .contains(RuntimeGlobals::HMR_DOWNLOAD_MANIFEST);

  sources.add(RawStringSource::from_static("(function() {\n"));
  if metadata.runtime_module_bridge.source == rspack_core::RuntimeModuleSourceKind::NeedsBridge {
    sources.add(RawStringSource::from(render_runtime_bridge_aliases(
      runtime_template,
    )));
  }
  let render_context_field = |runtime_global: RuntimeGlobals| {
    runtime_global
      .rspack_context_property_name()
      .map(|property_name| {
        let value = format!("{runtime_context}{}", property_access([property_name], 0));
        if runtime_global.should_initialize_as_object() {
          format!("{value}||{{}}")
        } else if runtime_global.should_initialize_as_array() {
          format!("{value}||[]")
        } else {
          value
        }
      })
  };
  let render_runtime_global = |runtime_global: RuntimeGlobals| render_context_field(runtime_global);
  sources.add(RawStringSource::from(render_lexical_declarations(
    (metadata.lexical_fields() | metadata.context_setter_fields())
      .difference(runtime_module_owned_define_fields(compilation, chunk_ukey)),
    Some(&render_runtime_global),
  )));

  for item in runtime_module_sources {
    sources.add(item.source);
    let mut context_fields = metadata
      .context_fields()
      .intersection(item.requirements.define | item.requirements.force_context);
    if is_hmr_runtime {
      context_fields.insert(
        item
          .requirements
          .lexical_requirements()
          .renderable_require_scope(),
      );
      context_fields.remove(RuntimeGlobals::REQUIRE | RuntimeGlobals::REQUIRE_SCOPE);
    }

    let setters = metadata.context_setter_fields();
    for (_, runtime_global) in context_fields.iter_names() {
      let (Some(key), Some(lexical_name)) = (
        runtime_global.rspack_context_property_name(),
        runtime_global.to_lexical_name(),
      ) else {
        continue;
      };
      if setters.contains(runtime_global)
        && (is_hmr_runtime || LIVE_BINDING_CONTEXT_GLOBALS.contains(runtime_global))
      {
        sources.add(RawStringSource::from(format!(
          "Object.defineProperty({}, {}, {{ configurable: true, get: function() {{ return {}; }}, set: function(value) {{ {} = value; }} }});\n",
          runtime_context,
          rspack_util::json_stringify(key),
          lexical_name,
          lexical_name
        )));
      } else {
        sources.add(RawStringSource::from(format!(
          "{}{} = {};\n",
          runtime_context,
          property_access([key], 0),
          lexical_name
        )));
      }
    }
  }

  sources.add(RawStringSource::from_static(runtime_iife_end(
    metadata.tree_runtime_requirements,
  )));

  Ok(sources.boxed())
}

pub async fn render_hot_update_chunk_runtime_modules(
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  runtime_template: &RuntimeCodeTemplate,
) -> Result<BoxSource> {
  let runtime_module_sources = render_runtime_module_sources(compilation, chunk_ukey).await?;
  let mut sources = ConcatSource::default();
  if runtime_module_sources.is_empty() {
    return Ok(sources.boxed());
  }
  let metadata = runtime_context_current_chunk_metadata(compilation, chunk_ukey);

  let runtime_context = runtime_template.render_runtime_argument();
  if metadata.runtime_module_bridge.source == rspack_core::RuntimeModuleSourceKind::NeedsBridge {
    sources.add(RawStringSource::from(render_runtime_bridge_aliases(
      runtime_template,
    )));
  }
  let mut hmr_state_keys = Vec::new();
  for runtime_module_id in compilation
    .build_chunk_graph_artifact
    .chunk_graph
    .get_chunk_runtime_modules_iterable(chunk_ukey)
  {
    let runtime_module = compilation
      .runtime_modules
      .get(runtime_module_id)
      .expect("should have runtime module");
    let Some(key) = (match runtime_module.get_constructor_name().as_str() {
      "JsonpChunkLoadingRuntimeModule" => Some("jsonp"),
      "ModuleChunkLoadingRuntimeModule" => Some("module"),
      "ImportScriptsChunkLoadingRuntimeModule" => Some("importScripts"),
      "ReadFileChunkLoadingRuntimeModule" => Some("readFileVm"),
      "RequireChunkLoadingRuntimeModule" => Some("require"),
      _ => None,
    }) else {
      continue;
    };
    hmr_state_keys.push(key);
  }

  let render_context_field = |runtime_global: RuntimeGlobals| {
    runtime_global
      .rspack_context_property_name()
      .map(|property_name| {
        let value = format!("{runtime_context}{}", property_access([property_name], 0));
        if runtime_global.should_initialize_as_object() {
          format!("{value}||{{}}")
        } else if runtime_global.should_initialize_as_array() {
          format!("{value}||[]")
        } else {
          value
        }
      })
  };
  sources.add(RawStringSource::from(render_lexical_declarations(
    (metadata.lexical_fields() | metadata.context_setter_fields())
      .difference(runtime_module_owned_define_fields(compilation, chunk_ukey)),
    Some(&render_context_field),
  )));
  if metadata
    .lexical_fields()
    .intersects(*HMR_RUNTIME_STATE_GLOBALS)
  {
    for key in &hmr_state_keys {
      let state_expression =
        if runtime_template.render_mode() == RuntimeGlobalsRenderMode::RspackExport {
          runtime_template.render_hmr_runtime_state_expression(key)
        } else {
          format!("hmrS_{key}")
        };
      sources.add(RawStringSource::from(format!("var {state_expression};\n")));
    }
  }

  let require = runtime_template.render_runtime_variable(&RuntimeVariable::Require);
  let modules = runtime_template.render_runtime_variable(&RuntimeVariable::Modules);
  let module_cache = runtime_template.render_runtime_variable(&RuntimeVariable::ModuleCache);
  sources.add(RawStringSource::from(format!(
    "var {require}={runtime_context}.r,{modules}={runtime_context}.m,{module_cache}={runtime_context}.c;\n"
  )));

  for item in runtime_module_sources {
    sources.add(item.source);
    let mut context_fields = metadata
      .context_fields()
      .intersection(item.requirements.lexical_requirements());
    context_fields.insert(
      item
        .requirements
        .lexical_requirements()
        .renderable_require_scope(),
    );
    context_fields.remove(RuntimeGlobals::REQUIRE | RuntimeGlobals::REQUIRE_SCOPE);
    if context_fields.is_empty() {
      continue;
    }
    for (_, runtime_global) in context_fields.iter_names() {
      let (Some(key), Some(lexical_name)) = (
        runtime_global.rspack_context_property_name(),
        runtime_global.to_lexical_name(),
      ) else {
        continue;
      };
      let json_key = rspack_util::json_stringify(key);
      let context_property = property_access([key], 0);
      // The module source may end with a line comment; a leading newline keeps
      // the injected IIFE out of it.
      sources.add(RawStringSource::from(format!(
        r#"
;(function() {{
var oldSetter = Object.getOwnPropertyDescriptor({runtime_context}, {json_key}) && Object.getOwnPropertyDescriptor({runtime_context}, {json_key}).set;
Object.defineProperty({runtime_context}, {json_key}, {{ configurable: true, get: function() {{ return {lexical_name}; }}, set: function(value) {{ {lexical_name} = value; if (oldSetter) oldSetter.call({runtime_context}, value); }} }});
{runtime_context}{context_property} = {lexical_name};
}})();
"#
      )));
    }
  }

  Ok(sources.boxed())
}

pub async fn render_rspack_runtime_modules(
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  runtime_template: &RuntimeCodeTemplate,
) -> Result<BoxSource> {
  let chunk = compilation
    .build_chunk_graph_artifact
    .chunk_by_ukey
    .expect_get(chunk_ukey);
  if matches!(chunk.kind(), ChunkKind::HotUpdate) {
    render_hot_update_chunk_runtime_modules(compilation, chunk_ukey, runtime_template).await
  } else if chunk.has_runtime(&compilation.build_chunk_graph_artifact.chunk_group_by_ukey) {
    render_runtime_chunk_runtime_modules(compilation, chunk_ukey, runtime_template).await
  } else {
    render_chunk_runtime_modules(compilation, chunk_ukey, runtime_template).await
  }
}

fn runtime_context_current_chunk_metadata(
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
) -> RuntimeProxyMetadata {
  let mut metadata = compilation
    .runtime_proxy_metadata_artifact
    .get(chunk_ukey)
    .cloned()
    .unwrap_or_default();
  if let Some(chunk_runtime_requirements) = compilation
    .cgc_runtime_requirements_artifact
    .get(chunk_ukey)
  {
    metadata
      .tree_runtime_requirements
      .insert(*chunk_runtime_requirements);
    metadata
      .runtime_module_requirements
      .insert(*chunk_runtime_requirements);
  }
  for runtime_module_id in compilation
    .build_chunk_graph_artifact
    .chunk_graph
    .get_chunk_runtime_modules_iterable(chunk_ukey)
  {
    let runtime_module = compilation
      .runtime_modules
      .get(runtime_module_id)
      .expect("should have runtime module");
    let module_runtime_requirements = runtime_module.runtime_requirements(compilation);
    metadata
      .tree_runtime_requirements
      .insert(module_runtime_requirements.lexical_requirements());
    metadata
      .runtime_module_requirements
      .insert(module_runtime_requirements.dependencies);
    metadata
      .force_context_fields
      .insert(module_runtime_requirements.force_context);
  }

  metadata
}
