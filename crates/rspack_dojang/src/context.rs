use std::{collections::HashMap, fmt, sync::Mutex};

use serde_json::{Map, Number, Value};

use crate::{eval::*, exec::*, expr::*};

#[derive(Debug)]
pub struct Context {
  pub context: Value,
}

pub enum FunctionContainer {
  F0(Box<dyn Fn() -> Operand + Send + Sync>),
  F1(Box<dyn Fn(Operand) -> Operand + Send + Sync>),
  F2(Box<dyn Fn(Operand, Operand) -> Operand + Send + Sync>),
  F3(Box<dyn Fn(Operand, Operand, Operand) -> Operand + Send + Sync>),
  F4(Box<dyn Fn(Operand, Operand, Operand, Operand) -> Operand + Send + Sync>),
}

impl fmt::Debug for FunctionContainer {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mut f = f.debug_struct("FunctionContainer");
    match self {
      FunctionContainer::F0(_) => f.field("F0", &0),
      FunctionContainer::F1(_) => f.field("F1", &1),
      FunctionContainer::F2(_) => f.field("F2", &2),
      FunctionContainer::F3(_) => f.field("F3", &3),
      FunctionContainer::F4(_) => f.field("F4", &4),
    };

    f.finish()
  }
}

impl FunctionContainer {
  pub fn param_num(&self) -> usize {
    match self {
      FunctionContainer::F0(_) => 0,
      FunctionContainer::F1(_) => 1,
      FunctionContainer::F2(_) => 2,
      FunctionContainer::F3(_) => 3,
      FunctionContainer::F4(_) => 4,
    }
  }
}

impl Context {
  pub fn new(context: Value) -> Self {
    Context { context }
  }

  fn get_value(&self, names: &Vec<&str>) -> Result<&Value, String> {
    if names.is_empty() {
      return Err(format!("Mapping not exist : {names:?}"));
    }

    let mut value;
    match self.context.get(names.first().unwrap()) {
      Some(v) => {
        value = v;
      }
      _ => {
        return Err(format!(
          "ReferenceError: {} is not defined",
          names.first().unwrap()
        ));
      }
    }

    for n in names.iter().skip(1) {
      if value.is_array() {
        match n.parse::<usize>() {
          Ok(index) => {
            match value.as_array().unwrap().get(index) {
              Some(elem) => value = elem,
              None => {
                return Err(format!(
                  "Element at the specified index of the array does not exist : array {value:?}, index : {n:?}"
                ));
              }
            };
          }
          _ => {
            return Err(format!(
              "Specified non-number parameter for an array : {value:?}, index : {n:?}"
            ));
          }
        }
      } else {
        match value.get(n) {
          Some(v) => {
            value = v;
          }
          _ => {
            return Err(format!(
              "ReferenceError: {} is not defined",
              names.join(".")
            ));
          }
        }
      }
    }

    Ok(value)
  }

  fn set_value(&mut self, names: &Vec<&str>, provided: &Value) -> Result<(), String> {
    if names.is_empty() {
      return Err(format!("Mapping not exist : {names:?}"));
    }

    let mut value: &mut Value;
    match self.context.get_mut(names.first().unwrap()) {
      Some(v) => {
        value = v;
      }
      _ => {
        if names.len() > 1 {
          return Err(format!(
            "Local variable should not use dot operator. {:?}",
            names.first().unwrap()
          ));
        }

        if !self.context.is_object() {
          self.context = Value::Object(Map::new());
        }

        self
          .context
          .as_object_mut()
          .unwrap()
          .insert(names.first().unwrap().to_string(), provided.clone());

        return Ok(());
      }
    }

    for n in names.iter().skip(1) {
      match value.get_mut(n) {
        Some(v) => {
          value = v;
        }
        _ => return Err(format!("Mapping not exist : {names:?} at {n}")),
      }
    }

    *value = provided.clone();
    Ok(())
  }
}

pub trait ComputeExpr {
  fn run(
    &self,
    context: &mut Context,
    templates: &HashMap<String, (Executer, String)>,
    functions: &HashMap<String, FunctionContainer>,
    includes: &mut Mutex<HashMap<String, String>>,
  ) -> Result<Operand, String>;

  fn run_for_range(
    &self,
    context: &mut Context,
    templates: &HashMap<String, (Executer, String)>,
    functions: &HashMap<String, FunctionContainer>,
    includes: &mut Mutex<HashMap<String, String>>,
  ) -> Result<Operand, String>;

  fn run_for_loop(
    &self,
    context: &mut Context,
    range: &Operand,
    for_index: usize,
  ) -> Result<bool, String>;
}

impl ComputeExpr for Eval {
  fn run(
    &self,
    context: &mut Context,
    templates: &HashMap<String, (Executer, String)>,
    functions: &HashMap<String, FunctionContainer>,
    includes: &mut Mutex<HashMap<String, String>>,
  ) -> Result<Operand, String> {
    let mut operands: Vec<Operand> = Vec::new();

    for expr in self.expr.iter().rev() {
      match expr {
        Expr::Op(op) => match op {
          Op::Operand(operand) => {
            operands.push(operand.clone());
          }
          optr => {
            let num_operands = operator_num_operands(optr);
            if operands.len() < num_operands {
              return Err(format!(
                "Number of operands for {optr:?} is less than {num_operands}"
              ));
            }

            if num_operands == 1 {
              let op = operands.pop().unwrap();
              match optr.compute_unary(context, op) {
                Ok(operand) => {
                  operands.push(operand);
                }
                Err(e) => return Err(e),
              }
            } else if num_operands == 2 {
              // Since we are iterating from back, left is the top most operand.
              let left = operands.pop().unwrap();
              let right = operands.pop().unwrap();

              match optr.compute_binary(context, left, right) {
                Ok(operand) => {
                  operands.push(operand);
                }
                Err(e) => return Err(e),
              }
            }
          }
        },
        Expr::Function(function) => match function.is_accessor {
          false => {
            if let Some(operand) =
              handle_predefined_functions(function, context, templates, functions, includes)?
            {
              operands.push(operand);
              continue;
            }

            let function_to_run = match functions.get(&function.name) {
              Some(f) => f,
              None => {
                return Err(format!(
                  "Function {:?} is not registered; Registered : {:?}",
                  function.name, functions
                ));
              }
            };

            let params = &function.params;
            if params.len() != function_to_run.param_num() {
              return Err(format!(
                "# of function params mistmatch! {} takes {} params but provided {} params",
                function.name,
                function_to_run.param_num(),
                params.len()
              ));
            }

            let mut evals = Vec::new();

            for param in params {
              evals.push(param.run(context, templates, functions, includes)?);
            }

            let return_value = match function_to_run {
              FunctionContainer::F0(f) => f(),
              FunctionContainer::F1(f) => f(evals.pop().unwrap()),
              FunctionContainer::F2(f) => {
                let p2 = evals.pop().unwrap();
                let p1 = evals.pop().unwrap();

                f(p1, p2)
              }
              FunctionContainer::F3(f) => {
                let p3 = evals.pop().unwrap();
                let p2 = evals.pop().unwrap();
                let p1 = evals.pop().unwrap();

                f(p1, p2, p3)
              }
              FunctionContainer::F4(f) => {
                let p4 = evals.pop().unwrap();
                let p3 = evals.pop().unwrap();
                let p2 = evals.pop().unwrap();
                let p1 = evals.pop().unwrap();

                f(p1, p2, p3, p4)
              }
            };

            operands.push(return_value);
          }
          true => {
            let params = &function.params;

            let mut obj_name = vec![function.name.clone()];
            for param in params {
              obj_name.push(param.run(context, templates, functions, includes)?.to_str())
            }

            operands.push(convert_value_to_operand(
              context
                .get_value(&obj_name.iter().map(String::as_str).collect())?
                .clone(),
            ))
          }
        },
      }
    }

    if operands.len() != 1 {
      return Err(format!(
        "# of operands after computing is not zero. {operands:?}"
      ));
    }

    match operands.pop().unwrap() {
      Operand::Object(obj) => Ok(convert_value_to_operand(
        context.get_value(&obj.name.split(".").collect())?.clone(),
      )),
      operand => Ok(operand),
    }
  }

  fn run_for_range(
    &self,
    context: &mut Context,
    templates: &HashMap<String, (Executer, String)>,
    functions: &HashMap<String, FunctionContainer>,
    includes: &mut Mutex<HashMap<String, String>>,
  ) -> Result<Operand, String> {
    if self.expr.len() != 3 {
      return Err(format!(
        "For loop must use 'for a in b' format, yours use {:?}",
        self.expr
      ));
    }

    match self.expr.first().unwrap() {
      Expr::Op(Op::Operand(Operand::Object(_))) => {}
      _ => {
        return Err(format!(
          "Range declaration in for loop must be an object; Yours : {:?}",
          self.expr
        ));
      }
    }

    if self.expr[1] != Expr::Op(Op::Operand(Operand::Keyword(Keyword::In))) {
      return Err(format!(
        "'in' is missing in your for-loop. Yours : {:?}",
        self.expr
      ));
    }

    let range = Eval {
      expr: vec![self.expr[2].clone()],
    };

    range.run(context, templates, functions, includes)
  }

  // Returns true if the for loop's condition is true.
  fn run_for_loop(
    &self,
    context: &mut Context,
    range: &Operand,
    for_index: usize,
  ) -> Result<bool, String> {
    if self.expr.len() != 3 {
      return Err(format!(
        "For loop must use 'for a in b' format, yours use {:?}",
        self.expr
      ));
    }

    let object_name = match self.expr.first().unwrap() {
      Expr::Op(Op::Operand(Operand::Object(object))) => &object.name,
      _ => {
        return Err(format!(
          "Range declaration in for loop must be an object; Yours : {:?}",
          self.expr
        ));
      }
    };

    match get_nth_element_from_operand(range, for_index) {
      Some(element) => {
        context.set_value(&object_name.split(".").collect(), &element)?;
        context.set_value(
          &vec!["index"],
          &Value::Number(Number::from(for_index as u64)),
        )?;
        Ok(true)
      }
      _ => Ok(false),
    }
  }
}

fn handle_predefined_functions(
  f: &Function,
  context: &mut Context,
  templates: &HashMap<String, (Executer, String)>,
  functions: &HashMap<String, FunctionContainer>,
  includes: &mut Mutex<HashMap<String, String>>,
) -> Result<Option<Operand>, String> {
  if f.name == "include" {
    if f.params.len() != 1 {
      return Err(format!(
        "Predefined function 'includes' must provide 1 string parameter. You gave : {f:?}"
      ));
    }

    let file_name = f
      .params
      .first()
      .unwrap()
      .run(context, templates, functions, includes)?
      .to_str();
    if let Some(file_data) = includes.get_mut().unwrap().get(&file_name) {
      return Ok(Some(Operand::Value(Value::String(file_data.clone()))));
    } else {
      let file_content = std::fs::read(&file_name).map_err(|e| e.to_string())?;

      return Ok(Some(Operand::Value(Value::String(
        String::from_utf8_lossy(&file_content).to_string(),
      ))));
    }
  } else if f.name == "include_template" {
    if f.params.len() != 1 {
      return Err(format!(
        "Predefined function 'includes' must provide 1 string parameter. You gave : {f:?}"
      ));
    }

    let file_name = f
      .params
      .first()
      .unwrap()
      .run(context, templates, functions, includes)?
      .to_str();

    if let Some((executer, template)) = templates.get(&file_name) {
      return Ok(Some(Operand::Value(Value::String(
        executer.render(context, templates, functions, template, includes)?,
      ))));
    } else {
      return Err(format!(
        "Predefined function 'include_template' must only use the pre-registered template. You gave : {file_name}"
      ));
    }
  }

  Ok(None)
}

pub trait Convert {
  fn is_true(&self) -> bool;
  fn to_str(&self) -> String;
}

impl Convert for Operand {
  fn is_true(&self) -> bool {
    match &self {
      Operand::Value(v) => value_to_boolean(v),
      Operand::Array(arr) => !arr.is_empty(),
      _ => {
        panic!("Unconvertible object {:?}", &self)
      }
    }
  }

  fn to_str(&self) -> String {
    match &self {
      Operand::Value(v) => match v {
        Value::String(s) => s.clone(),
        v => v.to_string(),
      },
      Operand::Array(arr) => format!("{arr:?}"),
      _ => {
        panic!("Unconvertible object {:?}", &self)
      }
    }
  }
}

trait ComputeOp {
  fn compute_binary(
    &self,
    context: &mut Context,
    left: Operand,
    right: Operand,
  ) -> Result<Operand, String>;

  fn compute_unary(&self, context: &Context, op: Operand) -> Result<Operand, String>;
}

impl ComputeOp for Op {
  fn compute_binary(
    &self,
    context: &mut Context,
    left: Operand,
    right: Operand,
  ) -> Result<Operand, String> {
    match self {
      Op::And => return compute_binary(context, left, right, compute_and),
      Op::Or => return compute_binary(context, left, right, compute_or),
      Op::Equal => return compute_binary(context, left, right, compute_eq),
      Op::NotEqual => return compute_binary(context, left, right, compute_neq),
      Op::Greater => return compute_binary(context, left, right, compute_greater),
      Op::GreaterEq => return compute_binary(context, left, right, compute_greater_eq),
      Op::Less => return compute_binary(context, left, right, compute_less),
      Op::LessEq => return compute_binary(context, left, right, compute_less_eq),
      Op::Assign => return compute_simple_assign(context, left, right),
      Op::Plus => return compute_binary(context, left, right, compute_add),
      Op::Minus => return compute_binary(context, left, right, compute_minus),
      Op::Multiply => return compute_binary(context, left, right, compute_multiply),
      Op::Divide => return compute_binary(context, left, right, compute_divide),
      _ => {}
    }

    panic!("Binary {self:?} is not implemented");
  }

  fn compute_unary(&self, context: &Context, op: Operand) -> Result<Operand, String> {
    if self == &Op::Not {
      return compute_unary(context, op, compute_not);
    }

    panic!("Unary {self:?} is not implemented");
  }
}

fn compute_unary<ComputeFunc>(
  context: &Context,
  op: Operand,
  compute_func: ComputeFunc,
) -> Result<Operand, String>
where
  ComputeFunc: Fn(&Value) -> Result<Value, String>,
{
  Ok(convert_value_to_operand(compute_func(
    get_value_from_operand(context, &op)?,
  )?))
}

fn get_value_from_operand<'a>(
  context: &'a Context,
  operand: &'a Operand,
) -> Result<&'a Value, String> {
  if let Operand::Object(obj) = operand {
    match context.get_value(&obj.name.split(".").collect()) {
      Ok(v) => Ok(v),
      Err(e) => Err(e),
    }
  } else if let Operand::Value(v) = operand {
    Ok(v)
  } else {
    Err(format!("Unable to extract value from {operand:?}"))
  }
}

fn compute_binary<ComputeFunc>(
  context: &Context,
  left: Operand,
  right: Operand,
  compute_func: ComputeFunc,
) -> Result<Operand, String>
where
  ComputeFunc: Fn(&Value, &Value) -> Result<Value, String>,
{
  let left_value = get_value_from_operand(context, &left)?;
  let right_value = get_value_from_operand(context, &right)?;

  Ok(convert_value_to_operand(compute_func(
    left_value,
    right_value,
  )?))
}

fn compute_simple_assign(
  context: &mut Context,
  left: Operand,
  right: Operand,
) -> Result<Operand, String> {
  if let Operand::Object(ref object) = left {
    match right {
      Operand::Object(right_obj) => {
        let val = context
          .get_value(&right_obj.name.split(".").collect())?
          .clone();
        context.set_value(&object.name.split(".").collect(), &val)?;
      }
      _ => {
        context.set_value(
          &object.name.split(".").collect(),
          &convert_operand_to_value(right),
        )?;
      }
    }
    Ok(left)
  } else {
    Err(format!("Cannot assign to non-object {left:?}"))
  }
}

/*
fn compute_binary_assign<ComputeFunc>(
    context: &mut Context,
    left: Operand,
    right: Operand,
    compute_func: ComputeFunc,
) -> Result<Operand, String>
where
    ComputeFunc: Fn(Operand, Operand) -> Result<Operand, String>,
{
    if let Operand::Object(ref object) = left {
        match right {
            Operand::Object(right_obj) => {
                context.set_value(
                    &object.name,
                    compute_func(
                        convert_value_to_operand(context.get_value(&object.name)?),
                        convert_value_to_operand(context.get_value(&right_obj.name)?),
                    )?,
                )?;
            }
            _ => {
                context.set_value(
                    &object.name,
                    compute_func(
                        convert_value_to_operand(context.get_value(&object.name)?),
                        right,
                    )?,
                )?;
            }
        }
        Ok(left)
    } else {
        return Err(format!("Cannot assign to non-object {:?}", left));
    }
}
*/

fn value_to_boolean(v: &Value) -> bool {
  match v {
    Value::Bool(v) => *v,
    Value::String(s) => !s.is_empty(),
    Value::Number(n) => {
      if n.is_i64() {
        n.as_i64().unwrap() != 0
      } else if n.is_f64() {
        n.as_f64().unwrap() != 0.
      } else {
        n.as_u64().unwrap() != 0
      }
    }
    Value::Array(arr) => !arr.is_empty(),
    Value::Object(_) => true,
    Value::Null => false,
  }
}

fn compute_and(left: &Value, right: &Value) -> Result<Value, String> {
  Ok(Value::from(
    value_to_boolean(left) && value_to_boolean(right),
  ))
}

fn compute_or(left: &Value, right: &Value) -> Result<Value, String> {
  Ok(Value::from(
    value_to_boolean(left) || value_to_boolean(right),
  ))
}

fn compute_greater(left: &Value, right: &Value) -> Result<Value, String> {
  match (&left, &right) {
    (Value::String(l), Value::String(r)) => {
      return Ok(Value::from((l > r) as i64));
    }
    (Value::Number(l), Value::Number(r)) => {
      if l.is_i64() && r.is_i64() {
        return Ok(Value::from(l.as_i64().unwrap() > r.as_i64().unwrap()));
      } else if l.is_f64() && r.is_f64() {
        return Ok(Value::from(l.as_f64().unwrap() > r.as_f64().unwrap()));
      }
    }
    _ => {}
  };

  Err(format!("Invalid operation '>' between {left:?} {right:?}"))
}

fn compute_greater_eq(left: &Value, right: &Value) -> Result<Value, String> {
  match (&left, &right) {
    (Value::String(l), Value::String(r)) => {
      return Ok(Value::from(l >= r));
    }
    (Value::Number(l), Value::Number(r)) => {
      if l.is_i64() && r.is_i64() {
        return Ok(Value::from(l.as_i64().unwrap() >= r.as_i64().unwrap()));
      } else if l.is_f64() && r.is_f64() {
        return Ok(Value::from(l.as_f64().unwrap() >= r.as_f64().unwrap()));
      }
    }
    _ => {}
  };

  Err(format!("Invalid operation '>=' between {left:?} {right:?}"))
}

fn compute_less(left: &Value, right: &Value) -> Result<Value, String> {
  match (&left, &right) {
    (Value::String(l), Value::String(r)) => {
      return Ok(Value::from(l < r));
    }
    (Value::Number(l), Value::Number(r)) => {
      if l.is_i64() && r.is_i64() {
        return Ok(Value::from(l.as_i64().unwrap() < r.as_i64().unwrap()));
      } else if l.is_f64() && r.is_f64() {
        return Ok(Value::from(l.as_f64().unwrap() < r.as_f64().unwrap()));
      }
    }
    _ => {}
  };

  Err(format!("Invalid operation '<' between {left:?} {right:?}"))
}

fn compute_less_eq(left: &Value, right: &Value) -> Result<Value, String> {
  match (&left, &right) {
    (Value::String(l), Value::String(r)) => {
      return Ok(Value::from(l <= r));
    }
    (Value::Number(l), Value::Number(r)) => {
      if l.is_i64() && r.is_i64() {
        return Ok(Value::from(l.as_i64().unwrap() <= r.as_i64().unwrap()));
      } else if l.is_f64() && r.is_f64() {
        return Ok(Value::from(l.as_f64().unwrap() <= r.as_f64().unwrap()));
      }
    }
    _ => {}
  };

  Err(format!("Invalid operation '<=' between {left:?} {right:?}"))
}

fn compute_eq(left: &Value, right: &Value) -> Result<Value, String> {
  match (&left, &right) {
    (Value::Bool(l), Value::Bool(r)) => return Ok(Value::from(*l == *r)),
    (Value::String(l), Value::String(r)) => {
      return Ok(Value::from(l == r));
    }
    (Value::Number(l), Value::Number(r)) => {
      if l.is_i64() && r.is_i64() {
        return Ok(Value::from(l.as_i64().unwrap() == r.as_i64().unwrap()));
      } else if l.is_f64() && r.is_f64() {
        return Ok(Value::from(l.as_f64().unwrap() == r.as_f64().unwrap()));
      }
    }
    _ => {}
  };

  Err(format!("Invalid operation '==' between {left:?} {right:?}"))
}

fn compute_neq(left: &Value, right: &Value) -> Result<Value, String> {
  match (&left, &right) {
    (Value::Bool(l), Value::Bool(r)) => return Ok(Value::from(*l != *r)),
    (Value::String(l), Value::String(r)) => {
      return Ok(Value::from(l != r));
    }
    (Value::Number(l), Value::Number(r)) => {
      if l.is_i64() && r.is_i64() {
        return Ok(Value::from(l.as_i64().unwrap() != r.as_i64().unwrap()));
      } else if l.is_f64() && r.is_f64() {
        return Ok(Value::from(l.as_f64().unwrap() != r.as_f64().unwrap()));
      }
    }
    _ => {}
  };

  Err(format!("Invalid operation '!=' between {left:?} {right:?}"))
}

fn compute_add(left: &Value, right: &Value) -> Result<Value, String> {
  match (&left, &right) {
    (Value::String(l), Value::String(r)) => {
      let mut left_clone = l.clone();
      left_clone.push_str(r);
      return Ok(Value::from(left_clone));
    }
    (Value::Number(l), Value::Number(r)) => {
      if l.is_i64() && r.is_i64() {
        return Ok(Value::from(l.as_i64().unwrap() + r.as_i64().unwrap()));
      } else if l.is_f64() && r.is_f64() {
        return Ok(Value::from(l.as_f64().unwrap() + r.as_f64().unwrap()));
      }
    }
    _ => {}
  };

  Err(format!("Invalid operation '+' between {left:?} {right:?}"))
}

fn compute_minus(left: &Value, right: &Value) -> Result<Value, String> {
  if let (Value::Number(l), Value::Number(r)) = (&left, &right) {
    if l.is_i64() && r.is_i64() {
      return Ok(Value::from(l.as_i64().unwrap() - r.as_i64().unwrap()));
    } else if l.is_f64() && r.is_f64() {
      return Ok(Value::from(l.as_f64().unwrap() - r.as_f64().unwrap()));
    }
  };

  Err(format!("Invalid operation '-' between {left:?} {right:?}"))
}

fn compute_multiply(left: &Value, right: &Value) -> Result<Value, String> {
  if let (Value::Number(l), Value::Number(r)) = (&left, &right) {
    if l.is_i64() && r.is_i64() {
      return Ok(Value::from(l.as_i64().unwrap() * r.as_i64().unwrap()));
    } else if l.is_f64() && r.is_f64() {
      return Ok(Value::from(l.as_f64().unwrap() * r.as_f64().unwrap()));
    }
  };

  Err(format!("Invalid operation '*' between {left:?} {right:?}"))
}

fn compute_divide(left: &Value, right: &Value) -> Result<Value, String> {
  if let (Value::Number(l), Value::Number(r)) = (&left, &right) {
    if l.is_i64() && r.is_i64() {
      return Ok(Value::from(l.as_i64().unwrap() / r.as_i64().unwrap()));
    } else if l.is_f64() && r.is_f64() {
      return Ok(Value::from(l.as_f64().unwrap() / r.as_f64().unwrap()));
    }
  };

  Err(format!("Invalid operation '/' between {left:?} {right:?}"))
}

fn compute_not(operand: &Value) -> Result<Value, String> {
  Ok(Value::from(!value_to_boolean(operand)))
}

#[cfg(test)]
fn get_expr(s: &str) -> Tokens {
  let mut res = Parser::parse(s).unwrap();
  match res.parse_tree.pop().unwrap() {
    Action::Do(expr) => expr,
    _ => panic!("No expr found"),
  }
}

#[test]
fn compute_and_test() {
  let context_json = r#"{"a": 1, "b":0, "c":"abc", "d":"", "e": "def"}"#;
  let context_value: Value = serde_json::from_str(context_json).unwrap();
  let mut context = Context::new(context_value);
  let mut includes = Mutex::new(HashMap::new());

  {
    let eval = Eval::new(get_expr(r"<% a && b %>")).unwrap();
    let result = eval.run(
      &mut context,
      &HashMap::new(),
      &HashMap::new(),
      &mut includes,
    );

    assert_eq!(result.unwrap(), Operand::Value(Value::from(false)));
  }
  {
    let eval = Eval::new(get_expr(r"<% c && d %>")).unwrap();
    let result = eval.run(
      &mut context,
      &HashMap::new(),
      &HashMap::new(),
      &mut includes,
    );

    assert_eq!(result.unwrap(), Operand::Value(Value::from(false)));
  }
  {
    let eval = Eval::new(get_expr(r"<% c && e %>")).unwrap();
    let result = eval.run(
      &mut context,
      &HashMap::new(),
      &HashMap::new(),
      &mut includes,
    );

    assert_eq!(result.unwrap(), Operand::Value(Value::from(true)));
  }
}

#[test]
fn compute_or_test() {
  let context_json = r#"{"a": 1, "b":0, "c":"abc", "d":"", "e": "def"}"#;
  let context_value: Value = serde_json::from_str(context_json).unwrap();
  let mut context = Context::new(context_value);
  let mut includes = Mutex::new(HashMap::new());

  {
    let eval = Eval::new(get_expr(r"<% (a && b) || (c && e) %>")).unwrap();
    let result = eval.run(
      &mut context,
      &HashMap::new(),
      &HashMap::new(),
      &mut includes,
    );

    assert_eq!(result.unwrap(), Operand::Value(Value::from(true)))
  }
  {
    let eval = Eval::new(get_expr(r"<% (a && b) || (c && d) %>")).unwrap();
    let result = eval.run(
      &mut context,
      &HashMap::new(),
      &HashMap::new(),
      &mut includes,
    );

    assert_eq!(result.unwrap(), Operand::Value(Value::from(false)));
  }
  {
    let eval = Eval::new(get_expr(r"<% c || e %>")).unwrap();
    let result = eval.run(
      &mut context,
      &HashMap::new(),
      &HashMap::new(),
      &mut includes,
    );

    assert_eq!(result.unwrap(), Operand::Value(Value::from(true)));
  }
}

#[test]
fn compute_complex() {
  let context_json = r#"{"a": 1, "b":0, "c":"abc", "d":"", "e": "def"}"#;
  let context_value: Value = serde_json::from_str(context_json).unwrap();
  let mut context = Context::new(context_value);
  let mut includes = Mutex::new(HashMap::new());

  {
    let eval = Eval::new(get_expr(r"<% !a && ((b != a) || c <= e) %>")).unwrap();
    let result = eval.run(
      &mut context,
      &HashMap::new(),
      &HashMap::new(),
      &mut includes,
    );

    assert_eq!(result.unwrap(), Operand::Value(Value::from(false)));
  }
  {
    let eval = Eval::new(get_expr(r"<% !b && ((b != a) || c <= e && !d) %>")).unwrap();
    let result = eval.run(
      &mut context,
      &HashMap::new(),
      &HashMap::new(),
      &mut includes,
    );

    assert_eq!(result.unwrap(), Operand::Value(Value::from(true)));
  }
  {
    let eval = Eval::new(get_expr(
      r#"<% (a == 1) && (b == 0) && (c == "abc") && !d && e == "def" %>"#,
    ))
    .unwrap();
    let result = eval.run(
      &mut context,
      &HashMap::new(),
      &HashMap::new(),
      &mut includes,
    );

    assert_eq!(result.unwrap(), Operand::Value(Value::from(true)));
  }
}

#[test]
fn compute_complex_object_name() {
  let context_json = r#"{"a": {"b" : 2, "c" : {"d" : 3 }}, "b" : 1}"#;
  let context_value: Value = serde_json::from_str(context_json).unwrap();
  let mut context = Context::new(context_value);
  let mut includes = Mutex::new(HashMap::new());

  {
    let eval = Eval::new(get_expr(r"<% a.b %>")).unwrap();
    let result = eval.run(
      &mut context,
      &HashMap::new(),
      &HashMap::new(),
      &mut includes,
    );

    assert_eq!(result.unwrap(), Operand::Value(Value::from(2)));
  }
  {
    let eval = Eval::new(get_expr(r"<% a.c.d %>")).unwrap();
    let result = eval.run(
      &mut context,
      &HashMap::new(),
      &HashMap::new(),
      &mut includes,
    );

    assert_eq!(result.unwrap(), Operand::Value(Value::from(3)));
  }
  {
    let eval = Eval::new(get_expr(r#"<% b %>"#)).unwrap();
    let result = eval.run(
      &mut context,
      &HashMap::new(),
      &HashMap::new(),
      &mut includes,
    );

    assert_eq!(result.unwrap(), Operand::Value(Value::from(1)));
  }
}

#[test]
fn compute_assign_test() {
  let context_json = r#"{"a": 1, "b":0, "c": 1}"#;
  let mut includes = Mutex::new(HashMap::new());

  {
    let context_value: Value = serde_json::from_str(context_json).unwrap();
    let mut context = Context::new(context_value);

    let eval = Eval::new(get_expr(r"<% a = a && b %>")).unwrap();
    let result = eval.run(
      &mut context,
      &HashMap::new(),
      &HashMap::new(),
      &mut includes,
    );

    assert_eq!(result.unwrap(), Operand::Value(Value::from(false)));
    assert!(!context.context.get("a").unwrap().as_bool().unwrap());
  }
  {
    let context_value: Value = serde_json::from_str(context_json).unwrap();
    let mut context = Context::new(context_value);

    let eval = Eval::new(get_expr(r"<% a = a && c %>")).unwrap();
    let result = eval.run(
      &mut context,
      &HashMap::new(),
      &HashMap::new(),
      &mut includes,
    );

    assert_eq!(result.unwrap(), Operand::Value(Value::from(true)));
    assert!(context.context.get("a").unwrap().as_bool().unwrap());
  }
  {
    let context_value: Value = serde_json::from_str(context_json).unwrap();
    let mut context = Context::new(context_value);

    let eval = Eval::new(get_expr(r"<% d = a && c %>")).unwrap();
    let result = eval.run(
      &mut context,
      &HashMap::new(),
      &HashMap::new(),
      &mut includes,
    );

    assert_eq!(result.unwrap(), Operand::Value(Value::from(true)));
    assert!(context.context.get("d").unwrap().as_bool().unwrap());
  }
}

#[test]
fn compute_arithmetic() {
  let context_json = r#"{"a": 1, "b":2, "c": 3, "d" : 2, "e" : 6, "f" : 2}"#;
  let mut includes = Mutex::new(HashMap::new());

  {
    let context_value: Value = serde_json::from_str(context_json).unwrap();
    let mut context = Context::new(context_value);

    // (1 + 2 * 3 - 6 / 2) * 2
    let eval = Eval::new(get_expr(r"<% b = a = (a + b * c - e / d) * f %>")).unwrap();
    let result = eval.run(
      &mut context,
      &HashMap::new(),
      &HashMap::new(),
      &mut includes,
    );

    assert_eq!(result.unwrap(), Operand::Value(Value::from(8)));
    assert_eq!(context.context.get("a").unwrap().as_i64().unwrap(), 8);
    assert_eq!(context.context.get("b").unwrap().as_i64().unwrap(), 8);
  }
}

#[test]
fn check_array_get() {
  let context_json = r#"{"a": [1,2,3]}"#;
  let mut includes = Mutex::new(HashMap::new());

  {
    let context_value: Value = serde_json::from_str(context_json).unwrap();
    let mut context = Context::new(context_value);

    // (1 + 2 * 3 - 6 / 2) * 2
    let eval = Eval::new(get_expr(r"<% a[1] %>")).unwrap();
    let result = eval.run(
      &mut context,
      &HashMap::new(),
      &HashMap::new(),
      &mut includes,
    );

    assert_eq!(result.unwrap(), Operand::Value(Value::from(2)));
  }
}

#[test]
fn convert_object_to_boolean() {
  let context_json = r#"{"a": {"b" : 1}}"#;
  let mut includes = Mutex::new(HashMap::new());

  // {
  //     let context_value: Value = serde_json::from_str(context_json).unwrap();
  //     let mut context = Context::new(context_value);

  //     let eval = Eval::new(get_expr(r"<% !b %>")).unwrap();
  //     let result = eval.run(
  //         &mut context,
  //         &HashMap::new(),
  //         &HashMap::new(),
  //         &mut includes,
  //     );

  //     assert_eq!(result.unwrap(), Operand::Value(Value::from(true)));
  // }

  {
    let context_value: Value = serde_json::from_str(context_json).unwrap();
    let mut context = Context::new(context_value);

    let eval = Eval::new(get_expr(r"<% a || !a %>")).unwrap();
    let result = eval.run(
      &mut context,
      &HashMap::new(),
      &HashMap::new(),
      &mut includes,
    );

    assert_eq!(result.unwrap(), Operand::Value(Value::from(true)));
  }
}
