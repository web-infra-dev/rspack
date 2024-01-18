use std::borrow::Cow;

use serde_json::json;
use swc_core::ecma::atoms::Atom;

use crate::{
  get_import_var, property_access, to_comment, to_normal_comment, AsyncDependenciesBlockId,
  Compilation, DependenciesBlock, DependencyId, ExportsType, FakeNamespaceObjectMode,
  InitFragmentExt, InitFragmentKey, InitFragmentStage, ModuleGraph, ModuleIdentifier,
  NormalInitFragment, RuntimeGlobals, TemplateContext,
};

#[allow(clippy::too_many_arguments)]
pub fn export_from_import(
  code_generatable_context: &mut TemplateContext,
  default_interop: bool,
  request: &str,
  import_var: &str,
  mut export_name: Vec<Atom>,
  id: &DependencyId,
  is_call: bool,
  call_context: bool,
) -> String {
  let TemplateContext {
    runtime_requirements,
    compilation,
    init_fragments,
    module,
    runtime,
    ..
  } = code_generatable_context;
  let Some(module_identifier) = compilation
    .module_graph
    .module_identifier_by_dependency_id(id)
    .copied()
  else {
    return missing_module(request);
  };
  let is_new_treeshaking = compilation.options.is_new_tree_shaking();

  let exports_type = get_exports_type(&compilation.module_graph, id, &module.identifier());

  if default_interop {
    if !export_name.is_empty()
      && let Some(first_export_name) = export_name.first()
      && first_export_name == "default"
    {
      match exports_type {
        ExportsType::Dynamic => {
          if is_call {
            return format!("{import_var}_default(){}", property_access(export_name, 1));
          } else {
            return format!(
              "({import_var}_default(){})",
              property_access(export_name, 1)
            );
          }
        }
        ExportsType::DefaultOnly | ExportsType::DefaultWithNamed => {
          export_name = export_name[1..].to_vec();
        }
        _ => {}
      }
    } else if !export_name.is_empty() {
      if matches!(exports_type, ExportsType::DefaultOnly) {
        return format!(
          "/* non-default import from non-esm module */undefined\n{}",
          property_access(export_name, 1)
        );
      } else if !matches!(exports_type, ExportsType::Namespace)
        && let Some(first_export_name) = export_name.first()
        && first_export_name == "__esModule"
      {
        return "/* __esModule */true".to_string();
      }
    } else if matches!(
      exports_type,
      ExportsType::DefaultOnly | ExportsType::DefaultWithNamed
    ) {
      runtime_requirements.insert(RuntimeGlobals::CREATE_FAKE_NAMESPACE_OBJECT);
      init_fragments.push(
        NormalInitFragment::new(
          format!("var {import_var}_namespace_cache;\n",),
          InitFragmentStage::StageHarmonyExports,
          -1,
          InitFragmentKey::unique(),
          None,
        )
        .boxed(),
      );
      return format!("/*#__PURE__*/ ({import_var}_namespace_cache || ({import_var}_namespace_cache = {}({import_var}{})))", RuntimeGlobals::CREATE_FAKE_NAMESPACE_OBJECT, if matches!(exports_type, ExportsType::DefaultOnly) { "" } else { ", 2" });
    }
  }

  if !export_name.is_empty() {
    let used_name = if is_new_treeshaking {
      let exports_info_id = compilation
        .module_graph
        .get_exports_info(&module_identifier)
        .id;
      let used = exports_info_id.get_used_name(
        &compilation.module_graph,
        *runtime,
        crate::UsedName::Vec(export_name.clone()),
      );
      if let Some(used) = used {
        let used = match used {
          crate::UsedName::Str(str) => vec![str],
          crate::UsedName::Vec(strs) => strs,
        };
        Cow::Owned(used)
      } else {
        // TODO: add some unused comments, part of runtime alignments
        return "".to_string();
      }
    } else {
      Cow::Borrowed(&export_name)
    };
    let comment = if *used_name != export_name {
      to_normal_comment(&property_access(&export_name, 0))
    } else {
      "".to_string()
    };
    let property = property_access(&*used_name, 0);
    if is_call && !call_context {
      format!("(0, {import_var}{comment}{property})")
    } else {
      format!("{import_var}{comment}{property}")
    }
  } else {
    import_var.to_string()
  }
}

pub fn get_exports_type(
  module_graph: &ModuleGraph,
  id: &DependencyId,
  parent_module: &ModuleIdentifier,
) -> ExportsType {
  let module = module_graph
    .module_identifier_by_dependency_id(id)
    .expect("should have module");
  let strict = module_graph
    .module_by_identifier(parent_module)
    .expect("should have mgm")
    .get_strict_harmony_module();
  module_graph
    .module_by_identifier(module)
    .expect("should have mgm")
    .get_exports_type(strict)
}

pub fn get_exports_type_with_strict(
  module_graph: &ModuleGraph,
  id: &DependencyId,
  strict: bool,
) -> ExportsType {
  let module = module_graph
    .module_identifier_by_dependency_id(id)
    .expect("should have module");
  module_graph
    .module_by_identifier(module)
    .expect("should have mgm")
    .get_exports_type(strict)
}

pub fn module_id_expr(request: &str, module_id: &str) -> String {
  format!(
    "{}{}",
    to_comment(request),
    serde_json::to_string(module_id).expect("should render module id")
  )
}

pub fn module_id(
  compilation: &Compilation,
  id: &DependencyId,
  request: &str,
  weak: bool,
) -> String {
  if let Some(module_identifier) = compilation
    .module_graph
    .module_identifier_by_dependency_id(id)
    && let Some(module_id) = compilation.chunk_graph.get_module_id(*module_identifier)
  {
    module_id_expr(request, module_id)
  } else if weak {
    "null /* weak dependency, without id */".to_string()
  } else {
    missing_module(request)
  }
}

pub fn import_statement(
  code_generatable_context: &mut TemplateContext,
  id: &DependencyId,
  request: &str,
  update: bool, // whether a new variable should be created or the existing one updated
) -> (String, String) {
  let TemplateContext {
    runtime_requirements,
    compilation,
    module,
    ..
  } = code_generatable_context;
  if compilation
    .module_graph
    .module_identifier_by_dependency_id(id)
    .is_none()
  {
    return (missing_module_statement(request), "".to_string());
  };

  let module_id_expr = module_id(compilation, id, request, false);

  runtime_requirements.insert(RuntimeGlobals::REQUIRE);

  let import_var = get_import_var(&compilation.module_graph, *id);

  let opt_declaration = if update { "" } else { "var " };

  let import_content = format!(
    "/* harmony import */{opt_declaration}{import_var} = {}({module_id_expr});\n",
    RuntimeGlobals::REQUIRE
  );

  let exports_type = get_exports_type(&compilation.module_graph, id, &module.identifier());
  if matches!(exports_type, ExportsType::Dynamic) {
    runtime_requirements.insert(RuntimeGlobals::COMPAT_GET_DEFAULT_EXPORT);
    return (
      import_content,
      format!(
        "/* harmony import */{opt_declaration}{import_var}_default = /*#__PURE__*/{}({import_var});\n",
        RuntimeGlobals::COMPAT_GET_DEFAULT_EXPORT,
      ),
    );
  }
  (import_content, "".to_string())
}

pub fn module_namespace_promise(
  code_generatable_context: &mut TemplateContext,
  dep_id: &DependencyId,
  block: Option<&AsyncDependenciesBlockId>,
  request: &str,
  _message: &str,
  weak: bool,
) -> String {
  let TemplateContext {
    runtime_requirements,
    compilation,
    module,
    ..
  } = code_generatable_context;
  if compilation
    .module_graph
    .module_identifier_by_dependency_id(dep_id)
    .is_none()
  {
    return missing_module_promise(request);
  };

  let promise = block_promise(block, runtime_requirements, compilation);
  let exports_type = get_exports_type(&compilation.module_graph, dep_id, &module.identifier());
  let module_id_expr = module_id(compilation, dep_id, request, weak);

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
        appending = format!(
          ".then({}.bind({}, {module_id_expr}))",
          RuntimeGlobals::REQUIRE,
          RuntimeGlobals::REQUIRE
        );
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
      if matches!(
        compilation.module_graph.is_async(
          compilation
            .module_graph
            .module_identifier_by_dependency_id(dep_id)
            .expect("should have module")
        ),
        Some(true)
      ) {
        if let Some(header) = header {
          appending = format!(
            ".then(function() {{\n {header}\nreturn {}\n}})",
            module_raw(compilation, runtime_requirements, dep_id, request, weak)
          )
        } else {
          runtime_requirements.insert(RuntimeGlobals::REQUIRE);
          appending = format!(
            ".then({}.bind({}, {module_id_expr}))",
            RuntimeGlobals::REQUIRE,
            RuntimeGlobals::REQUIRE
          );
        }
        appending.push_str(
          format!(
            ".then(function(m){{\n {}(m, {fake_type}) \n}})",
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
            RuntimeGlobals::REQUIRE
          );
        }
      }
    }
  }

  format!("{promise}{appending}")
}

pub fn block_promise(
  block: Option<&AsyncDependenciesBlockId>,
  runtime_requirements: &mut RuntimeGlobals,
  compilation: &Compilation,
) -> String {
  let Some(block) = block else {
    // ImportEagerDependency
    return "Promise.resolve()".to_string();
  };
  let chunk_group = compilation
    .chunk_graph
    .get_block_chunk_group(block, &compilation.chunk_group_by_ukey);
  let Some(chunk_group) = chunk_group else {
    return "Promise.resolve()".to_string();
  };
  if chunk_group.chunks.is_empty() {
    return "Promise.resolve()".to_string();
  }
  let chunks = chunk_group
    .chunks
    .iter()
    .map(|c| compilation.chunk_by_ukey.expect_get(c))
    .filter(|c| !c.has_runtime(&compilation.chunk_group_by_ukey) && c.id.is_some())
    .collect::<Vec<_>>();
  if chunks.len() == 1 {
    let chunk_id = serde_json::to_string(chunks[0].id.as_ref().expect("should have chunk.id"))
      .expect("should able to json stringify");
    runtime_requirements.insert(RuntimeGlobals::ENSURE_CHUNK);
    format!("{}({chunk_id})", RuntimeGlobals::ENSURE_CHUNK)
  } else if !chunks.is_empty() {
    runtime_requirements.insert(RuntimeGlobals::ENSURE_CHUNK);
    format!(
      "Promise.all([{}])",
      chunks
        .iter()
        .map(|c| format!(
          "{}({})",
          RuntimeGlobals::ENSURE_CHUNK,
          serde_json::to_string(c.id.as_ref().expect("should have chunk.id"))
            .expect("should able to json stringify")
        ))
        .collect::<Vec<_>>()
        .join(", ")
    )
  } else {
    "Promise.resolve()".to_string()
  }
}

pub fn module_raw(
  compilation: &Compilation,
  runtime_requirements: &mut RuntimeGlobals,
  id: &DependencyId,
  request: &str,
  weak: bool,
) -> String {
  if let Some(module_identifier) = compilation
    .module_graph
    .module_identifier_by_dependency_id(id)
    && let Some(module_id) = compilation.chunk_graph.get_module_id(*module_identifier)
  {
    runtime_requirements.insert(RuntimeGlobals::REQUIRE);
    format!(
      "{}({})",
      RuntimeGlobals::REQUIRE,
      module_id_expr(request, module_id)
    )
  } else if weak {
    weak_error(request)
  } else {
    missing_module(request)
  }
}

fn missing_module(request: &str) -> String {
  format!("Object({}())", throw_missing_module_error_function(request))
}

fn missing_module_statement(request: &str) -> String {
  format!("{};\n", missing_module(request))
}

fn missing_module_promise(request: &str) -> String {
  format!(
    "Promise.resolve().then({})",
    throw_missing_module_error_function(request)
  )
}

fn throw_missing_module_error_function(request: &str) -> String {
  format!(
    "function webpackMissingModule() {{ {} }}",
    throw_missing_module_error_block(request)
  )
}

pub fn throw_missing_module_error_block(request: &str) -> String {
  let e = format!("Cannot find module '{}'", request);
  format!(
    "var e = new Error({}); e.code = 'MODULE_NOT_FOUND'; throw e;",
    json!(e)
  )
}

fn weak_error(request: &str) -> String {
  format!("var e = new Error('Module is not available (weak dependency), request is {request}'); e.code = 'MODULE_NOT_FOUND'; throw e;")
}

pub fn returning_function(return_value: &str, args: &str) -> String {
  format!("function({args}) {{ return {return_value}; }}")
}

pub fn basic_function(args: &str, body: &str) -> String {
  format!("function({args}) {{\n{body}\n}}")
}

pub fn sync_module_factory(
  dep: &DependencyId,
  request: &str,
  compilation: &Compilation,
  runtime_requirements: &mut RuntimeGlobals,
) -> String {
  let factory = returning_function(
    &module_raw(compilation, runtime_requirements, dep, request, false),
    "",
  );
  returning_function(&factory, "")
}

pub fn async_module_factory(
  block_id: &AsyncDependenciesBlockId,
  request: &str,
  compilation: &Compilation,
  runtime_requirements: &mut RuntimeGlobals,
) -> String {
  let block = block_id.expect_get(compilation);
  let dep = block.get_dependencies()[0];
  let ensure_chunk = block_promise(Some(block_id), runtime_requirements, compilation);
  let factory = returning_function(
    &module_raw(compilation, runtime_requirements, &dep, request, false),
    "",
  );
  returning_function(
    &if ensure_chunk.starts_with("Promise.resolve(") {
      factory
    } else {
      format!("{ensure_chunk}.then({})", returning_function(&factory, ""))
    },
    "",
  )
}
