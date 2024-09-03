use std::{
  collections::HashMap,
  fmt::Debug,
  sync::{LazyLock, Mutex},
};

use itertools::Itertools;
use rspack_dojang::{dojang::DojangOptions, Context, Dojang, Operand};
use rspack_error::{miette, Result};
use serde_json::{Map, Value};

use crate::{Environment, RuntimeGlobals};

pub struct RuntimeTemplate {
  pub environment: Environment,
  pub dojang: Dojang,
}

static RUNTIME_GLOBALS_VALUE: LazyLock<Map<String, Value>> = LazyLock::new(|| {
  RuntimeGlobals::all()
    .iter_names()
    .map(|(name, value)| (name.to_string(), Value::String(value.to_string())))
    .collect::<Map<String, Value>>()
});

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
    dojang.with_options(DojangOptions {
      escape: "-".to_string(),
      unescape: "=".to_string(),
    });

    if environment.supports_arrow_function() {
      dojang
        .add_function_1("basicFunction".into(), basic_function_arrow)
        .expect("failed to add template function `basicFunction`");
      dojang
        .add_function_2("returningFunction".into(), returning_function_arrow)
        .expect("failed to add template function `returningFunction`");
    } else {
      dojang
        .add_function_1("basicFunction".into(), basic_function)
        .expect("failed to add template function `basicFunction`");
      dojang
        .add_function_2("returningFunction".into(), returning_function)
        .expect("failed to add template function `returningFunction`");
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

  pub fn render(
    &self,
    key: &str,
    params: Option<HashMap<String, String>>,
  ) -> Result<String, miette::Error> {
    let mut render_params = Value::Object(Map::new());

    if let Some(params) = params {
      for (k, v) in params {
        render_params
          .as_object_mut()
          .unwrap_or_else(|| unreachable!())
          .insert(k, Value::String(v));
      }
    }

    render_params
      .as_object_mut()
      .unwrap_or_else(|| unreachable!())
      .extend(RUNTIME_GLOBALS_VALUE.clone());

    if let Some((executer, file_content)) = self.dojang.templates.get(key) {
      executer
        .render(
          &mut Context::new(render_params),
          &self.dojang.templates,
          &self.dojang.functions,
          file_content,
          &mut Mutex::new(HashMap::new()),
        )
        .map_err(|err| {
          miette::Error::msg(format!(
            "Runtime module: failed to render template from: {err}"
          ))
        })
    } else {
      Err(miette::Error::msg(format!(
        "Runtime module: Template {} is not found",
        key
      )))
    }
  }
}

fn to_string(val: &Operand) -> String {
  match val {
    Operand::Value(val) => val.as_str().unwrap_or_default().to_string(),
    _ => String::default(),
  }
}

fn join_to_string(val: &Operand, sep: &str) -> String {
  match val {
    Operand::Array(items) => items.iter().map(to_string).join(sep),
    _ => to_string(val),
  }
}

fn basic_function_arrow(args: Operand) -> Operand {
  Operand::Value(Value::from(format!(
    r#"({}) => "#,
    join_to_string(&args, ", ")
  )))
}

fn basic_function(args: Operand) -> Operand {
  Operand::Value(Value::from(format!(
    r#"function({}) "#,
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
          format!("var {} = {}[{}];", item_name, value_name, idx)
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
          format!("var {} = {}[{}];", item_name, value_name, idx)
        }
      })
      .join("\n"),
    _ => String::default(),
  };
  Operand::Value(Value::from(items))
}
