use std::{
  fmt::Debug,
  sync::{Arc, LazyLock, Mutex},
};

use cow_utils::CowUtils;
use itertools::Itertools;
use regex::{Captures, Regex};
use rspack_dojang::{Context, Dojang, FunctionContainer, Operand};
use rspack_error::{Error, Result, ToStringResultToRspackResultExt, error};
use rspack_util::{fx_hash::FxIndexSet, json_stringify};
use rustc_hash::FxHashSet as HashSet;
use serde_json::{Map, Value, json};
use swc_core::atoms::Atom;

use crate::{
  AsyncDependenciesBlockIdentifier, ChunkGraph, Compilation, CompilerOptions, DependenciesBlock,
  DependencyId, DependencyType, ExportsArgument, ExportsInfoGetter, ExportsType,
  FakeNamespaceObjectMode, GetUsedNameParam, ImportPhase, InitFragmentExt, InitFragmentKey,
  InitFragmentStage, Module, ModuleArgument, ModuleGraph, ModuleGraphCacheArtifact, ModuleId,
  ModuleIdentifier, NormalInitFragment, PathInfo, PrefetchExportsInfoMode, RuntimeCondition,
  RuntimeGlobals, RuntimeSpec, TemplateContext, UsedName, compile_boolean_matcher_from_lists,
  contextify, property_access,
  runtime_globals::{RuntimeVariable, runtime_globals_to_string, runtime_variable_to_string},
  to_comment, to_normal_comment,
};

pub struct RuntimeTemplate {
  compiler_options: Arc<CompilerOptions>,
  runtime_globals: Arc<Map<String, Value>>,
  dojang: Option<Dojang>,
}

static RUNTIME_GLOBALS_PATTERN: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"\$\$RUNTIME_GLOBAL_(.*?)\$\$").expect("failed to create regex"));

fn replace_runtime_globals(template: String, runtime_globals: &Map<String, Value>) -> String {
  RUNTIME_GLOBALS_PATTERN
    .replace_all(&template, |caps: &Captures| {
      let name = caps.get(1).expect("name should be a string").as_str();
      runtime_globals
        .get(name)
        .map(|value| match value {
          Value::String(value) => value.clone(),
          _ => unreachable!(),
        })
        .expect("value should be a string")
    })
    .to_string()
}

impl Debug for RuntimeTemplate {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("runtime_template").finish()
  }
}

impl RuntimeTemplate {
  pub fn new(compiler_options: Arc<CompilerOptions>) -> Self {
    let runtime_globals = Arc::new(
      RuntimeGlobals::all()
        .iter_names()
        .map(|(name, value)| {
          (
            name.to_string(),
            Value::String(runtime_globals_to_string(&value, &compiler_options)),
          )
        })
        .collect::<Map<String, Value>>(),
    );

    let mut dojang = Dojang::new();

    let runtime_globals_cloned = runtime_globals.clone();
    let compiler_options_cloned = compiler_options.clone();
    dojang.functions.insert(
      "basicFunction".into(),
      FunctionContainer::F1(Box::new(move |args: Operand| {
        dojang_basic_function(args, &runtime_globals_cloned, &compiler_options_cloned)
      })),
    );

    let runtime_globals_cloned = runtime_globals.clone();
    let compiler_options_cloned = compiler_options.clone();
    dojang.functions.insert(
      "returningFunction".into(),
      FunctionContainer::F2(Box::new(move |args: Operand, return_value: Operand| {
        dojang_returning_function(
          args,
          return_value,
          &runtime_globals_cloned,
          &compiler_options_cloned,
        )
      })),
    );

    let runtime_globals_cloned = runtime_globals.clone();
    let compiler_options_cloned = compiler_options.clone();
    dojang.functions.insert(
      "expressionFunction".into(),
      FunctionContainer::F2(Box::new(move |args: Operand, expression: Operand| {
        dojang_expression_function(
          args,
          expression,
          &runtime_globals_cloned,
          &compiler_options_cloned,
        )
      })),
    );

    let compiler_options_cloned = compiler_options.clone();
    dojang.functions.insert(
      "emptyFunction".into(),
      FunctionContainer::F0(Box::new(move || {
        dojang_empty_function(&compiler_options_cloned)
      })),
    );

    let runtime_globals_cloned = runtime_globals.clone();
    let compiler_options_cloned = compiler_options.clone();
    dojang.functions.insert(
      "destructureArray".into(),
      FunctionContainer::F2(Box::new(move |items: Operand, value: Operand| {
        dojang_array_destructure(
          items,
          value,
          &runtime_globals_cloned,
          &compiler_options_cloned,
        )
      })),
    );

    Self {
      compiler_options,
      runtime_globals,
      dojang: Some(dojang),
    }
  }

  pub fn clone_without_dojang(&self) -> Arc<Self> {
    Arc::new(Self {
      compiler_options: self.compiler_options.clone(),
      runtime_globals: self.runtime_globals.clone(),
      dojang: None,
    })
  }

  pub fn add_templates(&mut self, templates: Vec<(String, String)>) {
    for (key, template) in templates {
      if !self
        .dojang
        .as_ref()
        .expect("dojang should be initialized")
        .templates
        .contains_key(&key)
      {
        self
          .dojang
          .as_mut()
          .expect("dojang should be initialized")
          .add_with_option(key.clone(), template)
          .unwrap_or_else(|_| panic!("failed to add template {key}"));
      }
    }
  }

  pub fn render(&self, key: &str, params: Option<serde_json::Value>) -> Result<String, Error> {
    let mut render_params = Value::Object(Default::default());

    render_params
      .as_object_mut()
      .unwrap_or_else(|| unreachable!())
      .extend(
        self
          .runtime_globals
          .iter()
          .map(|(k, v)| (k.clone(), v.clone())),
      );

    if let Some(params) = params {
      match params {
        Value::Object(params) => {
          for (k, v) in params {
            render_params
              .as_object_mut()
              .unwrap_or_else(|| unreachable!())
              .insert(k, v);
          }
        }
        _ => panic!("Should receive a map value"),
      }
    }

    if let Some((executer, file_content)) = self
      .dojang
      .as_ref()
      .expect("dojang should be initialized")
      .templates
      .get(key)
    {
      executer
        .render(
          &mut Context::new(render_params),
          &self
            .dojang
            .as_ref()
            .expect("dojang should be initialized")
            .templates,
          &self
            .dojang
            .as_ref()
            .expect("dojang should be initialized")
            .functions,
          file_content,
          &mut Mutex::new(std::collections::HashMap::new()),
        )
        // Replace Windows-style line endings (\r\n) with Unix-style (\n) to ensure consistent runtime templates across platforms
        .map(|render| render.cow_replace("\r\n", "\n").to_string())
        .to_rspack_result_with_message(|e| {
          format!("Runtime module: failed to render template {key} from: {e}")
        })
    } else {
      Err(error!("Runtime module: Template {key} is not found"))
    }
  }

  pub fn render_runtime_globals(&self, runtime_globals: &RuntimeGlobals) -> String {
    runtime_globals_to_string(runtime_globals, &self.compiler_options)
  }

  pub fn render_runtime_variable(&self, runtime_variable: &RuntimeVariable) -> String {
    runtime_variable_to_string(runtime_variable, &self.compiler_options)
  }
}

fn to_string(val: &Operand, runtime_globals: &Map<String, Value>) -> String {
  replace_runtime_globals(
    match val {
      Operand::Value(val) => val.as_str().unwrap_or_default().to_string(),
      _ => String::default(),
    },
    runtime_globals,
  )
}

fn join_to_string(val: &Operand, sep: &str, runtime_globals: &Map<String, Value>) -> String {
  replace_runtime_globals(
    match val {
      Operand::Array(items) => items
        .iter()
        .map(|item| to_string(item, runtime_globals))
        .join(sep),
      _ => to_string(val, runtime_globals),
    },
    runtime_globals,
  )
}

fn dojang_basic_function(
  args: Operand,
  runtime_globals: &Map<String, Value>,
  compiler_options: &Arc<CompilerOptions>,
) -> Operand {
  if compiler_options
    .output
    .environment
    .supports_arrow_function()
  {
    Operand::Value(Value::from(format!(
      r#"({}) =>"#,
      join_to_string(&args, ", ", runtime_globals)
    )))
  } else {
    Operand::Value(Value::from(format!(
      r#"function({})"#,
      join_to_string(&args, ", ", runtime_globals)
    )))
  }
}

fn dojang_returning_function(
  return_value: Operand,
  args: Operand,
  runtime_globals: &Map<String, Value>,
  compiler_options: &Arc<CompilerOptions>,
) -> Operand {
  if compiler_options
    .output
    .environment
    .supports_arrow_function()
  {
    Operand::Value(Value::from(format!(
      "({}) => ({})",
      join_to_string(&args, ", ", runtime_globals),
      to_string(&return_value, runtime_globals)
    )))
  } else {
    Operand::Value(Value::from(format!(
      "function({}) {{ return {}; }}",
      join_to_string(&args, ", ", runtime_globals),
      to_string(&return_value, runtime_globals)
    )))
  }
}

fn dojang_expression_function(
  expression: Operand,
  args: Operand,
  runtime_globals: &Map<String, Value>,
  compiler_options: &Arc<CompilerOptions>,
) -> Operand {
  if compiler_options
    .output
    .environment
    .supports_arrow_function()
  {
    Operand::Value(Value::from(format!(
      "({}) => ({})",
      join_to_string(&args, ", ", runtime_globals),
      to_string(&expression, runtime_globals)
    )))
  } else {
    Operand::Value(Value::from(format!(
      "function({}) {{ {}; }}",
      join_to_string(&args, ", ", runtime_globals),
      to_string(&expression, runtime_globals)
    )))
  }
}

fn dojang_empty_function(compiler_options: &Arc<CompilerOptions>) -> Operand {
  if compiler_options
    .output
    .environment
    .supports_arrow_function()
  {
    Operand::Value(Value::from("() => {}"))
  } else {
    Operand::Value(Value::from("function() {}"))
  }
}

fn dojang_array_destructure(
  items: Operand,
  value: Operand,
  runtime_globals: &Map<String, Value>,
  compiler_options: &Arc<CompilerOptions>,
) -> Operand {
  if compiler_options.output.environment.supports_destructuring() {
    Operand::Value(Value::from(format!(
      "var [{}] = {};",
      join_to_string(&items, ", ", runtime_globals),
      to_string(&value, runtime_globals)
    )))
  } else {
    let value_name = to_string(&value, runtime_globals);
    let items = match items {
      Operand::Array(items) => items
        .iter()
        .enumerate()
        .map(|(idx, item)| {
          let item_name = to_string(item, runtime_globals);
          if item_name.is_empty() {
            String::default()
          } else {
            format!("var {item_name} = {value_name}[{idx}];")
          }
        })
        .join("\n"),
      Operand::Value(val) => val
        .as_str()
        .unwrap_or_default()
        .split(",")
        .enumerate()
        .map(|(idx, item)| {
          let item_name = item.trim().to_string();
          if item_name.is_empty() {
            String::default()
          } else {
            format!("var {item_name} = {value_name}[{idx}];")
          }
        })
        .join("\n"),
      _ => String::default(),
    };
    Operand::Value(Value::from(items))
  }
}

// information content of the comment
#[derive(Default)]
struct CommentOptions<'a> {
  // request string used originally
  request: Option<&'a str>,
  // name of the chunk referenced
  chunk_name: Option<&'a str>,
  // additional message
  message: Option<&'a str>,
}

impl RuntimeTemplate {
  pub fn returning_function(&self, return_value: &str, args: &str) -> String {
    if self
      .compiler_options
      .output
      .environment
      .supports_arrow_function()
    {
      format!("({args}) => ({return_value})")
    } else {
      format!("function({args}) {{ return {return_value}; }}")
    }
  }

  pub fn basic_function(&self, args: &str, body: &str) -> String {
    if self
      .compiler_options
      .output
      .environment
      .supports_arrow_function()
    {
      format!("({args}) => {{\n{body}\n}}")
    } else {
      format!("function({args}) {{\n{body}\n}}")
    }
  }

  pub fn runtime_condition_expression(
    &self,
    chunk_graph: &ChunkGraph,
    runtime_condition: Option<&RuntimeCondition>,
    runtime: Option<&RuntimeSpec>,
    runtime_requirements: &mut RuntimeGlobals,
  ) -> String {
    let Some(runtime_condition) = runtime_condition else {
      return "true".to_string();
    };

    let runtime_condition = match runtime_condition {
      RuntimeCondition::Boolean(v) => return v.to_string(),
      RuntimeCondition::Spec(spec) => spec,
    };

    let mut positive_runtime_ids = HashSet::default();
    for_each_runtime(
      Some(runtime_condition),
      |runtime| {
        if let Some(runtime_id) =
          runtime.and_then(|runtime| chunk_graph.get_runtime_id(runtime.as_str()))
        {
          positive_runtime_ids.insert(runtime_id);
        }
      },
      false,
    );

    let mut negative_runtime_ids = HashSet::default();
    for_each_runtime(
      subtract_runtime(runtime, Some(runtime_condition)).as_ref(),
      |runtime| {
        if let Some(runtime_id) =
          runtime.and_then(|runtime| chunk_graph.get_runtime_id(runtime.as_str()))
        {
          negative_runtime_ids.insert(runtime_id);
        }
      },
      false,
    );

    runtime_requirements.insert(RuntimeGlobals::RUNTIME_ID);

    compile_boolean_matcher_from_lists(
      positive_runtime_ids.into_iter().collect::<Vec<_>>(),
      negative_runtime_ids.into_iter().collect::<Vec<_>>(),
    )
    .render(
      self
        .render_runtime_globals(&RuntimeGlobals::RUNTIME_ID)
        .as_str(),
    )
  }

  #[allow(clippy::too_many_arguments)]
  pub fn export_from_import(
    &self,
    code_generatable_context: &mut TemplateContext,
    default_interop: bool,
    request: &str,
    import_var: &str,
    export_name: &[Atom],
    id: &DependencyId,
    is_call: bool,
    call_context: bool,
    asi_safe: Option<bool>,
    phase: ImportPhase,
  ) -> String {
    let TemplateContext {
      runtime_requirements,
      compilation,
      init_fragments,
      module,
      runtime,
      ..
    } = code_generatable_context;
    let mg = compilation.get_module_graph();
    let Some(target_module) = mg.get_module_by_dependency_id(id) else {
      return self.missing_module(request);
    };

    let exports_type = get_exports_type(
      &mg,
      &compilation.module_graph_cache_artifact,
      id,
      &module.identifier(),
    );

    let target_module_identifier = target_module.identifier();

    let is_deferred = phase.is_defer() && !target_module.build_meta().has_top_level_await;

    let mut exclude_default_export_name = None;
    if default_interop {
      if !export_name.is_empty()
        && let Some(first_export_name) = export_name.first()
        && first_export_name == "default"
      {
        if is_deferred && !matches!(exports_type, ExportsType::Namespace) {
          let name = &export_name[1..];
          let Some(used) = ExportsInfoGetter::get_used_name(
            GetUsedNameParam::WithNames(&mg.get_prefetched_exports_info(
              &target_module_identifier,
              PrefetchExportsInfoMode::Nested(name),
            )),
            *runtime,
            name,
          ) else {
            return to_normal_comment(&format!(
              "unused export {}",
              property_access(export_name, 0)
            )) + "undefined";
          };
          let UsedName::Normal(used) = used else {
            unreachable!("can't inline the exports of defer imported module")
          };
          let access = format!("{import_var}.a{}", property_access(used, 0));
          if is_call {
            return access;
          }
          let Some(asi_safe) = asi_safe else {
            return access;
          };
          return if asi_safe {
            format!("({access})")
          } else {
            format!(";({access})")
          };
        }
        match exports_type {
          ExportsType::Dynamic => {
            if is_call {
              return format!("{import_var}_default(){}", property_access(export_name, 1));
            } else {
              return if let Some(asi_safe) = asi_safe {
                match asi_safe {
                  true => format!(
                    "({import_var}_default(){})",
                    property_access(export_name, 1)
                  ),
                  false => format!(
                    ";({import_var}_default(){})",
                    property_access(export_name, 1)
                  ),
                }
              } else {
                format!("{import_var}_default.a{}", property_access(export_name, 1))
              };
            }
          }
          ExportsType::DefaultOnly | ExportsType::DefaultWithNamed => {
            exclude_default_export_name = Some(export_name[1..].to_vec());
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
      } else if is_deferred {
        // now exportName.length is 0
        // fall through to the end of this function, create the namespace there.
      } else if matches!(
        exports_type,
        ExportsType::DefaultOnly | ExportsType::DefaultWithNamed
      ) {
        runtime_requirements.insert(RuntimeGlobals::CREATE_FAKE_NAMESPACE_OBJECT);

        let name = format!("var {import_var}_namespace_cache;\n");
        init_fragments.push(
          NormalInitFragment::new(
            name.clone(),
            InitFragmentStage::StageESMExports,
            -1,
            InitFragmentKey::ESMFakeNamespaceObjectFragment(name),
            None,
          )
          .boxed(),
        );
        let prefix = if let Some(asi_safe) = asi_safe {
          match asi_safe {
            true => "",
            false => ";",
          }
        } else {
          "Object"
        };
        return format!(
          "/*#__PURE__*/ {prefix}({import_var}_namespace_cache || ({import_var}_namespace_cache = {}({import_var}{})))",
          self.render_runtime_globals(&RuntimeGlobals::CREATE_FAKE_NAMESPACE_OBJECT),
          if matches!(exports_type, ExportsType::DefaultOnly) {
            ""
          } else {
            ", 2"
          }
        );
      }
    }

    let export_name = exclude_default_export_name
      .as_deref()
      .unwrap_or(export_name);
    if !export_name.is_empty() {
      let used_name = match ExportsInfoGetter::get_used_name(
        GetUsedNameParam::WithNames(&mg.get_prefetched_exports_info(
          &target_module_identifier,
          PrefetchExportsInfoMode::Nested(export_name),
        )),
        *runtime,
        export_name,
      ) {
        Some(UsedName::Normal(used_name)) => used_name,
        Some(UsedName::Inlined(inlined)) => {
          assert!(
            !is_deferred,
            "can't inline the exports of defer imported module"
          );
          return format!(
            "{} {}",
            to_normal_comment(&format!(
              "inlined export {}",
              property_access(export_name, 0)
            )),
            inlined.render()
          );
        }
        None => {
          return format!(
            "{} undefined",
            to_normal_comment(&format!(
              "unused export {}",
              property_access(export_name, 0)
            ))
          );
        }
      };
      let comment = if used_name != export_name {
        to_normal_comment(&property_access(export_name, 0))
      } else {
        String::new()
      };
      let property = property_access(used_name, 0);
      let access = format!(
        "{import_var}{}{comment}{property}",
        if is_deferred { ".a" } else { "" }
      );
      if is_call && !call_context {
        if let Some(asi_safe) = asi_safe {
          match asi_safe {
            true => format!("(0,{access})"),
            false => format!(";(0,{access})"),
          }
        } else {
          format!("Object({access})")
        }
      } else {
        access
      }
    } else if is_deferred {
      let cache_var = format!("var {import_var}_deferred_namespace_cache;\n");
      init_fragments.push(
        NormalInitFragment::new(
          cache_var.clone(),
          InitFragmentStage::StageConstants,
          -1,
          InitFragmentKey::ESMDeferImportNamespaceObjectFragment(cache_var),
          None,
        )
        .boxed(),
      );
      runtime_requirements.insert(RuntimeGlobals::MAKE_DEFERRED_NAMESPACE_OBJECT);
      let module_id =
        ChunkGraph::get_module_id(&compilation.module_ids_artifact, target_module_identifier);
      let mode = render_make_deferred_namespace_mode_from_exports_type(exports_type);
      format!(
        "/*#__PURE__*/ {}({import_var}_deferred_namespace_cache || ({import_var}_deferred_namespace_cache = {}({}, {})))",
        match asi_safe {
          Some(true) => "",
          Some(false) => ";",
          None => "Object",
        },
        self.render_runtime_globals(&RuntimeGlobals::MAKE_DEFERRED_NAMESPACE_OBJECT),
        json_stringify(&module_id),
        mode,
      )
    } else {
      import_var.to_string()
    }
  }

  #[allow(clippy::too_many_arguments)]
  pub fn import_statement(
    &self,
    module: &dyn Module,
    compilation: &Compilation,
    runtime_requirements: &mut RuntimeGlobals,
    id: &DependencyId,
    import_var: &str,
    request: &str,
    phase: ImportPhase,
    update: bool, // whether a new variable should be created or the existing one updated
  ) -> (String, String) {
    let mg = compilation.get_module_graph();
    let Some(target_module) = mg.get_module_by_dependency_id(id) else {
      return (self.missing_module_statement(request), String::new());
    };

    let module_id_expr = self.module_id(compilation, id, request, false);

    runtime_requirements.insert(RuntimeGlobals::REQUIRE);

    let opt_declaration = if update { "" } else { "var " };

    let exports_type = get_exports_type(
      &mg,
      &compilation.module_graph_cache_artifact,
      id,
      &module.identifier(),
    );

    if phase.is_defer() && !target_module.build_meta().has_top_level_await {
      let async_deps = get_outgoing_async_modules(compilation, target_module.as_ref());
      let import_content = format!(
        "/* deferred import */{opt_declaration}{import_var} = {};\n",
        self.get_property_accessed_deferred_module(exports_type, &module_id_expr, async_deps)
      );
      return (import_content, String::new());
    }

    let import_content = format!(
      "/* import */ {opt_declaration}{import_var} = {}({module_id_expr});\n",
      self.render_runtime_globals(&RuntimeGlobals::REQUIRE)
    );
    if matches!(exports_type, ExportsType::Dynamic) {
      runtime_requirements.insert(RuntimeGlobals::COMPAT_GET_DEFAULT_EXPORT);
      return (
        import_content,
        format!(
          "/* import */ {opt_declaration}{import_var}_default = /*#__PURE__*/{}({import_var});\n",
          self.render_runtime_globals(&RuntimeGlobals::COMPAT_GET_DEFAULT_EXPORT),
        ),
      );
    }
    (import_content, String::new())
  }

  // add a comment
  fn comment(&self, comment_options: CommentOptions) -> String {
    let used_pathinfo = matches!(
      self.compiler_options.output.pathinfo,
      PathInfo::Bool(true) | PathInfo::String(_)
    );
    let content = if used_pathinfo {
      vec![
        comment_options.message,
        comment_options.request,
        comment_options.chunk_name,
      ]
    } else {
      vec![comment_options.message, comment_options.chunk_name]
    }
    .iter()
    .filter_map(|&item| item)
    .map(|item| contextify(self.compiler_options.context.as_path(), item))
    .collect::<Vec<_>>()
    .join(" | ");

    if content.is_empty() {
      return String::new();
    }

    if used_pathinfo {
      format!("{} ", to_comment(&content))
    } else {
      format!("{} ", to_normal_comment(&content))
    }
  }

  pub fn module_id_expr(&self, request: &str, module_id: &ModuleId) -> String {
    format!(
      "{}{}",
      self.comment(CommentOptions {
        request: Some(request),
        ..Default::default()
      }),
      json_stringify(module_id)
    )
  }

  pub fn module_id(
    &self,
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
      self.module_id_expr(request, module_id)
    } else if weak {
      "null /* weak dependency, without id */".to_string()
    } else {
      self.missing_module(request)
    }
  }

  pub fn module_namespace_promise(
    &self,
    code_generatable_context: &mut TemplateContext,
    dep_id: &DependencyId,
    block: Option<&AsyncDependenciesBlockIdentifier>,
    request: &str,
    message: &str,
    weak: bool,
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
      return self.missing_module_promise(request);
    };

    let promise = self.block_promise(block, runtime_requirements, compilation, message);
    let exports_type = get_exports_type(
      &compilation.get_module_graph(),
      &compilation.module_graph_cache_artifact,
      dep_id,
      &module.identifier(),
    );
    let module_id_expr = self.module_id(compilation, dep_id, request, weak);

    let header = if weak {
      runtime_requirements.insert(RuntimeGlobals::MODULE_FACTORIES);
      Some(format!(
        "if(!{}[{module_id_expr}]) {{\n {} \n}}",
        self.render_runtime_globals(&RuntimeGlobals::MODULE_FACTORIES),
        self.weak_error(request)
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
            self.module_raw(compilation, runtime_requirements, dep_id, request, weak)
          )
        } else {
          runtime_requirements.insert(RuntimeGlobals::REQUIRE);
          appending = format!(
            ".then({}.bind({}, {module_id_expr}))",
            self.render_runtime_globals(&RuntimeGlobals::REQUIRE),
            self.render_runtime_globals(&RuntimeGlobals::REQUIRE),
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
              self.module_raw(compilation, runtime_requirements, dep_id, request, weak)
            )
          } else {
            runtime_requirements.insert(RuntimeGlobals::REQUIRE);
            appending = format!(
              ".then({}.bind({}, {module_id_expr}))",
              self.render_runtime_globals(&RuntimeGlobals::REQUIRE),
              self.render_runtime_globals(&RuntimeGlobals::REQUIRE),
            );
          }
          appending.push_str(
            format!(
              ".then(function(m){{\n return {}(m, {fake_type}) \n}})",
              self.render_runtime_globals(&RuntimeGlobals::CREATE_FAKE_NAMESPACE_OBJECT)
            )
            .as_str(),
          );
        } else {
          fake_type |= FakeNamespaceObjectMode::MODULE_ID;
          if let Some(header) = header {
            let expr = format!(
              "{}({module_id_expr}, {fake_type}))",
              self.render_runtime_globals(&RuntimeGlobals::CREATE_FAKE_NAMESPACE_OBJECT)
            );
            appending = format!(".then(function() {{\n {header} return {expr};\n}})");
          } else {
            runtime_requirements.insert(RuntimeGlobals::REQUIRE);
            appending = format!(
              ".then({}.bind({}, {module_id_expr}, {fake_type}))",
              self.render_runtime_globals(&RuntimeGlobals::CREATE_FAKE_NAMESPACE_OBJECT),
              self.render_runtime_globals(&RuntimeGlobals::REQUIRE),
            );
          }
        }
      }
    }

    format!("{promise}{appending}")
  }

  pub fn block_promise(
    &self,
    block: Option<&AsyncDependenciesBlockIdentifier>,
    runtime_requirements: &mut RuntimeGlobals,
    compilation: &Compilation,
    message: &str,
  ) -> String {
    let Some(block) = block else {
      let comment = self.comment(CommentOptions {
        request: None,
        chunk_name: None,
        message: Some(message),
      });
      return format!("Promise.resolve({})", comment.trim());
    };
    let chunk_group = compilation
      .chunk_graph
      .get_block_chunk_group(block, &compilation.chunk_group_by_ukey);
    let Some(chunk_group) = chunk_group else {
      let comment = self.comment(CommentOptions {
        request: None,
        chunk_name: None,
        message: Some(message),
      });
      return format!("Promise.resolve({})", comment.trim());
    };
    if chunk_group.chunks.is_empty() {
      let comment = self.comment(CommentOptions {
        request: None,
        chunk_name: None,
        message: Some(message),
      });
      return format!("Promise.resolve({})", comment.trim());
    }
    let mg = compilation.get_module_graph();
    let block = mg.block_by_id_expect(block);
    let comment = self.comment(CommentOptions {
      request: None,
      chunk_name: block.get_group_options().and_then(|o| o.name()),
      message: Some(message),
    });
    let chunks = chunk_group
      .chunks
      .iter()
      .map(|c| compilation.chunk_by_ukey.expect_get(c))
      .filter(|c| {
        !c.has_runtime(&compilation.chunk_group_by_ukey)
          && c.id(&compilation.chunk_ids_artifact).is_some()
      })
      .collect::<Vec<_>>();

    if chunks.len() == 1 {
      let chunk_id = serde_json::to_string(
        chunks[0]
          .id(&compilation.chunk_ids_artifact)
          .expect("should have chunk.id"),
      )
      .expect("should able to json stringify");
      runtime_requirements.insert(RuntimeGlobals::ENSURE_CHUNK);

      let fetch_priority = chunk_group
        .kind
        .get_normal_options()
        .and_then(|x| x.fetch_priority);

      if fetch_priority.is_some() {
        runtime_requirements.insert(RuntimeGlobals::HAS_FETCH_PRIORITY);
      }

      format!(
        "{}({comment}{chunk_id}{})",
        compilation
          .runtime_template
          .render_runtime_globals(&RuntimeGlobals::ENSURE_CHUNK),
        fetch_priority
          .map(|x| format!(r#", "{x}""#))
          .unwrap_or_default()
      )
    } else if !chunks.is_empty() {
      runtime_requirements.insert(RuntimeGlobals::ENSURE_CHUNK);

      let fetch_priority = chunk_group
        .kind
        .get_normal_options()
        .and_then(|x| x.fetch_priority);

      if fetch_priority.is_some() {
        runtime_requirements.insert(RuntimeGlobals::HAS_FETCH_PRIORITY);
      }

      format!(
        "Promise.all({comment}[{}])",
        chunks
          .iter()
          .map(|c| format!(
            "{}({}{})",
            self.render_runtime_globals(&RuntimeGlobals::ENSURE_CHUNK),
            serde_json::to_string(
              c.id(&compilation.chunk_ids_artifact)
                .expect("should have chunk.id")
            )
            .expect("should able to json stringify"),
            fetch_priority
              .map(|x| format!(r#", "{x}""#))
              .unwrap_or_default()
          ))
          .collect::<Vec<_>>()
          .join(", ")
      )
    } else {
      format!("Promise.resolve({})", comment.trim())
    }
  }

  pub fn module_raw(
    &self,
    compilation: &Compilation,
    runtime_requirements: &mut RuntimeGlobals,
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
      runtime_requirements.insert(RuntimeGlobals::REQUIRE);
      format!(
        "{}({})",
        self.render_runtime_globals(&RuntimeGlobals::REQUIRE),
        self.module_id_expr(request, module_id)
      )
    } else if weak {
      self.weak_error(request)
    } else {
      self.missing_module(request)
    }
  }

  pub fn sync_module_factory(
    &self,
    dep: &DependencyId,
    request: &str,
    compilation: &Compilation,
    runtime_requirements: &mut RuntimeGlobals,
  ) -> String {
    let factory = self.returning_function(
      &self.module_raw(compilation, runtime_requirements, dep, request, false),
      "",
    );
    self.returning_function(&factory, "")
  }

  pub fn async_module_factory(
    &self,
    block_id: &AsyncDependenciesBlockIdentifier,
    request: &str,
    compilation: &Compilation,
    runtime_requirements: &mut RuntimeGlobals,
  ) -> String {
    let module_graph = compilation.get_module_graph();
    let block = module_graph
      .block_by_id(block_id)
      .expect("should have block");
    let dep = block.get_dependencies()[0];
    let ensure_chunk = self.block_promise(Some(block_id), runtime_requirements, compilation, "");
    let factory = self.returning_function(
      &self.module_raw(compilation, runtime_requirements, &dep, request, false),
      "",
    );
    self.returning_function(
      &if ensure_chunk.starts_with("Promise.resolve(") {
        factory
      } else {
        format!(
          "{ensure_chunk}.then({})",
          self.returning_function(&factory, "")
        )
      },
      "",
    )
  }

  pub fn define_es_module_flag_statement(
    &self,
    exports_argument: ExportsArgument,
    runtime_requirements: &mut RuntimeGlobals,
  ) -> String {
    runtime_requirements.insert(RuntimeGlobals::MAKE_NAMESPACE_OBJECT);
    runtime_requirements.insert(RuntimeGlobals::EXPORTS);

    format!(
      "{}({});\n",
      self.render_runtime_globals(&RuntimeGlobals::MAKE_NAMESPACE_OBJECT),
      self.render_exports_argument(exports_argument)
    )
  }

  pub fn render_exports_argument(&self, exports_argument: ExportsArgument) -> String {
    match exports_argument {
      ExportsArgument::Exports => "exports".to_string(),
      ExportsArgument::RspackExports => self.render_runtime_variable(&RuntimeVariable::Exports),
    }
  }

  pub fn render_module_argument(&self, module_argument: ModuleArgument) -> String {
    match module_argument {
      ModuleArgument::Module => "module".to_string(),
      ModuleArgument::RspackModule => self.render_runtime_variable(&RuntimeVariable::Module),
    }
  }

  pub fn get_property_accessed_deferred_module(
    &self,
    exports_type: ExportsType,
    module_id_expr: &str,
    async_deps: FxIndexSet<ModuleId>,
  ) -> String {
    let is_async = !async_deps.is_empty();
    let mut content = "{\nget a() {\n  ".to_string();
    let namespace_or_dynamic =
      matches!(exports_type, ExportsType::Namespace | ExportsType::Dynamic);
    if namespace_or_dynamic {
      content += "var exports = ";
    } else {
      content += "return ";
    }
    content += &self.render_runtime_globals(&RuntimeGlobals::REQUIRE);
    content += "(";
    content += module_id_expr;
    content += ")";
    if is_async {
      content += "[";
      content += &self.render_runtime_globals(&RuntimeGlobals::ASYNC_MODULE_EXPORT_SYMBOL);
      content += "]";
    }
    content += ";\n";
    if namespace_or_dynamic {
      content += "  ";
      if matches!(exports_type, ExportsType::Dynamic) {
        content += "if (exports.__esModule) ";
      }
      content += "Object.defineProperty(this, \"a\", { value: exports });\n  ";
      content += "return exports;\n";
    }
    content += "},\n";
    if is_async {
      content += "[";
      content +=
        &self.render_runtime_globals(&RuntimeGlobals::MAKE_DEFERRED_NAMESPACE_OBJECT_SYMBOL);
      content += "]: ";
      content += &json_stringify(&async_deps);
      content += ",\n";
    }
    content += "}";
    content
  }

  pub fn missing_module(&self, request: &str) -> String {
    format!(
      "Object({}())",
      self.throw_missing_module_error_function(request)
    )
  }

  pub fn missing_module_statement(&self, request: &str) -> String {
    format!("{};\n", self.missing_module(request))
  }

  pub fn missing_module_promise(&self, request: &str) -> String {
    format!(
      "Promise.resolve().then({})",
      self.throw_missing_module_error_function(request)
    )
  }

  pub fn throw_missing_module_error_function(&self, request: &str) -> String {
    format!(
      "function __rspack_missing_module() {{ {} }}",
      self.throw_missing_module_error_block(request)
    )
  }

  pub fn throw_missing_module_error_block(&self, request: &str) -> String {
    let e = format!("Cannot find module '{request}'");
    format!(
      "var e = new Error({}); e.code = 'MODULE_NOT_FOUND'; throw e;",
      json!(e)
    )
  }

  pub fn weak_error(&self, request: &str) -> String {
    format!(
      "var e = new Error('Module is not available (weak dependency), request is {request}'); e.code = 'MODULE_NOT_FOUND'; throw e;"
    )
  }
}

fn subtract_runtime(a: Option<&RuntimeSpec>, b: Option<&RuntimeSpec>) -> Option<RuntimeSpec> {
  match (a, b) {
    (Some(a), None) => Some(a.clone()),
    (None, None) => None,
    (None, Some(b)) => Some(b.clone()),
    (Some(a), Some(b)) => Some(a.subtract(b)),
  }
}

pub fn for_each_runtime<F>(runtime: Option<&RuntimeSpec>, mut f: F, deterministic_order: bool)
where
  F: FnMut(Option<&ustr::Ustr>),
{
  match runtime {
    None => f(None),
    Some(runtime) => {
      if deterministic_order {
        for r in runtime.iter().sorted() {
          f(Some(r));
        }
      } else {
        for r in runtime.iter() {
          f(Some(r));
        }
      }
    }
  }
}

pub fn render_make_deferred_namespace_mode_from_exports_type(exports_type: ExportsType) -> String {
  match exports_type {
    ExportsType::Namespace => "0".to_string(),
    ExportsType::DefaultOnly => "1".to_string(),
    ExportsType::DefaultWithNamed => "2".to_string(),
    ExportsType::Dynamic => "3".to_string(),
  }
}

pub fn get_exports_type(
  module_graph: &ModuleGraph,
  module_graph_cache: &ModuleGraphCacheArtifact,
  id: &DependencyId,
  parent_module: &ModuleIdentifier,
) -> ExportsType {
  let strict = module_graph
    .module_by_identifier(parent_module)
    .expect("should have mgm")
    .get_strict_esm_module();
  get_exports_type_with_strict(module_graph, module_graph_cache, id, strict)
}

pub fn get_exports_type_with_strict(
  module_graph: &ModuleGraph,
  module_graph_cache: &ModuleGraphCacheArtifact,
  id: &DependencyId,
  strict: bool,
) -> ExportsType {
  let module = module_graph
    .module_identifier_by_dependency_id(id)
    .expect("should have module");
  module_graph
    .module_by_identifier(module)
    .expect("should have module")
    .get_exports_type(module_graph, module_graph_cache, strict)
}

fn get_outgoing_async_modules(
  compilation: &Compilation,
  module: &dyn Module,
) -> FxIndexSet<ModuleId> {
  fn helper(
    compilation: &Compilation,
    mg: &ModuleGraph,
    module: &dyn Module,
    set: &mut FxIndexSet<ModuleId>,
    visited: &mut HashSet<ModuleIdentifier>,
  ) {
    let module_identifier = module.identifier();
    if !ModuleGraph::is_async(&compilation.async_modules_artifact, &module_identifier) {
      return;
    }
    if !visited.insert(module_identifier) {
      return;
    }
    if module.build_meta().has_top_level_await {
      set.insert(
        ChunkGraph::get_module_id(&compilation.module_ids_artifact, module_identifier)
          .expect("should have module_id")
          .clone(),
      );
    } else {
      for (module, connections) in mg.get_outcoming_connections_by_module(&module_identifier) {
        let is_esm = connections.iter().any(|connection| {
          mg.dependency_by_id(&connection.dependency_id)
            .map(|dep| {
              matches!(
                dep.dependency_type(),
                DependencyType::EsmImport | DependencyType::EsmExportImport
              )
            })
            .unwrap_or_default()
        });
        if is_esm {
          helper(
            compilation,
            mg,
            mg.module_by_identifier(&module)
              .expect("should have module")
              .as_ref(),
            set,
            visited,
          );
        }
      }
    }
  }

  let mut set = FxIndexSet::default();
  let mut visited = HashSet::default();
  helper(
    compilation,
    &compilation.get_module_graph(),
    module,
    &mut set,
    &mut visited,
  );
  set
}
