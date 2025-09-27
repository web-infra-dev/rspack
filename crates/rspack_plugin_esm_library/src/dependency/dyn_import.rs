use std::borrow::Cow;

use rspack_core::{
  Dependency, DependencyId, DependencyTemplate, ExportsType, ExtendedReferencedExport,
  FakeNamespaceObjectMode, ModuleGraph, RuntimeGlobals, TemplateContext, UsageState,
  get_exports_type, missing_module_promise, module_id,
};
use rspack_plugin_javascript::dependency::ImportDependency;

use crate::EsmLibraryPlugin;

pub static NAMESPACE_SYMBOL: &str = "mod";

fn then_expr(
  code_generatable_context: &mut TemplateContext,
  dep_id: &DependencyId,
  request: &str,
) -> String {
  let TemplateContext {
    runtime_requirements,
    compilation,
    module,
    ..
  } = code_generatable_context;
  if compilation
    .get_module_graph()
    .module_identifier_by_dependency_id(dep_id)
    .is_none()
  {
    return missing_module_promise(request);
  };

  let exports_type = get_exports_type(
    &compilation.get_module_graph(),
    &compilation.module_graph_cache_artifact,
    dep_id,
    &module.identifier(),
  );
  let module_id_expr = module_id(compilation, dep_id, request, false);

  let mut fake_type = FakeNamespaceObjectMode::PROMISE_LIKE;
  let mut appending;

  match exports_type {
    ExportsType::Namespace => {
      runtime_requirements.insert(RuntimeGlobals::REQUIRE);
      appending = format!(
        ".then({}.bind({}, {module_id_expr}))",
        RuntimeGlobals::REQUIRE,
        RuntimeGlobals::REQUIRE
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
      runtime_requirements.insert(RuntimeGlobals::CREATE_FAKE_NAMESPACE_OBJECT);
      if ModuleGraph::is_async(
        compilation,
        compilation
          .get_module_graph()
          .module_identifier_by_dependency_id(dep_id)
          .expect("should have module"),
      ) {
        runtime_requirements.insert(RuntimeGlobals::REQUIRE);
        appending = format!(
          ".then({}.bind({}, {module_id_expr}))",
          RuntimeGlobals::REQUIRE,
          RuntimeGlobals::REQUIRE
        );
        appending.push_str(
          format!(
            ".then(function(m){{\n return {}(m, {fake_type}) \n}})",
            RuntimeGlobals::CREATE_FAKE_NAMESPACE_OBJECT
          )
          .as_str(),
        );
      } else {
        fake_type |= FakeNamespaceObjectMode::MODULE_ID;
        runtime_requirements.insert(RuntimeGlobals::REQUIRE);
        appending = format!(
          ".then({}.bind({}, {module_id_expr}, {fake_type}))",
          RuntimeGlobals::CREATE_FAKE_NAMESPACE_OBJECT,
          RuntimeGlobals::REQUIRE
        );
      }
    }
  }
  appending
}

#[derive(Debug, Default)]
pub struct DynamicImportDependencyTemplate;

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
      let missing_promise = missing_module_promise(request);
      source.replace(
        import_dep.range.start,
        import_dep.range.end,
        &missing_promise,
        None,
      );
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
    let already_in_chunk = ref_chunk == orig_chunk;
    let import_promise = if already_in_chunk {
      Cow::Borrowed("Promise.resolve()")
    } else {
      Cow::Owned(format!(
        "import(\"__RSPACK_ESM_CHUNK_{}\")",
        ref_chunk.as_u32()
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
      code_generatable_context
        .runtime_requirements
        .insert(RuntimeGlobals::REQUIRE);
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

    // importer and importee are both scope hoisted
    let ref_exports = dep.get_referenced_exports(
      &module_graph,
      &code_generatable_context
        .compilation
        .module_graph_cache_artifact,
      None,
    );

    let render_exports = if !ref_exports.is_empty()
      && !ref_exports.iter().any(|ref_exports| match ref_exports {
        ExtendedReferencedExport::Array(atoms) => atoms.is_empty(),
        ExtendedReferencedExport::Export(referenced_export) => referenced_export.name.is_empty(),
      }) {
      // we only extract the named exports
      // const { a, b } = await import('./refModule');
      // const { a, b } = await import('./refChunk').then(mod => ({ a: __WEBPACK_MODULE_DYNAMIC_REFERENCE__0_a, b: __WEBPACK_MODULE_DYNAMIC_REFERENCE__0_b }));
      let ref_exports = ref_exports
        .iter()
        .flat_map(|ref_exports| match ref_exports {
          ExtendedReferencedExport::Array(atoms) => atoms
            .iter()
            .map(|atom| {
              format!(
                "{atom}: {}",
                concatenation_scope.create_dynamic_module_reference(
                  &ref_module.identifier(),
                  already_in_chunk,
                  atom
                )
              )
            })
            .collect::<Vec<_>>(),
          ExtendedReferencedExport::Export(referenced_export) => referenced_export
            .name
            .iter()
            .map(|atom| {
              format!(
                "{atom}: {}",
                concatenation_scope.create_dynamic_module_reference(
                  &ref_module.identifier(),
                  already_in_chunk,
                  atom
                )
              )
            })
            .collect::<Vec<_>>(),
        })
        .collect::<Vec<_>>();
      ref_exports.join(",")
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
      import_dep.range.start,
      import_dep.range.end,
      &format!(
        "{}{}",
        import_promise,
        if render_exports.is_empty() {
          Cow::Borrowed("")
        } else {
          Cow::Owned(format!(
            ".then(({}) => ({{ {} }}))",
            if already_in_chunk {
              ""
            } else {
              NAMESPACE_SYMBOL
            },
            render_exports
          ))
        }
      ),
      None,
    );
  }
}
