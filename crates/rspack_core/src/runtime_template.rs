use std::{
  collections::HashMap,
  fmt::Debug,
  sync::{Arc, LazyLock, Mutex},
};

use cow_utils::CowUtils;
use itertools::Itertools;
use regex::{Captures, Regex};
use rspack_dojang::{Context, Dojang, FunctionContainer, Operand};
use rspack_error::{Error, Result, ToStringResultToRspackResultExt, error};
use serde_json::{Map, Value};

use crate::{CompilerOptions, RuntimeGlobals};

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
        basic_function(args, &runtime_globals_cloned, &compiler_options_cloned)
      })),
    );

    let runtime_globals_cloned = runtime_globals.clone();
    let compiler_options_cloned = compiler_options.clone();
    dojang.functions.insert(
      "returningFunction".into(),
      FunctionContainer::F2(Box::new(move |args: Operand, return_value: Operand| {
        returning_function(
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
        expression_function(
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
      FunctionContainer::F0(Box::new(move || empty_function(&compiler_options_cloned))),
    );

    let runtime_globals_cloned = runtime_globals.clone();
    let compiler_options_cloned = compiler_options.clone();
    dojang.functions.insert(
      "destructureArray".into(),
      FunctionContainer::F2(Box::new(move |items: Operand, value: Operand| {
        array_destructure(
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
          &mut Mutex::new(HashMap::new()),
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

  pub fn render_runtime_globals(&self, runtime_globals: &RuntimeGlobals) -> String {
    runtime_globals_to_string(runtime_globals, &self.compiler_options)
  }
}

fn runtime_globals_to_string(
  runtime_globals: &RuntimeGlobals,
  _compiler_options: &CompilerOptions,
) -> String {
  let scope_name = "__webpack_require__";
  match *runtime_globals {
    RuntimeGlobals::REQUIRE_SCOPE => format!("{scope_name}.*"),
    RuntimeGlobals::MODULE => "module".to_string(),
    RuntimeGlobals::MODULE_ID => "module.id".to_string(),
    RuntimeGlobals::MODULE_LOADED => "module.loaded".to_string(),
    RuntimeGlobals::REQUIRE => scope_name.to_string(),
    RuntimeGlobals::MODULE_CACHE => format!("{scope_name}.c"),
    RuntimeGlobals::ENSURE_CHUNK => format!("{scope_name}.e"),
    RuntimeGlobals::ENSURE_CHUNK_HANDLERS => format!("{scope_name}.f"),
    RuntimeGlobals::PUBLIC_PATH => format!("{scope_name}.p"),
    RuntimeGlobals::GET_CHUNK_SCRIPT_FILENAME => format!("{scope_name}.u"),
    RuntimeGlobals::GET_CHUNK_CSS_FILENAME => format!("{scope_name}.k"),
    RuntimeGlobals::LOAD_SCRIPT => format!("{scope_name}.l"),
    RuntimeGlobals::HAS_OWN_PROPERTY => format!("{scope_name}.o"),
    RuntimeGlobals::MODULE_FACTORIES_ADD_ONLY => format!("{scope_name}.m (add only)"),
    RuntimeGlobals::ON_CHUNKS_LOADED => format!("{scope_name}.O"),
    RuntimeGlobals::CHUNK_CALLBACK => "webpackChunk".to_string(),
    RuntimeGlobals::MODULE_FACTORIES => format!("{scope_name}.m"),
    RuntimeGlobals::INTERCEPT_MODULE_EXECUTION => format!("{scope_name}.i"),
    RuntimeGlobals::HMR_DOWNLOAD_MANIFEST => format!("{scope_name}.hmrM"),
    RuntimeGlobals::HMR_DOWNLOAD_UPDATE_HANDLERS => format!("{scope_name}.hmrC"),
    RuntimeGlobals::GET_UPDATE_MANIFEST_FILENAME => format!("{scope_name}.hmrF"),
    RuntimeGlobals::GET_CHUNK_UPDATE_SCRIPT_FILENAME => format!("{scope_name}.hu"),
    RuntimeGlobals::GET_CHUNK_UPDATE_CSS_FILENAME => format!("{scope_name}.hk"),
    RuntimeGlobals::HMR_MODULE_DATA => format!("{scope_name}.hmrD"),
    RuntimeGlobals::HMR_RUNTIME_STATE_PREFIX => format!("{scope_name}.hmrS"),
    RuntimeGlobals::AMD_DEFINE => format!("{scope_name}.amdD"),
    RuntimeGlobals::AMD_OPTIONS => format!("{scope_name}.amdO"),
    RuntimeGlobals::EXTERNAL_INSTALL_CHUNK => format!("{scope_name}.C"),
    RuntimeGlobals::GET_FULL_HASH => format!("{scope_name}.h"),
    RuntimeGlobals::GLOBAL => format!("{scope_name}.g"),
    RuntimeGlobals::RETURN_EXPORTS_FROM_RUNTIME => "return-exports-from-runtime".to_string(),
    RuntimeGlobals::INSTANTIATE_WASM => format!("{scope_name}.v"),
    RuntimeGlobals::ASYNC_MODULE => format!("{scope_name}.a"),
    RuntimeGlobals::ASYNC_MODULE_EXPORT_SYMBOL => format!("{scope_name}.aE"),
    RuntimeGlobals::BASE_URI => format!("{scope_name}.b"),
    RuntimeGlobals::STARTUP_ENTRYPOINT => format!("{scope_name}.X"),
    RuntimeGlobals::STARTUP_CHUNK_DEPENDENCIES => format!("{scope_name}.x (chunk dependencies)"),
    RuntimeGlobals::CREATE_SCRIPT_URL => format!("{scope_name}.tu"),
    RuntimeGlobals::CREATE_SCRIPT => format!("{scope_name}.ts"),
    RuntimeGlobals::GET_TRUSTED_TYPES_POLICY => format!("{scope_name}.tt"),
    RuntimeGlobals::DEFINE_PROPERTY_GETTERS => format!("{scope_name}.d"),
    RuntimeGlobals::ENTRY_MODULE_ID => format!("{scope_name}.s"),
    RuntimeGlobals::STARTUP_NO_DEFAULT => format!("{scope_name}.x (no default handler)"),
    RuntimeGlobals::ENSURE_CHUNK_INCLUDE_ENTRIES => format!("{scope_name}.f (include entries)"),
    RuntimeGlobals::STARTUP => format!("{scope_name}.x"),
    RuntimeGlobals::MAKE_NAMESPACE_OBJECT => format!("{scope_name}.r"),
    RuntimeGlobals::MAKE_DEFERRED_NAMESPACE_OBJECT => format!("{scope_name}.z"),
    RuntimeGlobals::MAKE_DEFERRED_NAMESPACE_OBJECT_SYMBOL => format!("{scope_name}.zS"),
    RuntimeGlobals::EXPORTS => "__webpack_exports__".to_string(),
    RuntimeGlobals::COMPAT_GET_DEFAULT_EXPORT => format!("{scope_name}.n"),
    RuntimeGlobals::CREATE_FAKE_NAMESPACE_OBJECT => format!("{scope_name}.t"),
    RuntimeGlobals::ESM_MODULE_DECORATOR => format!("{scope_name}.hmd"),
    RuntimeGlobals::NODE_MODULE_DECORATOR => format!("{scope_name}.nmd"),
    RuntimeGlobals::SYSTEM_CONTEXT => format!("{scope_name}.y"),
    RuntimeGlobals::THIS_AS_EXPORTS => "top-level-this-exports".to_string(),
    RuntimeGlobals::CURRENT_REMOTE_GET_SCOPE => format!("{scope_name}.R"),
    RuntimeGlobals::SHARE_SCOPE_MAP => format!("{scope_name}.S"),
    RuntimeGlobals::INITIALIZE_SHARING => format!("{scope_name}.I"),
    RuntimeGlobals::SCRIPT_NONCE => format!("{scope_name}.nc"),
    RuntimeGlobals::RELATIVE_URL => format!("{scope_name}.U"),
    RuntimeGlobals::CHUNK_NAME => format!("{scope_name}.cn"),
    RuntimeGlobals::RUNTIME_ID => format!("{scope_name}.j"),
    RuntimeGlobals::PREFETCH_CHUNK => format!("{scope_name}.E"),
    RuntimeGlobals::PREFETCH_CHUNK_HANDLERS => format!("{scope_name}.F"),
    RuntimeGlobals::PRELOAD_CHUNK => format!("{scope_name}.G"),
    RuntimeGlobals::PRELOAD_CHUNK_HANDLERS => format!("{scope_name}.H"),
    RuntimeGlobals::UNCAUGHT_ERROR_HANDLER => format!("{scope_name}.oe"),
    // rspack only
    RuntimeGlobals::RSPACK_VERSION => format!("{scope_name}.rv"),
    RuntimeGlobals::RSPACK_UNIQUE_ID => format!("{scope_name}.ruid"),
    RuntimeGlobals::HAS_CSS_MODULES => "has css modules".to_string(),

    RuntimeGlobals::HAS_FETCH_PRIORITY => "has fetch priority".to_string(),
    _ => unreachable!(),
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

fn basic_function(
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

fn returning_function(
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

fn expression_function(
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

fn empty_function(compiler_options: &Arc<CompilerOptions>) -> Operand {
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

fn array_destructure(
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
