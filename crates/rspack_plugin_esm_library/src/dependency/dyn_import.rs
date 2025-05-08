use std::borrow::Cow;

use rspack_core::{
  AsModuleDependency, ChunkGraph, ChunkUkey, Compilation, DependencyTemplate, ModuleIdentifier,
  RuntimeGlobals, UsageState, missing_module_promise,
};
use rspack_plugin_javascript::dependency::ImportDependency;

use crate::EsmLibraryPlugin;

pub static NAMESPACE_SYMBOL: &str = "mod";

#[derive(Debug, Default)]
pub struct DynamicImportDependencyTemplate;

impl DependencyTemplate for DynamicImportDependencyTemplate {
  fn render(
    &self,
    dep: &dyn rspack_core::DependencyCodeGeneration,
    source: &mut rspack_core::TemplateReplaceSource,
    code_generatable_context: &mut rspack_core::TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<ImportDependency>()
      .expect("ImportDependencyTemplate can only be applied to ImportDependency");
    let dep_id = dep
      .as_module_dependency()
      .expect("should be module dep")
      .id();
    let module_graph = code_generatable_context.compilation.get_module_graph();

    let Some(ref_module) = module_graph.get_module_by_dependency_id(dep_id) else {
      let missing_promise = missing_module_promise(dep.request.as_str());
      source.replace(dep.range.start, dep.range.end, &missing_promise, None);
      return;
    };

    let ref_chunk = EsmLibraryPlugin::get_module_chunk(
      ref_module.identifier(),
      code_generatable_context.compilation,
    );

    let orig_chunk = EsmLibraryPlugin::get_module_chunk(
      *module_graph
        .get_parent_module(dep_id)
        .expect("should have parent module for import dep"),
      code_generatable_context.compilation,
    );

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
    let Some(concatenation_scope) = &mut code_generatable_context.concatenation_scope else {
      // if we are not in a concatenation scope, then all its children are not scope hoisted as well
      // we can safely use __webpack_require__ to fetch module
      source.replace(
        dep.range.start,
        dep.range.end,
        &fetch_raw_module(
          ref_module.identifier(),
          ref_chunk,
          orig_chunk,
          code_generatable_context.compilation,
        ),
        None,
      );
      code_generatable_context
        .runtime_requirements
        .insert(RuntimeGlobals::REQUIRE);
      return;
    };

    let is_ref_module_concatenated =
      concatenation_scope.is_module_concatenated(&ref_module.identifier());

    if !is_ref_module_concatenated {
      // if we are not in a concatenation scope, then all its children are not scope hoisted as well
      // we can safely use __webpack_require__ to fetch module
      source.replace(
        dep.range.start,
        dep.range.end,
        &fetch_raw_module(
          ref_module.identifier(),
          ref_chunk,
          orig_chunk,
          code_generatable_context.compilation,
        ),
        None,
      );

      return;
    }

    let already_in_chunk = ref_chunk == orig_chunk;
    let import_promise = if already_in_chunk {
      Cow::Borrowed("Promise.resolve()")
    } else {
      Cow::Owned(format!(
        "import(\"__RSPACK_ESM_CHUNK_{}\")",
        ref_chunk.as_u32()
      ))
    };

    // importer and importee are both scope hoisted
    let render_exports = if let Some(ref_exports) = &dep.referenced_exports {
      // we only extract the named exports
      // const { a, b } = await import('./refModule');
      // const { a, b } = await import('./refChunk').then(mod => ({ a: __WEBPACK_MODULE_DYNAMIC_REFERENCE__0_a, b: __WEBPACK_MODULE_DYNAMIC_REFERENCE__0_b }));
      ref_exports
        .iter()
        .map(|ref_export| {
          format!(
            "{}: {}",
            ref_export,
            concatenation_scope.create_dynamic_module_reference(
              &ref_module.identifier(),
              already_in_chunk,
              ref_export
            )
          )
        })
        .collect::<Vec<_>>()
        .join(",")
    } else {
      let ref_exports_info = module_graph.get_prefetched_exports_info(
        &ref_module.identifier(),
        rspack_core::PrefetchExportsInfoMode::Default,
      );
      let all_exports = ref_exports_info.get_relevant_exports(None);
      all_exports
        .iter()
        .filter(|export| !matches!(export.get_used(None), UsageState::Unused))
        .filter_map(|export| export.name())
        .map(|ref_export| {
          format!(
            "{}: {}",
            ref_export,
            concatenation_scope.create_dynamic_module_reference(
              &ref_module.identifier(),
              already_in_chunk,
              ref_export
            )
          )
        })
        .collect::<Vec<_>>()
        .join(",")
    };

    source.replace(
      dep.range.start,
      dep.range.end,
      &format!(
        "{}.then(({}) => ({{ {} }}))",
        import_promise,
        if already_in_chunk {
          ""
        } else {
          NAMESPACE_SYMBOL
        },
        render_exports
      ),
      None,
    );
  }
}

fn fetch_raw_module(
  ref_module: ModuleIdentifier,
  ref_chunk: ChunkUkey,
  orig_chunk: ChunkUkey,
  compilation: &Compilation,
) -> String {
  format!(
    "{}.then(() => {}({}))",
    if ref_chunk == orig_chunk {
      Cow::Borrowed("Promise.resolve()")
    } else {
      Cow::Owned(format!(
        "import(\"__RSPACK_ESM_CHUNK_{}\")",
        ref_chunk.as_u32()
      ))
    },
    RuntimeGlobals::REQUIRE,
    serde_json::to_string(
      ChunkGraph::get_module_id(&compilation.module_ids_artifact, ref_module)
        .expect("should have id")
    )
    .expect("should serde to string")
  )
}
