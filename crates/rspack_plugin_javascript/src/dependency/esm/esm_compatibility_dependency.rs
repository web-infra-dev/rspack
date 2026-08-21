use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  ChunkGraph, Compilation, DependencyCodeGeneration, DependencyTemplate, DependencyTemplateType,
  InitFragmentKey, InitFragmentStage, ModuleCodeTemplate, ModuleGraph, ModuleInitFragments,
  NormalInitFragment, RuntimeGlobals, TemplateContext, TemplateReplaceSource, UsageState,
};
use swc_atoms::Atom;

pub(super) fn add_async_module_boundary(
  init_fragments: &mut ModuleInitFragments,
  compilation: &Compilation,
  module: &dyn rspack_core::Module,
  runtime_template: &mut ModuleCodeTemplate,
  use_module_exports: bool,
) {
  let module_id = ChunkGraph::get_module_id(&compilation.module_ids_artifact, module.identifier())
    .map(ToString::to_string)
    .unwrap_or_default();
  let module_argument = runtime_template.render_module_argument(module.get_module_argument());
  let async_module_parameter = if use_module_exports {
    ", __rspack_async_module"
  } else {
    ""
  };
  let module_argument_assignment = if use_module_exports {
    format!("{module_argument} = __rspack_async_module;\n")
  } else {
    String::new()
  };
  let async_module_arguments = match (
    module.build_meta().has_top_level_await(),
    use_module_exports,
  ) {
    (false, false) => "",
    (true, false) => ", 1",
    (false, true) => ", 0, 1",
    (true, true) => ", 1, 1",
  };
  init_fragments.push(Box::new(NormalInitFragment::new(
    format!(
      "{}({}, async function (__rspack_load_async_deps, __rspack_async_done{async_module_parameter}) {{ try {{\n{module_argument_assignment}",
      runtime_template.render_runtime_globals(&RuntimeGlobals::ASYNC_MODULE),
      module_argument,
    ),
    InitFragmentStage::StageAsyncBoundary,
    0,
    InitFragmentKey::AsyncBoundary(module_id),
    Some(format!(
      "\n__rspack_async_done();\n}} catch(e) {{ __rspack_async_done(e); }} }}{async_module_arguments});",
    )),
  )));
}

// Mark module `__esModule`.
// Add `__rspack_require.r(__rspack_exports);`.
#[cacheable]
#[derive(Debug, Clone)]
pub struct ESMCompatibilityDependency;

#[cacheable_dyn]
impl DependencyCodeGeneration for ESMCompatibilityDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(ESMCompatibilityDependencyTemplate::template_type())
  }
}

#[cacheable]
#[derive(Debug, Default)]
pub struct ESMCompatibilityDependencyTemplate;

impl ESMCompatibilityDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Custom("ESMCompatibilityDependency")
  }
}

impl DependencyTemplate for ESMCompatibilityDependencyTemplate {
  fn render(
    &self,
    _dep: &dyn DependencyCodeGeneration,
    _source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let TemplateContext {
      init_fragments,
      compilation,
      module,
      runtime,
      concatenation_scope,
      runtime_template,
      ..
    } = code_generatable_context;
    if concatenation_scope.is_some() {
      return;
    }
    let module_graph = compilation.get_module_graph();
    let module = module_graph
      .module_by_identifier(&module.identifier())
      .expect("should have mgm");
    let name = Atom::from("__esModule");
    let exports_info = compilation
      .exports_info_artifact
      .get_exports_info_data(&module.identifier());
    let used = exports_info
      .get_read_only_export_info(&name)
      .get_used(*runtime);
    if !matches!(used, UsageState::Unused) {
      init_fragments.push(Box::new(NormalInitFragment::new(
        format!(
          "{}({});\n",
          runtime_template.render_runtime_globals(&RuntimeGlobals::MAKE_NAMESPACE_OBJECT),
          runtime_template.render_exports_argument(module.get_exports_argument()),
        ),
        InitFragmentStage::StageESMExports,
        0,
        InitFragmentKey::ESMCompatibility,
        None,
      )));
    }

    if ModuleGraph::is_async(&compilation.async_modules_artifact, &module.identifier()) {
      add_async_module_boundary(
        init_fragments,
        compilation,
        module.as_ref(),
        runtime_template,
        false,
      );
    }
  }
}
