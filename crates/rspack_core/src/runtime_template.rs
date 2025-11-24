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

use crate::{CompilerOptions, RuntimeGlobals, runtime_globals::runtime_globals_to_string};

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
