use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AsContextDependency, AsyncDependenciesBlockIdentifier, ChunkGraph, Compilation, Dependency,
  DependencyCategory, DependencyCodeGeneration, DependencyCondition, DependencyId, DependencyRange,
  DependencyTemplate, DependencyTemplateType, DependencyType, ExportsInfoArtifact, ExportsType,
  FakeNamespaceObjectMode, ImportAttributes, ImportPhase, ModuleCodeTemplate, ModuleDependency,
  ModuleGraph, ModuleGraphCacheArtifact, ReferencedExport, RuntimeGlobals, TemplateContext,
  TemplateReplaceSource, get_exports_type,
};
use rspack_plugin_javascript::dependency::ImportDependency;
use rspack_util::json_stringify_str;

#[cacheable]
#[derive(Debug)]
pub struct RstestImportDependency {
  pub inner: ImportDependency,
  pub rstest_request: String,
}

impl RstestImportDependency {
  pub fn new(inner: ImportDependency, rstest_request: String) -> Self {
    Self {
      inner,
      rstest_request,
    }
  }
}

#[cacheable_dyn]
impl Dependency for RstestImportDependency {
  fn id(&self) -> &DependencyId {
    self.inner.id()
  }
  fn resource_identifier(&self) -> Option<&str> {
    self.inner.resource_identifier()
  }
  fn category(&self) -> &DependencyCategory {
    self.inner.category()
  }
  fn dependency_type(&self) -> &DependencyType {
    self.inner.dependency_type()
  }
  fn get_attributes(&self) -> Option<&ImportAttributes> {
    self.inner.get_attributes()
  }
  fn get_phase(&self) -> ImportPhase {
    self.inner.get_phase()
  }
  fn range(&self) -> Option<DependencyRange> {
    self.inner.range()
  }
  fn get_referenced_exports(
    &self,
    module_graph: &rspack_core::ModuleGraph,
    module_graph_cache: &ModuleGraphCacheArtifact,
    exports_info_artifact: &ExportsInfoArtifact,
    runtime: Option<&rspack_core::RuntimeSpec>,
  ) -> Vec<ReferencedExport> {
    self.inner.get_referenced_exports(
      module_graph,
      module_graph_cache,
      exports_info_artifact,
      runtime,
    )
  }
  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    self.inner.could_affect_referencing_module()
  }
}

#[cacheable_dyn]
impl ModuleDependency for RstestImportDependency {
  fn request(&self) -> &str {
    self.inner.request()
  }
  fn user_request(&self) -> &str {
    self.inner.user_request()
  }
  fn get_optional(&self) -> bool {
    self.inner.get_optional()
  }
  fn get_condition(&self) -> Option<DependencyCondition> {
    self.inner.get_condition()
  }
}

impl AsContextDependency for RstestImportDependency {}

#[cacheable_dyn]
impl DependencyCodeGeneration for RstestImportDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(ImportDependencyTemplate::template_type())
  }
}

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
    let (inner, rstest_request) =
      if let Some(dep) = dep.as_any().downcast_ref::<RstestImportDependency>() {
        (&dep.inner, Some(dep.rstest_request.as_str()))
      } else if let Some(dep) = dep.as_any().downcast_ref::<ImportDependency>() {
        (dep, None)
      } else {
        panic!("ImportDependencyTemplate can only be applied to ImportDependency");
      };
    let range = inner.range().expect("ImportDependency should have range");
    let module_graph = code_generatable_context.compilation.get_module_graph();
    let block = module_graph.get_parent_block(inner.id());
    let attributes = &inner.get_attributes();
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
    let is_import_mock = attributes
      .and_then(|attrs| attrs.get("rstest"))
      .is_some_and(|value| value == "importMock");
    let mock_request = rstest_request.unwrap_or(inner.request());

    source.replace(
      range.start,
      range.end,
      module_namespace_promise_rstest(
        code_generatable_context,
        inner.id(),
        block,
        inner.request(),
        inner.dependency_type().as_str(),
        false,
        is_import_actual,
        is_import_mock,
        mock_request,
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

// To support use `__rspack_require.import_actual` for `importActual`.
#[allow(clippy::too_many_arguments)]
fn module_namespace_promise_rstest(
  code_generatable_context: &mut TemplateContext,
  dep_id: &DependencyId,
  block: Option<&AsyncDependenciesBlockIdentifier>,
  request: &str,
  message: &str,
  weak: bool,
  is_import_actual: bool,
  is_import_mock: bool,
  mock_request: &str,
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
    let require = runtime_template.render_runtime_globals(&RuntimeGlobals::REQUIRE);
    return if is_import_mock {
      format!(
        "({require}.rstest_import_mock ? {require}.rstest_import_mock(undefined, {}, null) : {require}(\"{request}\"))",
        json_stringify_str(mock_request)
      )
    } else {
      format!("{require}(\"{request}\")")
    };
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
  } else if is_import_mock {
    format!(
      "{}.rstest_import_mock",
      runtime_template.render_runtime_globals(&RuntimeGlobals::REQUIRE),
    )
  } else {
    runtime_template.render_runtime_globals(&RuntimeGlobals::REQUIRE)
  };

  // Externalized specifiers exhibit a two-id split: rspack mints a distinct id
  // for the dynamic import (`external import "X"`) vs the one `rs.mock` patches
  // (`external module "X"`). Internal modules share one id, so a hoisted
  // `rs.mock` already covers their dynamic import — leave that byte-identical.
  // Only the Namespace arm below needs the shim: a split always resolves to
  // `import`/`module` (= Namespace) for rstest's plain-string externals (a
  // `has_rest()` property-access external would resolve to Dynamic and miss it,
  // but rstest never emits those).
  let use_dynamic_shim = !is_import_actual && !is_import_mock && {
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
      if is_import_mock {
        let require = runtime_template.render_runtime_globals(&RuntimeGlobals::REQUIRE);
        appending = format!(
          ".then({require}.rstest_import_mock ? {require}.rstest_import_mock.bind({require}.rstest_import_mock, {module_id_expr}, {}, null) : {require}.bind({require}, {module_id_expr}))",
          json_stringify_str(mock_request)
        );
      } else if let Some(header) = header {
        appending = format!(
          ".then(function() {{ {header}\nreturn {}}})",
          runtime_template.module_raw(compilation, dep_id, request, weak)
        )
      } else if use_dynamic_shim {
        // Route the external dynamic import through the request-keyed
        // `rstest_dynamic_require` (in the @rstest/core runtime, a separate repo:
        // .../plugins/mockRuntimeCode.js) so the hoisted `rs.mock` is found.
        //
        // TODO(compat): the `rstest_dynamic_require ? … : <plain require>` guard
        // falls back to plain require for an older @rstest/core lacking the helper;
        // drop it once the minimum @rstest/core always ships it.
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
        if is_import_mock {
          let require = runtime_template.render_runtime_globals(&RuntimeGlobals::REQUIRE);
          appending = format!(
            ".then({require}.rstest_import_mock ? {require}.rstest_import_mock.bind({require}.rstest_import_mock, {module_id_expr}, {}, null) : {require}.bind({require}, {module_id_expr}))",
            json_stringify_str(mock_request)
          );
        } else if let Some(header) = header {
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
        if is_import_mock {
          let require = runtime_template.render_runtime_globals(&RuntimeGlobals::REQUIRE);
          appending = format!(
            ".then({require}.rstest_import_mock ? {require}.rstest_import_mock.bind({require}.rstest_import_mock, {module_id_expr}, {}, null) : {require}.bind({require}, {module_id_expr})).then(function(m){{ return {}(m, {fake_type}) }})",
            json_stringify_str(mock_request),
            runtime_template.render_runtime_globals(&RuntimeGlobals::CREATE_FAKE_NAMESPACE_OBJECT),
          );
        } else if let Some(header) = header {
          fake_type |= FakeNamespaceObjectMode::MODULE_ID;
          let expr = format!(
            "{}({module_id_expr}, {fake_type}))",
            runtime_template.render_runtime_globals(&RuntimeGlobals::CREATE_FAKE_NAMESPACE_OBJECT)
          );
          appending = format!(".then(function() {{\n {header} return {expr};\n}})");
        } else {
          fake_type |= FakeNamespaceObjectMode::MODULE_ID;
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
