use rspack_cacheable::cacheable;
use rspack_core::{
  block_promise, get_exports_type, module_id_expr, module_raw, weak_error,
  AsyncDependenciesBlockIdentifier, ChunkGraph, Compilation, Dependency, DependencyCodeGeneration,
  DependencyId, DependencyTemplate, DependencyTemplateType, DependencyType, ExportsType,
  FakeNamespaceObjectMode, ModuleDependency, ModuleGraph, RuntimeGlobals, TemplateContext,
  TemplateReplaceSource,
};
use rspack_plugin_javascript::dependency::ImportDependency;

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
      )
      .as_str(),
      None,
    );
  }
}

pub fn module_id_rstest(
  compilation: &Compilation,
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
    module_id_expr(&compilation.options, request, module_id)
  } else if weak {
    "null /* weak dependency, without id */".to_string()
  } else {
    // missing_module(request)
    // NOTE: Rstest allow missing module, so we return the request as a string
    format!("\"{request}\"")
  }
}

const WEBPACK_REQUIRE_IMPORT_ACTUAL: &str = "__webpack_require__.import_actual";

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
    return format!("__webpack_require__(\"{request}\")");
  };

  let promise = block_promise(block, runtime_requirements, compilation, message);
  let exports_type = get_exports_type(
    &compilation.get_module_graph(),
    &compilation.module_graph_cache_artifact,
    dep_id,
    &module.identifier(),
  );

  let module_id_expr = module_id_rstest(compilation, dep_id, request, weak);

  let final_require = if is_import_actual {
    WEBPACK_REQUIRE_IMPORT_ACTUAL
  } else {
    "__webpack_require__"
  };

  let header = if weak {
    runtime_requirements.insert(RuntimeGlobals::MODULE_FACTORIES);
    Some(format!(
      "if(!{}[{module_id_expr}]) {{\n {} \n}}",
      RuntimeGlobals::MODULE_FACTORIES,
      weak_error(request)
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
          module_raw(compilation, runtime_requirements, dep_id, request, weak)
        )
      } else {
        runtime_requirements.insert(RuntimeGlobals::REQUIRE);
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
      runtime_requirements.insert(RuntimeGlobals::CREATE_FAKE_NAMESPACE_OBJECT);
      if ModuleGraph::is_async(
        compilation,
        compilation
          .get_module_graph()
          .module_identifier_by_dependency_id(dep_id)
          .expect("should have module"),
      ) {
        if let Some(header) = header {
          appending = format!(
            ".then(function() {{\n {header}\nreturn {}\n}})",
            module_raw(compilation, runtime_requirements, dep_id, request, weak)
          )
        } else {
          runtime_requirements.insert(RuntimeGlobals::REQUIRE);
          appending = format!(".then({final_require}.bind({final_require}, {module_id_expr}))");
        }
        appending.push_str(
          format!(
            ".then(function(m){{\n return {}(m, {fake_type}) \n}})",
            RuntimeGlobals::CREATE_FAKE_NAMESPACE_OBJECT
          )
          .as_str(),
        );
      } else {
        fake_type |= FakeNamespaceObjectMode::MODULE_ID;
        if let Some(header) = header {
          let expr = format!(
            "{}({module_id_expr}, {fake_type}))",
            RuntimeGlobals::CREATE_FAKE_NAMESPACE_OBJECT
          );
          appending = format!(".then(function() {{\n {header} return {expr};\n}})");
        } else {
          runtime_requirements.insert(RuntimeGlobals::REQUIRE);
          appending = format!(
            ".then({}.bind({}, {module_id_expr}, {fake_type}))",
            RuntimeGlobals::CREATE_FAKE_NAMESPACE_OBJECT,
            final_require
          );
        }
      }
    }
  }

  format!("{promise}{appending}")
}
