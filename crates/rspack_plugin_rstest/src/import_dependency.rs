use rspack_cacheable::cacheable;
use rspack_core::{
  AsyncDependenciesBlockIdentifier, ChunkGraph, Compilation, Dependency, DependencyCodeGeneration,
  DependencyId, DependencyTemplate, DependencyTemplateType, DependencyType, ExportsType,
  FakeNamespaceObjectMode, ModuleCodeTemplate, ModuleDependency, ModuleGraph, RuntimeGlobals,
  TemplateContext, TemplateReplaceSource, get_exports_type,
};
use rspack_plugin_javascript::dependency::ImportDependency;
use rspack_util::json_stringify_str;

#[cacheable]
#[derive(Debug, Default)]
pub struct ImportDependencyTemplate;

impl ImportDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Dependency(DependencyType::DynamicImport)
  }
}

impl DependencyTemplate for ImportDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<ImportDependency>()
      .expect("ImportDependencyTemplate can only be applied to ImportDependency");
    let range = dep.range().expect("ImportDependency should have range");
    let module_graph = code_generatable_context.compilation.get_module_graph();
    let block = module_graph.get_parent_block(dep.id());
    let attributes = &dep.get_attributes();
    let is_import_actual = if let Some(attrs) = attributes {
      // loop attrs and check is there a key `rstest` is `importActual`
      if let Some(actual) = attrs.get("rstest") {
        actual == "importActual"
      } else {
        false
      }
    } else {
      false
    };

    source.replace(
      range.start,
      range.end,
      module_namespace_promise_rstest(
        code_generatable_context,
        dep.id(),
        block,
        dep.request(),
        dep.dependency_type().as_str(),
        false,
        is_import_actual,
      ),
      None,
    );
  }
}

pub fn module_id_rstest(
  compilation: &Compilation,
  runtime_template: &mut ModuleCodeTemplate,
  id: &DependencyId,
  request: &str,
  weak: bool,
) -> String {
  if let Some(module_identifier) = compilation
    .get_module_graph()
    .module_identifier_by_dependency_id(id)
    && let Some(module_id) =
      ChunkGraph::get_module_id(&compilation.module_ids_artifact, *module_identifier)
  {
    runtime_template.module_id_expr(request, module_id)
  } else if weak {
    "null /* weak dependency, without id */".to_string()
  } else {
    // missing_module(request)
    // NOTE: Rstest allow missing module, so we return the request as a string
    format!("\"{request}\"")
  }
}

// To support use `__webpack_require__.import_actual` for `importActual`.
fn module_namespace_promise_rstest(
  code_generatable_context: &mut TemplateContext,
  dep_id: &DependencyId,
  block: Option<&AsyncDependenciesBlockIdentifier>,
  request: &str,
  message: &str,
  weak: bool,
  is_import_actual: bool,
) -> String {
  let TemplateContext {
    runtime_template,
    compilation,
    module,
    ..
  } = code_generatable_context;
  if compilation
    .get_module_graph()
    .module_identifier_by_dependency_id(dep_id)
    .is_none()
  {
    return format!(
      "{}(\"{request}\")",
      runtime_template.render_runtime_globals(&RuntimeGlobals::REQUIRE)
    );
  };

  let promise = runtime_template.block_promise(block, compilation, message);
  let exports_type = get_exports_type(
    compilation.get_module_graph(),
    &compilation.module_graph_cache_artifact,
    &compilation.exports_info_artifact,
    dep_id,
    &module.identifier(),
  );

  let module_id_expr = module_id_rstest(compilation, runtime_template, dep_id, request, weak);

  let final_require = if is_import_actual {
    format!(
      "{}.rstest_import_actual",
      runtime_template.render_runtime_globals(&RuntimeGlobals::REQUIRE),
    )
  } else {
    runtime_template.render_runtime_globals(&RuntimeGlobals::REQUIRE)
  };

  // Only an externalized specifier exhibits the two-id split: rspack mints a
  // distinct id for the dynamic import (`external import "X"`) than the one the
  // hoisted `rs.mock` patches (`external module "X"`). Internal modules resolve
  // both the static/mock dependency and the dynamic import to the SAME id, so a
  // hoisted `rs.mock` already covers their dynamic import — leave that codegen
  // byte-identical to upstream. importActual keeps its dedicated
  // `rstest_import_actual` path regardless, so `&&` short-circuits the
  // module-graph lookup for it.
  let use_dynamic_shim = !is_import_actual && {
    compilation
      .get_module_graph()
      .get_module_by_dependency_id(dep_id)
      .is_some_and(|m| m.as_external_module().is_some())
  };

  let header = if weak {
    Some(format!(
      "if(!{}[{module_id_expr}]) {{\n {} \n}}",
      runtime_template.render_runtime_globals(&RuntimeGlobals::MODULE_FACTORIES),
      runtime_template.weak_error(request)
    ))
  } else {
    None
  };
  let mut fake_type = FakeNamespaceObjectMode::PROMISE_LIKE;
  let mut appending;
  match exports_type {
    ExportsType::Namespace => {
      if let Some(header) = header {
        appending = format!(
          ".then(function() {{ {header}\nreturn {}}})",
          runtime_template.module_raw(compilation, dep_id, request, weak)
        )
      } else if use_dynamic_shim {
        // External dynamic `import(request)`: route through `rstest_dynamic_require`
        // with the clean request literal so the hoisted `rs.mock` (installed under
        // the different static id) is found by request. `final_require` is the
        // REQUIRE global here (use_dynamic_shim implies !is_import_actual).
        // See packages/core/src/core/plugins/mockRuntimeCode.js.
        //
        // TODO(compat): the `rstest_dynamic_require ? … : <plain require>` guard is
        // a fallback for an OLDER @rstest/core runtime that predates the helper
        // (`undefined` → degrade to the plain `__webpack_require__(id)` form instead
        // of throwing `undefined.bind(...)`). Drop the guard, leaving the
        // unconditional `.bind` shim, once the minimum supported @rstest/core always
        // ships `rstest_dynamic_require`.
        appending = format!(
          ".then({final_require}.rstest_dynamic_require ? {final_require}.rstest_dynamic_require.bind({final_require}.rstest_dynamic_require, {module_id_expr}, {}) : {final_require}.bind({final_require}, {module_id_expr}))",
          json_stringify_str(request)
        );
      } else {
        appending = format!(".then({final_require}.bind({final_require}, {module_id_expr}))");
      }
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
        if let Some(header) = header {
          appending = format!(
            ".then(function() {{\n {header}\nreturn {}\n}})",
            runtime_template.module_raw(compilation, dep_id, request, weak)
          )
        } else {
          appending = format!(".then({final_require}.bind({final_require}, {module_id_expr}))");
        }
        appending.push_str(
          format!(
            ".then(function(m){{\n return {}(m, {fake_type}) \n}})",
            runtime_template.render_runtime_globals(&RuntimeGlobals::CREATE_FAKE_NAMESPACE_OBJECT)
          )
          .as_str(),
        );
      } else {
        fake_type |= FakeNamespaceObjectMode::MODULE_ID;
        if let Some(header) = header {
          let expr = format!(
            "{}({module_id_expr}, {fake_type}))",
            runtime_template.render_runtime_globals(&RuntimeGlobals::CREATE_FAKE_NAMESPACE_OBJECT)
          );
          appending = format!(".then(function() {{\n {header} return {expr};\n}})");
        } else {
          appending = format!(
            ".then({}.bind({}, {module_id_expr}, {fake_type}))",
            runtime_template.render_runtime_globals(&RuntimeGlobals::CREATE_FAKE_NAMESPACE_OBJECT),
            final_require
          );
        }
      }
    }
  }

  format!("{promise}{appending}")
}
