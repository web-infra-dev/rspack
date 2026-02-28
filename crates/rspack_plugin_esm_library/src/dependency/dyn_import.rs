use std::{borrow::Cow, sync::Arc};

use atomic_refcell::AtomicRefCell;
use rspack_collections::IdentifierMap;
use rspack_core::{
  ChunkUkey, Dependency, DependencyId, DependencyTemplate, ExportsType, FakeNamespaceObjectMode,
  ModuleGraph, RuntimeGlobals, TemplateContext, get_exports_type,
};
use rspack_plugin_javascript::dependency::ImportDependency;
use rspack_plugin_rslib::dyn_import_external::render_dyn_import_external_module;

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

#[derive(Debug)]
pub struct DynamicImportDependencyTemplate {
  /// module_id → facade_chunk_ukey. Shared with EsmLibraryPlugin.
  /// Written during optimize_chunks, read during code generation.
  pub facade_chunks: Arc<AtomicRefCell<IdentifierMap<ChunkUkey>>>,
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
    let dep = import_dep as &dyn Dependency;
    let dep_id = dep.id();
    let module_graph = code_generatable_context.compilation.get_module_graph();
    let request = dep
      .as_module_dependency()
      .expect("should be module dep")
      .request();

    let Some(ref_module) = module_graph.get_module_by_dependency_id(dep_id) else {
      let missing_promise = code_generatable_context
        .runtime_template
        .missing_module_promise(request);
      source.replace(
        import_dep.range.start,
        import_dep.range.end,
        &missing_promise,
        None,
      );
      return;
    };

    if let Some(external_module) = ref_module.as_external_module() {
      render_dyn_import_external_module(import_dep, external_module, source);
      return;
    }

    let source_chunk = EsmLibraryPlugin::get_module_chunk(
      ref_module.identifier(),
      code_generatable_context.compilation,
    );

    let orig_chunk = EsmLibraryPlugin::get_module_chunk(
      *module_graph
        .get_parent_module(dep_id)
        .expect("should have parent module for import dep"),
      code_generatable_context.compilation,
    );

    // If there's a facade chunk for this module, redirect the import to the facade.
    // The facade chunk is empty (only re-exports), so import() yields the correct namespace directly.
    let ref_chunk_ukey = {
      let facade_map = self.facade_chunks.borrow();
      facade_map
        .get(&ref_module.identifier())
        .copied()
        .unwrap_or(source_chunk)
    };

    /*
    For:
    const { a, b } = await import('./refModule');
    const unknownImports = await import('./refModule');

    1. if refModule is in the same chunk
      a. if refModule is scope hoisted
        const { a, b } = await Promise.resolve().then(() => ({ a: __MODULE_REF_A, b: __MODULE_REF_B }));
      b. if refModule is not scope hoisted
        const { a, b } = await Promise.resolve().then(() => __webpack_require__(./refModule));

    2. if refModule is in other chunks
      a. if refModule is scope hoisted
        const { a, b } = await import('./ref-chunk').then((ns) => ({ a: ns.a, b: ns.b }));
        const unknownImports = await import('./refModule').then((ns) => ns);

      b. if refModule is not scope hoisted
        const { a, b } = await import('./ref-chunk').then(() => __webpack_require__(./refModule));
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
      // we can safely use __webpack_require__ to fetch module
      source.replace(
        import_dep.range.start,
        import_dep.range.end,
        &format!(
          "{import_promise}{}",
          then_expr(code_generatable_context, dep_id, request)
        ),
        None,
      );
      return;
    };

    let is_ref_module_concatenated =
      concatenation_scope.is_module_concatenated(&ref_module.identifier());

    if !is_ref_module_concatenated {
      // if target is not in a concatenation scope, then all its children are not scope hoisted as well
      // we can safely use __webpack_require__ to fetch module
      source.replace(
        import_dep.range.start,
        import_dep.range.end,
        &format!(
          "{import_promise}{}",
          then_expr(code_generatable_context, dep_id, request)
        ),
        None,
      );

      return;
    }

    // For empty facade chunks (0 modules, only re-exports) or single-module chunks,
    // the chunk's exports exactly match the module's exports
    // (ensured by link_entry_module_exports with strict_exports).
    // No .then() remapping needed — import() directly yields the correct namespace.
    if !already_in_chunk {
      let chunk_modules = code_generatable_context
        .compilation
        .build_chunk_graph_artifact
        .chunk_graph
        .get_chunk_modules_identifier(&ref_chunk_ukey);
      if chunk_modules.len() <= 1 {
        source.replace(
          import_dep.range.start,
          import_dep.range.end,
          &import_promise,
          None,
        );
        return;
      }
    }

    source.replace(
      import_dep.range.start,
      import_dep.range.end,
      &import_promise,
      None,
    );
  }
}
