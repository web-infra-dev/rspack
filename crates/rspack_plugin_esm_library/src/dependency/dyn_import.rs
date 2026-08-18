use std::{borrow::Cow, sync::Arc};

use atomic_refcell::AtomicRefCell;
use rspack_collections::IdentifierMap;
use rspack_core::{
  Dependency, DependencyId, DependencyTemplate, ExportsType, ExternalModule,
  FakeNamespaceObjectMode, ModuleGraph, ModuleReferenceOptions, RuntimeGlobals, TemplateContext,
  get_exports_type, property_access,
};
use rspack_plugin_javascript::dependency::ImportDependency;
use rspack_plugin_rslib::dyn_import_external::render_dyn_import_external_module;
use rspack_util::atom::Atom;

use crate::EsmLibraryPlugin;

fn then_expr(
  code_generatable_context: &mut TemplateContext,
  dep_id: &DependencyId,
  request: &str,
) -> String {
  let TemplateContext {
    compilation,
    module,
    runtime_template,
    ..
  } = code_generatable_context;
  if compilation
    .get_module_graph()
    .module_identifier_by_dependency_id(dep_id)
    .is_none()
  {
    return runtime_template.missing_module_promise(request);
  };

  let exports_type = get_exports_type(
    compilation.get_module_graph(),
    &compilation.module_graph_cache_artifact,
    &compilation.exports_info_artifact,
    dep_id,
    &module.identifier(),
  );
  let module_id_expr = runtime_template.module_id(compilation, dep_id, request, false);

  let mut fake_type = FakeNamespaceObjectMode::PROMISE_LIKE;
  let mut appending;

  match exports_type {
    ExportsType::Namespace => {
      appending = format!(
        ".then({}.bind({}, {module_id_expr}))",
        runtime_template.render_runtime_globals(&RuntimeGlobals::REQUIRE),
        runtime_template.render_runtime_globals(&RuntimeGlobals::REQUIRE),
      );
    }
    _ => {
      if matches!(exports_type, ExportsType::Dynamic) {
        fake_type |= FakeNamespaceObjectMode::RETURN_VALUE;
      }
      if matches!(
        exports_type,
        ExportsType::DefaultWithNamed | ExportsType::Dynamic
      ) {
        fake_type |= FakeNamespaceObjectMode::MERGE_PROPERTIES;
      }
      if ModuleGraph::is_async(
        &compilation.async_modules_artifact,
        compilation
          .get_module_graph()
          .module_identifier_by_dependency_id(dep_id)
          .expect("should have module"),
      ) {
        appending = format!(
          ".then({}.bind({}, {module_id_expr}))",
          runtime_template.render_runtime_globals(&RuntimeGlobals::REQUIRE),
          runtime_template.render_runtime_globals(&RuntimeGlobals::REQUIRE)
        );
        appending.push_str(
          format!(
            r#".then(function(m){{
 return {}(m, {fake_type})
}})"#,
            runtime_template.render_runtime_globals(&RuntimeGlobals::CREATE_FAKE_NAMESPACE_OBJECT)
          )
          .as_str(),
        );
      } else {
        fake_type |= FakeNamespaceObjectMode::MODULE_ID;
        appending = format!(
          ".then({}.bind({}, {module_id_expr}, {fake_type}))",
          runtime_template.render_runtime_globals(&RuntimeGlobals::CREATE_FAKE_NAMESPACE_OBJECT),
          runtime_template.render_runtime_globals(&RuntimeGlobals::REQUIRE)
        );
      }
    }
  }
  appending
}

fn get_fake_namespace_object_mode(
  code_generatable_context: &TemplateContext,
  dep_id: &DependencyId,
) -> FakeNamespaceObjectMode {
  let exports_type = get_exports_type(
    code_generatable_context.compilation.get_module_graph(),
    &code_generatable_context
      .compilation
      .module_graph_cache_artifact,
    &code_generatable_context.compilation.exports_info_artifact,
    dep_id,
    &code_generatable_context.module.identifier(),
  );
  let mut fake_type = FakeNamespaceObjectMode::PROMISE_LIKE;
  if matches!(exports_type, ExportsType::Dynamic) {
    fake_type |= FakeNamespaceObjectMode::RETURN_VALUE;
  }
  if matches!(
    exports_type,
    ExportsType::DefaultWithNamed | ExportsType::Dynamic
  ) {
    fake_type |= FakeNamespaceObjectMode::MERGE_PROPERTIES;
  }
  fake_type
}

fn render_lazy_require_external_import(
  create_fake_namespace_object: &str,
  request_expr: &str,
  properties: &str,
  fake_type: FakeNamespaceObjectMode,
) -> String {
  format!(
    "Promise.resolve().then(function() {{ return {create_fake_namespace_object}(require({request_expr}){properties}, {fake_type}) }})"
  )
}

fn render_lazy_create_require_external_import(
  code_generatable_context: &TemplateContext,
  create_fake_namespace_object: &str,
  request_expr: &str,
  properties: &str,
  fake_type: FakeNamespaceObjectMode,
) -> String {
  let need_prefix = code_generatable_context
    .compilation
    .options
    .output
    .environment
    .supports_node_prefix_for_core_modules();
  let import_meta_name = &code_generatable_context
    .compilation
    .options
    .output
    .import_meta_name;
  let module_request =
    rspack_util::json_stringify_str(if need_prefix { "node:module" } else { "module" });

  format!(
    "import({module_request}).then(function(module) {{ return {create_fake_namespace_object}(module.createRequire({import_meta_name}.url)({request_expr}){properties}, {fake_type}) }})"
  )
}

fn render_lazy_commonjs_external_import(
  code_generatable_context: &mut TemplateContext,
  external_module: &ExternalModule,
  fake_type: FakeNamespaceObjectMode,
) -> Option<String> {
  let request = external_module.get_request();
  let request_expr = rspack_util::json_stringify_str(request.primary());
  let properties = property_access(request.iter(), 1);
  let create_fake_namespace_object = code_generatable_context
    .runtime_template
    .render_runtime_globals(&RuntimeGlobals::CREATE_FAKE_NAMESPACE_OBJECT);

  match external_module.resolve_external_type() {
    "commonjs" | "commonjs2" | "commonjs-module" | "commonjs-static" | "node-commonjs" => {
      if code_generatable_context.compilation.options.output.module {
        Some(render_lazy_create_require_external_import(
          code_generatable_context,
          &create_fake_namespace_object,
          &request_expr,
          &properties,
          fake_type,
        ))
      } else {
        Some(render_lazy_require_external_import(
          &create_fake_namespace_object,
          &request_expr,
          &properties,
          fake_type,
        ))
      }
    }
    _ => None,
  }
}

#[derive(Debug)]
pub struct DynamicImportDependencyTemplate {
  /// module_id → namespace export name in the chunk.
  /// For modules whose exports were renamed in a multi-module chunk,
  /// the import needs `.then(m => m.<ns_name>)` to get the correct namespace.
  /// Written during link, read during code generation.
  pub dyn_import_ns_map: Arc<AtomicRefCell<IdentifierMap<Atom>>>,
}

impl DynamicImportDependencyTemplate {
  /// Renders the dynamic-import promise expression for the resolved target.
  ///
  /// Returns `None` when the branch has already written the replacement into
  /// `source` itself (a phase-aware external `import`/`module`, which manages
  /// its own callee). Otherwise returns the promise expression, which the
  /// caller finalizes with a single `source.replace` and the shared
  /// source-phase unwrap.
  fn render_import_expr(
    &self,
    import_dep: &ImportDependency,
    source: &mut rspack_core::TemplateReplaceSource,
    code_generatable_context: &mut rspack_core::TemplateContext,
  ) -> Option<String> {
    let dep = import_dep as &dyn Dependency;
    let dep_id = dep.id();
    let module_graph = code_generatable_context.compilation.get_module_graph();
    let request = dep
      .as_module_dependency()
      .expect("should be module dep")
      .request();

    let Some(ref_module) = module_graph.get_module_by_dependency_id(dep_id) else {
      return Some(
        code_generatable_context
          .runtime_template
          .missing_module_promise(request),
      );
    };

    if let Some(external_module) = ref_module.as_external_module()
      && matches!(external_module.resolve_external_type(), "import" | "module")
    {
      // `render_dyn_import_external_module` is phase-aware (it emits
      // `import.source(...)` / `import.defer(...)` for the matching phase), so
      // let it own the replacement and skip the shared unwrap.
      render_dyn_import_external_module(import_dep, external_module, source);
      return None;
    }
    if let Some(external_module) = ref_module.as_external_module() {
      let fake_type = get_fake_namespace_object_mode(code_generatable_context, dep_id);
      if let Some(external_import) =
        render_lazy_commonjs_external_import(code_generatable_context, external_module, fake_type)
      {
        return Some(external_import);
      }
    }

    let ref_chunk_ukey = match EsmLibraryPlugin::get_module_chunk(
      ref_module.identifier(),
      code_generatable_context.compilation,
    ) {
      Ok(c) => c,
      Err(e) => {
        tracing::warn!(error = %e, "failed to resolve module chunk for dynamic import target");
        return Some(
          code_generatable_context
            .runtime_template
            .missing_module_promise(request),
        );
      }
    };

    let orig_chunk = match EsmLibraryPlugin::get_module_chunk(
      *module_graph
        .get_parent_module(dep_id)
        .expect("should have parent module for import dep"),
      code_generatable_context.compilation,
    ) {
      Ok(c) => c,
      Err(e) => {
        tracing::warn!(error = %e, "failed to resolve module chunk for dynamic import source");
        return Some(
          code_generatable_context
            .runtime_template
            .missing_module_promise(request),
        );
      }
    };

    /*
    For:
    const { a, b } = await import('./refModule');
    const unknownImports = await import('./refModule');

    1. if refModule is in the same chunk
      a. if refModule is scope hoisted
        const { a, b } = await Promise.resolve().then(() => ({ a: __MODULE_REF_A, b: __MODULE_REF_B }));
      b. if refModule is not scope hoisted
        const { a, b } = await Promise.resolve().then(() => __rspack_require(./refModule));

    2. if refModule is in other chunks
      a. if refModule is scope hoisted and exports NOT renamed
        const { a, b } = await import('./ref-chunk');
      b. if refModule is scope hoisted and exports renamed (or namespace access)
        const { a, b } = await import('./ref-chunk').then(m => m.__ns_name);
      c. if refModule is not scope hoisted
        const { a, b } = await import('./ref-chunk').then(() => __rspack_require(./refModule));
    */
    let already_in_chunk = ref_chunk_ukey == orig_chunk;
    let ref_chunk = code_generatable_context
      .compilation
      .build_chunk_graph_artifact
      .chunk_by_ukey
      .expect_get(&ref_chunk_ukey);
    let import_promise = if already_in_chunk {
      Cow::Borrowed("Promise.resolve()")
    } else {
      Cow::Owned(format!(
        "import(\"__RSPACK_ESM_CHUNK_{}\")",
        ref_chunk.expect_id().as_str()
      ))
    };

    let Some(concatenation_scope) = &mut code_generatable_context.concatenation_scope else {
      // if we are not in a concatenation scope, then all its children are not scope hoisted as well
      // we can safely use __rspack_require to fetch module
      return Some(format!(
        "{import_promise}{}",
        then_expr(code_generatable_context, dep_id, request)
      ));
    };

    let is_ref_module_concatenated =
      concatenation_scope.is_module_concatenated(&ref_module.identifier());

    if !is_ref_module_concatenated {
      // if target is not in a concatenation scope, then all its children are not scope hoisted as well
      // we can safely use __rspack_require to fetch module
      return Some(format!(
        "{import_promise}{}",
        then_expr(code_generatable_context, dep_id, request)
      ));
    }

    if already_in_chunk {
      // Same chunk + scope hoisted: the module's variables are already in scope.
      // Use a namespace module reference so the link phase resolves it to the
      // module's namespace object (e.g., `Promise.resolve(dynamic_namespaceObject)`).
      let ns_ref = concatenation_scope.create_module_reference(
        &ref_module.identifier(),
        ModuleReferenceOptions {
          ids: vec![],
          call: false,
          direct_import: true,
          deferred_import: false,
          asi_safe: Some(true),
          ..Default::default()
        },
      );
      return Some(format!("Promise.resolve({ns_ref})"));
    }

    // Cross-chunk: check if the module needs namespace remapping (exports were renamed or namespace access)
    let ns_name = {
      let ns_map = self.dyn_import_ns_map.borrow();
      ns_map.get(&ref_module.identifier()).cloned()
    };

    if let Some(ns_name) = ns_name {
      // Module's exports were renamed in the chunk or accessed as namespace.
      // Use .then(m => m.<ns_name>) to get the correct module namespace.
      Some(format!("{import_promise}.then(m => m.{ns_name})"))
    } else {
      // Module's exports are not renamed in the chunk — direct import works.
      Some(import_promise.into_owned())
    }
  }
}

impl DependencyTemplate for DynamicImportDependencyTemplate {
  fn render(
    &self,
    dep: &dyn rspack_core::DependencyCodeGeneration,
    source: &mut rspack_core::TemplateReplaceSource,
    code_generatable_context: &mut rspack_core::TemplateContext,
  ) {
    let import_dep = dep
      .as_any()
      .downcast_ref::<ImportDependency>()
      .expect("ImportDependencyTemplate can only be applied to ImportDependency");

    let Some(mut content) = self.render_import_expr(import_dep, source, code_generatable_context)
    else {
      // The branch already wrote its replacement into `source`.
      return;
    };

    // Source-phase imports (e.g. `import.source('./add.wasm')`) resolve to the
    // module value itself — for WebAssembly that is a `WebAssembly.Module`,
    // exposed as the namespace `default`. Unwrap it here so callers receive the
    // value rather than the namespace object, matching the standard
    // dynamic-import template (`ImportDependencyTemplate` in rspack_plugin_javascript).
    if import_dep.get_phase().is_source() {
      content = format!(
        "{content}.then({})",
        code_generatable_context
          .runtime_template
          .returning_function("m[\"default\"]", "m")
      );
    }

    source.replace(import_dep.range.start, import_dep.range.end, content, None);
  }
}
