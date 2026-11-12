use serde_json::Value;

use crate::expr::{Operand, convert_operand_to_value};

pub fn val_length(op: Operand) -> Operand {
  match op {
    Operand::Value(val) => {
      if val.is_string() {
        Operand::Value(Value::from(val.as_str().unwrap().len()))
      } else if val.is_array() {
        Operand::Value(Value::from(val.as_array().unwrap().len()))
      } else {
        Operand::Value(Value::from(0))
      }
    }
    Operand::Array(arr) => Operand::Value(Value::from(arr.len())),
    _ => Operand::Value(Value::from(0)),
  }
}

pub fn val_range(op: Operand) -> Operand {
  if let Operand::Value(val) = op
    && val.is_i64()
  {
    return Operand::Value(Value::from(
      (0..val.as_i64().unwrap()).collect::<Vec<i64>>(),
    ));
  }

  Operand::Value(Value::from(Vec::<i64>::new()))
}

pub fn val_stringify(op: Operand) -> Operand {
  Operand::Value(Value::from(
    serde_json::to_string(&convert_operand_to_value(op)).unwrap(),
  ))
}
