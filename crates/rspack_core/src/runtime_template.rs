use std::fmt::Debug;

use itertools::Itertools;
use rspack_dojang::{dojang::DojangOptions, Dojang, Operand};
use rspack_error::{miette, Result};
use serde_json::{Map, Value};

use crate::{Environment, RuntimeGlobals};

pub struct RuntimeTemplate {
  pub environment: Environment,
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
    Self { environment }
  }

  pub fn render(&self, content: String, params: Option<Value>) -> Result<String, miette::Error> {
    let mut dj = Dojang::new();
    dj.with_options(DojangOptions {
      escape: "-".to_string(),
      unescape: "=".to_string(),
    });

    let mut render_params = Value::Object(Map::new());

    if let Some(params) = params {
      merge_json(&mut render_params, params);
    }

    for (name, runtime) in RuntimeGlobals::all().iter_names() {
      render_params
        .as_object_mut()
        .unwrap_or_else(|| panic!("merged json is not an object"))
        .entry(name)
        .or_insert(Value::String(runtime.to_string()));
    }

    if self.environment.supports_arrow_function() {
      dj.add_function_1("basicFunction".into(), basic_function_arrow)
        .expect("failed to add template function `basicFunction`");
      dj.add_function_2("returningFunction".into(), returning_function_arrow)
        .expect("failed to add template function `returningFunction`");
    } else {
      dj.add_function_1("basicFunction".into(), basic_function)
        .expect("failed to add template function `basicFunction`");
      dj.add_function_2("returningFunction".into(), returning_function)
        .expect("failed to add template function `returningFunction`");
    }

    if self.environment.supports_destructuring() {
      dj.add_function_2("destructureArray".into(), array_destructure)
        .expect("failed to add template function `destructureArray`");
    } else {
      dj.add_function_2("destructureArray".into(), array_variable)
        .expect("failed to add template function `destructureArray`");
    }

    dj.add_with_option("runtime_module".to_string(), content)
      .expect("failed to add template");
    dj.render("runtime_module", render_params).map_err(|err| {
      miette::Error::msg(format!(
        "Runtime module: failed to render template from: {err}"
      ))
    })
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

fn merge_json(a: &mut Value, b: Value) {
  match (a, b) {
    (a @ &mut Value::Object(_), Value::Object(b)) => {
      let a = a
        .as_object_mut()
        .unwrap_or_else(|| panic!("merged json is not an object"));
      for (k, v) in b {
        merge_json(a.entry(k).or_insert(Value::Null), v);
      }
    }
    (a, b) => *a = b,
  }
}
