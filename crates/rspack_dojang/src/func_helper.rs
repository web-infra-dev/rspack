// List of traits needed for wrapping regular functions to FunctionContainer.

use serde_json::Value;

use crate::expr::*;

impl From<Operand> for bool {
  fn from(op: Operand) -> Self {
    if let Operand::Value(val) = op
      && val.is_boolean()
    {
      return val.as_bool().unwrap();
    }
    false
  }
}

impl From<bool> for Operand {
  fn from(b: bool) -> Self {
    Operand::Value(Value::from(b))
  }
}

impl From<Operand> for i64 {
  fn from(op: Operand) -> Self {
    if let Operand::Value(val) = op
      && val.is_i64()
    {
      return val.as_i64().unwrap();
    }
    0
  }
}

impl From<i64> for Operand {
  fn from(n: i64) -> Self {
    Operand::Value(Value::from(n))
  }
}

impl From<Operand> for f64 {
  fn from(op: Operand) -> Self {
    if let Operand::Value(val) = op
      && val.is_f64()
    {
      return val.as_f64().unwrap();
    }
    0.
  }
}

impl From<f64> for Operand {
  fn from(n: f64) -> Self {
    Operand::Value(Value::from(n))
  }
}

impl From<Operand> for String {
  fn from(op: Operand) -> Self {
    if let Operand::Value(val) = op
      && val.is_string()
    {
      return val.as_str().unwrap().to_string();
    }
    String::new()
  }
}

impl From<String> for Operand {
  fn from(s: String) -> Self {
    Operand::Value(Value::from(s))
  }
}

impl From<Operand> for Value {
  fn from(op: Operand) -> Self {
    if let Operand::Value(val) = op {
      val
    } else {
      Value::Bool(false)
    }
  }
}

impl From<Value> for Operand {
  fn from(val: Value) -> Self {
    Operand::Value(val)
  }
}
