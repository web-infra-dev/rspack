use std::{
  collections::HashMap,
  fmt::Debug,
  sync::{LazyLock, Mutex},
};

use cow_utils::CowUtils;
use itertools::Itertools;
use regex::{Captures, Regex};
use rspack_dojang::{Context, Dojang, Operand};
use rspack_error::{Error, Result, ToStringResultToRspackResultExt, error};
use serde_json::{Map, Value};

use crate::{Environment, RuntimeGlobals};

pub struct RuntimeTemplate {
  environment: Environment,
  dojang: Dojang,
}

static RUNTIME_GLOBALS_VALUE: LazyLock<Map<String, Value>> = LazyLock::new(|| {
  RuntimeGlobals::all()
    .iter_names()
    .map(|(name, value)| (name.to_string(), Value::String(value.to_string())))
    .collect::<Map<String, Value>>()
});

static RUNTIME_GLOBALS_PATTERN: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"\$\$RUNTIME_GLOBAL_(.*?)\$\$").expect("failed to create regex"));

fn replace_runtime_globals(template: String) -> String {
  RUNTIME_GLOBALS_PATTERN
    .replace_all(&template, |caps: &Captures| {
      let name = caps.get(1).expect("name should be a string").as_str();
      RUNTIME_GLOBALS_VALUE
        .get(name)
        .map(|value| match value {
          Value::String(value) => value.to_string(),
          _ => unreachable!(),
        })
        .expect("value should be a string")
    })
    .to_string()
}

impl Debug for RuntimeTemplate {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("runtime_template")
      .field("environment", &self.environment)
      .finish()
  }
}

impl RuntimeTemplate {
  pub fn new(environment: Environment) -> Self {
    let mut dojang = Dojang::new();

    if environment.supports_arrow_function() {
      dojang
        .add_function_1("basicFunction".into(), basic_function_arrow)
        .expect("failed to add template function `basicFunction`");
      dojang
        .add_function_2("returningFunction".into(), returning_function_arrow)
        .expect("failed to add template function `returningFunction`");
      dojang
        .add_function_2("expressionFunction".into(), expression_function_arrow)
        .expect("failed to add template function `expressionFunction`");
      dojang
        .add_function_0("emptyFunction".into(), empty_function_arrow)
        .expect("failed to add template function `emptyFunction`");
    } else {
      dojang
        .add_function_1("basicFunction".into(), basic_function)
        .expect("failed to add template function `basicFunction`");
      dojang
        .add_function_2("returningFunction".into(), returning_function)
        .expect("failed to add template function `returningFunction`");
      dojang
        .add_function_2("expressionFunction".into(), expression_function)
        .expect("failed to add template function `expressionFunction`");
      dojang
        .add_function_0("emptyFunction".into(), empty_function)
        .expect("failed to add template function `emptyFunction`");
    }

    if environment.supports_destructuring() {
      dojang
        .add_function_2("destructureArray".into(), array_destructure)
        .expect("failed to add template function `destructureArray`");
    } else {
      dojang
        .add_function_2("destructureArray".into(), array_variable)
        .expect("failed to add template function `destructureArray`");
    }
    Self {
      environment,
      dojang,
    }
  }

  pub fn add_templates(&mut self, templates: Vec<(String, String)>) {
    for (key, template) in templates {
      if !self.dojang.templates.contains_key(&key) {
        self
          .dojang
          .add_with_option(key.clone(), template)
          .unwrap_or_else(|_| panic!("failed to add template {key}"));
      }
    }
  }

  pub fn render(&self, key: &str, params: Option<serde_json::Value>) -> Result<String, Error> {
    let mut render_params = Value::Object(RUNTIME_GLOBALS_VALUE.clone());

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

    if let Some((executer, file_content)) = self.dojang.templates.get(key) {
      executer
        .render(
          &mut Context::new(render_params),
          &self.dojang.templates,
          &self.dojang.functions,
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
    if self.environment.supports_arrow_function() {
      format!("({args}) => ({return_value})")
    } else {
      format!("function({args}) {{ return {return_value}; }}")
    }
  }

  pub fn basic_function(&self, args: &str, body: &str) -> String {
    if self.environment.supports_arrow_function() {
      format!("({args}) => {{\n {body} \n}}")
    } else {
      format!("function({args}) {{\n {body} \n}}")
    }
  }
}

fn to_string(val: &Operand) -> String {
  replace_runtime_globals(match val {
    Operand::Value(val) => val.as_str().unwrap_or_default().to_string(),
    _ => String::default(),
  })
}

fn join_to_string(val: &Operand, sep: &str) -> String {
  replace_runtime_globals(match val {
    Operand::Array(items) => items.iter().map(to_string).join(sep),
    _ => to_string(val),
  })
}

fn basic_function_arrow(args: Operand) -> Operand {
  Operand::Value(Value::from(format!(
    r#"({}) =>"#,
    join_to_string(&args, ", ")
  )))
}

fn basic_function(args: Operand) -> Operand {
  Operand::Value(Value::from(format!(
    r#"function({})"#,
    join_to_string(&args, ", ")
  )))
}

fn returning_function_arrow(return_value: Operand, args: Operand) -> Operand {
  Operand::Value(Value::from(format!(
    "({}) => ({})",
    join_to_string(&args, ", "),
    to_string(&return_value)
  )))
}

fn expression_function(expression: Operand, args: Operand) -> Operand {
  Operand::Value(Value::from(format!(
    "function({}) {{ {}; }}",
    join_to_string(&args, ", "),
    to_string(&expression)
  )))
}

fn expression_function_arrow(expression: Operand, args: Operand) -> Operand {
  Operand::Value(Value::from(format!(
    "({}) => ({})",
    join_to_string(&args, ", "),
    to_string(&expression)
  )))
}

fn returning_function(return_value: Operand, args: Operand) -> Operand {
  Operand::Value(Value::from(format!(
    "function({}) {{ return {}; }}",
    join_to_string(&args, ", "),
    to_string(&return_value)
  )))
}

fn array_destructure(items: Operand, value: Operand) -> Operand {
  Operand::Value(Value::from(format!(
    "var [{}] = {};",
    join_to_string(&items, ", "),
    to_string(&value)
  )))
}

fn array_variable(items: Operand, value: Operand) -> Operand {
  let value_name = to_string(&value);
  let items = match items {
    Operand::Array(items) => items
      .iter()
      .enumerate()
      .map(|(idx, item)| {
        let item_name = to_string(item);
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

fn empty_function() -> Operand {
  Operand::Value(Value::from("function() {}"))
}

fn empty_function_arrow() -> Operand {
  Operand::Value(Value::from("function() {}"))
}
