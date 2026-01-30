use bitflags::bitflags;
use rspack_cacheable::{
  cacheable, cacheable_dyn,
  with::{AsOption, AsPreset},
};
use rspack_core::{
  AffectType, AsContextDependency, AsModuleDependency, Dependency, DependencyCategory,
  DependencyCodeGeneration, DependencyId, DependencyRange, DependencyTemplate,
  DependencyTemplateType, DependencyType, ExportsArgument, ModuleArgument,
  ModuleCodegenRuntimeTemplate, RuntimeGlobals, TemplateContext, TemplateReplaceSource,
};
use rspack_util::{atom::Atom, json_stringify};

use super::local_module::LocalModule;

bitflags! {
  #[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
  struct Branch: u8 {
    const L = 1 << 0;
    const A = 1 << 1;
    const O = 1 << 2;
    const F = 1 << 3;
  }
}

impl Branch {
  pub fn get_definition(self, local_module_var: &Option<String>) -> String {
    let name = match local_module_var {
      Some(name) => name,
      None => "XXX",
    };
    match self {
      f if f == Branch::F => "var __rspack_amd_exports;".to_string(),
      o if o == Branch::O => String::new(),
      o_f if o_f == (Branch::O | Branch::F) => {
        "var __rspack_amd_factory, __rspack_amd_exports;".to_string()
      }
      a_f if a_f == (Branch::A | Branch::F) => {
        "var __rspack_amd_deps, __rspack_amd_exports;".to_string()
      }
      a_o if a_o == (Branch::A | Branch::O) => String::new(),
      a_o_f if a_o_f == (Branch::A | Branch::O | Branch::F) => {
        "var __rspack_amd_factory, __rspack_amd_deps, __rspack_amd_exports;".to_string()
      }
      l_f if l_f == (Branch::L | Branch::F) => {
        format!("var {name}, {name}module;")
      }
      l_o if l_o == (Branch::L | Branch::O) => {
        format!("var {name};")
      }
      l_o_f if l_o_f == (Branch::L | Branch::O | Branch::F) => {
        format!("var {name}, {name}factory, {name}module;")
      }
      l_a_f if l_a_f == (Branch::L | Branch::A | Branch::F) => {
        format!("var __rspack_amd_deps, {name}, {name}exports;")
      }
      l_a_o if l_a_o == (Branch::L | Branch::A | Branch::O) => {
        format!("var {name};")
      }
      l_a_o_f if l_a_o_f == (Branch::L | Branch::A | Branch::O | Branch::F) => {
        format!("var {name}array, {name}factory, {name}exports, {name};")
      }
      _ => String::new(),
    }
  }

  pub fn get_content(
    self,
    local_module_var: &Option<String>,
    named_module: &Option<Atom>,
    runtime_template: &mut ModuleCodegenRuntimeTemplate,
  ) -> String {
    let local_module_var = match local_module_var {
      Some(name) => name,
      None => "XXX",
    };
    let named_module = match named_module {
      Some(name) => name,
      None => "YYY",
    };
    match self {
      f if f == Branch::F => {
        format!(
          "!(__rspack_amd_exports = (#).call({exports}, {require}, {exports}, {module}),
		__rspack_amd_exports !== undefined && ({module}.exports = __rspack_amd_exports))",
          require = runtime_template.render_runtime_globals(&RuntimeGlobals::REQUIRE),
          exports = runtime_template.render_exports_argument(ExportsArgument::Exports),
          module = runtime_template.render_module_argument(ModuleArgument::Module),
        )
      }
      o if o == Branch::O => format!("!({}.exports = #)", runtime_template.render_module_argument(ModuleArgument::Module)),
      o_f if o_f == (Branch::O | Branch::F) => {
        format!(
          "!(__rspack_amd_factory = (#),
		__rspack_amd_exports = (typeof __rspack_amd_factory === 'function' ?
		(__rspack_amd_factory.call({exports}, {require}, {exports}, {module})) :
		__rspack_amd_factory),
		__rspack_amd_exports !== undefined && ({module}.exports = __rspack_amd_exports))",
          require = runtime_template.render_runtime_globals(&RuntimeGlobals::REQUIRE),
          exports = runtime_template.render_exports_argument(ExportsArgument::Exports),
          module = runtime_template.render_module_argument(ModuleArgument::Module),
        )
      }
      a_f if a_f == (Branch::A | Branch::F) => format!("!(__rspack_amd_deps = #, __rspack_amd_exports = (#).apply({exports}, __rspack_amd_deps),
		__rspack_amd_exports !== undefined && ({module}.exports = __rspack_amd_exports))", 
        exports = runtime_template.render_exports_argument(ExportsArgument::Exports),
        module = runtime_template.render_module_argument(ModuleArgument::Module),
      ),
      a_o if a_o == (Branch::A | Branch::O) => format!("!(#, {}.exports = #)", runtime_template.render_module_argument(ModuleArgument::Module)),
      a_o_f if a_o_f == (Branch::A | Branch::O | Branch::F) => {
        format!("!(__rspack_amd_deps = #, __rspack_amd_factory = (#),
		__rspack_amd_exports = (typeof __rspack_amd_factory === 'function' ?
		(__rspack_amd_factory.apply({exports}, __rspack_amd_deps)) : __rspack_amd_factory),
		__rspack_amd_exports !== undefined && ({module}.exports = __rspack_amd_exports))",
          exports = runtime_template.render_exports_argument(ExportsArgument::Exports),
          module = runtime_template.render_module_argument(ModuleArgument::Module),
        )
      }
      l_f if l_f == (Branch::L | Branch::F) => {
        format!(
          "!({var_name}module = {{ id: {module_id}, exports: {{}}, loaded: false }}, {var_name} = (#).call({var_name}module.exports, {require}, {var_name}module.exports, {var_name}module), {var_name}module.loaded = true, {var_name} === undefined && ({var_name} = {var_name}module.exports))",
          var_name = local_module_var,
          module_id = json_stringify(named_module),
          require = runtime_template.render_runtime_globals(&RuntimeGlobals::REQUIRE),
        )
      }
      l_o if l_o == (Branch::L | Branch::O) => format!("!({local_module_var} = #)"),
      l_o_f if l_o_f == (Branch::L | Branch::O | Branch::F) => {
        format!(
          "!({var_name}factory = (#), (typeof {var_name}factory === 'function' ? (({var_name}module = {{ id: {module_id}, exports: {{}}, loaded: false }}), ({var_name} = {var_name}factory.call({var_name}module.exports, {require}, {var_name}module.exports, {var_name}module)), ({var_name}module.loaded = true), {var_name} === undefined && ({var_name} = {var_name}module.exports)) : {var_name} = {var_name}factory))",
          var_name = local_module_var,
          module_id = json_stringify(named_module),
          require = runtime_template.render_runtime_globals(&RuntimeGlobals::REQUIRE),
        )
      }
      l_a_f if l_a_f == (Branch::L | Branch::A | Branch::F) => format!("!(__rspack_amd_deps = #, {local_module_var} = (#).apply({local_module_var}exports = {{}}, __rspack_amd_deps), {local_module_var} === undefined && ({local_module_var} = {local_module_var}exports))"),
      l_a_o if l_a_o == (Branch::L | Branch::A | Branch::O) => format!("!(#, {local_module_var} = #)"),
      l_a_o_f if l_a_o_f == (Branch::L | Branch::A | Branch::O | Branch::F) => format!(
        "!({local_module_var}array = #, {local_module_var}factory = (#),
		(typeof {local_module_var}factory === 'function' ?
			(({local_module_var} = {local_module_var}factory.apply({local_module_var}exports = {{}}, {local_module_var}array)), {local_module_var} === undefined && ({local_module_var} = {local_module_var}exports)) :
			({local_module_var} = {local_module_var}factory)
		))",
      ),
      _ => String::new(),
    }
  }
}

#[cacheable]
#[derive(Debug, Clone)]
pub struct AMDDefineDependency {
  id: DependencyId,
  range: DependencyRange,
  array_range: Option<DependencyRange>,
  function_range: Option<DependencyRange>,
  object_range: Option<DependencyRange>,
  #[cacheable(with=AsOption<AsPreset>)]
  named_module: Option<Atom>,
  local_module: Option<LocalModule>,
}

impl AMDDefineDependency {
  pub fn new(
    range: DependencyRange,
    array_range: Option<DependencyRange>,
    function_range: Option<DependencyRange>,
    object_range: Option<DependencyRange>,
    named_module: Option<Atom>,
  ) -> Self {
    Self {
      id: DependencyId::new(),
      range,
      array_range,
      function_range,
      object_range,
      named_module,
      local_module: None,
    }
  }

  pub fn set_local_module(&mut self, local_module: LocalModule) {
    self.local_module = Some(local_module);
  }
}

#[cacheable_dyn]
impl Dependency for AMDDefineDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn range(&self) -> Option<DependencyRange> {
    Some(self.range)
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Amd
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::AmdDefine
  }

  fn could_affect_referencing_module(&self) -> AffectType {
    AffectType::False
  }
}

impl AMDDefineDependency {
  fn local_module_var(&self) -> Option<String> {
    self.local_module.as_ref().and_then(|m| {
      if m.is_used() {
        Some(m.variable_name())
      } else {
        None
      }
    })
  }

  fn branch(&self) -> Branch {
    let mut ret = Branch::empty();
    if self.local_module.as_ref().is_some_and(|m| m.is_used()) {
      ret |= Branch::L;
    }
    if self.array_range.is_some() {
      ret |= Branch::A;
    }
    if self.object_range.is_some() {
      ret |= Branch::O;
    }
    if self.function_range.is_some() {
      ret |= Branch::F;
    }
    ret
  }
}

impl AsModuleDependency for AMDDefineDependency {}

impl AsContextDependency for AMDDefineDependency {}

#[cacheable_dyn]
impl DependencyCodeGeneration for AMDDefineDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(AMDDefineDependencyTemplate::template_type())
  }
}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct AMDDefineDependencyTemplate;

impl AMDDefineDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Dependency(DependencyType::AmdDefine)
  }
}

impl DependencyTemplate for AMDDefineDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<AMDDefineDependency>()
      .expect("AMDDefineDependencyTemplate should only be used for AMDDefineDependency");

    let branch = dep.branch();

    let local_module_var = dep.local_module_var();

    let text = branch.get_content(
      &local_module_var,
      &dep.named_module,
      code_generatable_context.runtime_template,
    );
    let definition = branch.get_definition(&local_module_var);

    let mut texts = text.split('#');

    if !definition.is_empty() {
      source.insert(0, &definition, None);
    }

    let mut current = dep.range.start;
    if let Some(array_range) = &dep.array_range {
      source.replace(current, array_range.start, texts.next().unwrap_or(""), None);
      current = array_range.end;
    }

    if let Some(object_range) = &dep.object_range {
      source.replace(
        current,
        object_range.start,
        texts.next().unwrap_or(""),
        None,
      );
      current = object_range.end;
    } else if let Some(function_range) = &dep.function_range {
      source.replace(
        current,
        function_range.start,
        texts.next().unwrap_or(""),
        None,
      );
      current = function_range.end;
    }

    source.replace(current, dep.range.end, texts.next().unwrap_or(""), None);

    if texts.next().is_some() {
      panic!("Implementation error");
    }
  }
}
