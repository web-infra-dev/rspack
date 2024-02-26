mod eval_array_expr;
mod eval_binary_expr;
mod eval_call_expr;
mod eval_cond_expr;
mod eval_lit_expr;
mod eval_new_expr;
mod eval_tpl_expr;
mod eval_unary_expr;

use bitflags::bitflags;
use rspack_core::DependencyLocation;

pub use self::eval_array_expr::eval_array_expression;
pub use self::eval_binary_expr::eval_binary_expression;
pub use self::eval_call_expr::eval_call_expression;
pub use self::eval_cond_expr::eval_cond_expression;
pub use self::eval_lit_expr::{eval_lit_expr, eval_prop_name};
pub use self::eval_new_expr::eval_new_expression;
pub use self::eval_tpl_expr::{eval_tpl_expression, TemplateStringKind};
pub use self::eval_unary_expr::eval_unary_expression;
use crate::visitors::ExportedVariableInfo;

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
enum Ty {
  Unknown,
  Undefined,
  Null,
  String,
  Number,
  Boolean,
  RegExp,
  Conditional,
  Array,
  Wrapped,
  ConstArray,
  BigInt,
  Identifier,
  TemplateString,
}

type Boolean = bool;
type Number = f64;
type Bigint = num_bigint::BigInt;
// type Array<'a> = &'a ast::ArrayLit;
type String = swc_core::atoms::Atom;
type Regexp = (String, String); // (expr, flags)

// I really don't want there has many alloc, maybe this can be optimized after
// parse finished.
#[derive(Debug)]
pub struct BasicEvaluatedExpression {
  ty: Ty,
  range: Option<DependencyLocation>,
  falsy: bool,
  truthy: bool,
  side_effects: bool,
  nullish: Option<bool>,
  boolean: Option<Boolean>,
  number: Option<Number>,
  string: Option<String>,
  bigint: Option<Bigint>,
  regexp: Option<Regexp>,
  identifier: Option<String>,
  root_info: Option<ExportedVariableInfo>,
  items: Option<Vec<BasicEvaluatedExpression>>,
  quasis: Option<Vec<BasicEvaluatedExpression>>,
  parts: Option<Vec<BasicEvaluatedExpression>>,
  prefix: Option<Box<BasicEvaluatedExpression>>,
  postfix: Option<Box<BasicEvaluatedExpression>>,
  wrapped_inner_expressions: Option<Vec<BasicEvaluatedExpression>>,
  // array: Option<Array>
  template_string_kind: Option<TemplateStringKind>,

  options: Option<Vec<BasicEvaluatedExpression>>,
}

impl Default for BasicEvaluatedExpression {
  fn default() -> Self {
    Self::new()
  }
}

impl BasicEvaluatedExpression {
  pub fn new() -> Self {
    Self {
      ty: Ty::Unknown,
      range: None,
      falsy: false,
      truthy: false,
      side_effects: true,
      nullish: None,
      boolean: None,
      number: None,
      bigint: None,
      quasis: None,
      parts: None,
      identifier: None,
      root_info: None,
      template_string_kind: None,
      options: None,
      string: None,
      items: None,
      regexp: None,
      postfix: None,
      prefix: None,
      wrapped_inner_expressions: None,
    }
  }

  pub fn with_range(start: u32, end: u32) -> Self {
    let mut expr = BasicEvaluatedExpression::new();
    expr.set_range(start, end);
    expr
  }

  // pub fn is_unknown(&self) -> bool {
  //   matches!(self.ty, Ty::Unknown)
  // }

  pub fn is_identifier(&self) -> bool {
    matches!(self.ty, Ty::Identifier)
  }

  pub fn is_null(&self) -> bool {
    matches!(self.ty, Ty::Null)
  }

  pub fn is_unknown(&self) -> bool {
    matches!(self.ty, Ty::Unknown)
  }

  pub fn is_undefined(&self) -> bool {
    matches!(self.ty, Ty::Undefined)
  }

  pub fn is_conditional(&self) -> bool {
    matches!(self.ty, Ty::Conditional)
  }

  pub fn is_string(&self) -> bool {
    matches!(self.ty, Ty::String)
  }

  pub fn is_bool(&self) -> bool {
    matches!(self.ty, Ty::Boolean)
  }

  pub fn is_array(&self) -> bool {
    matches!(self.ty, Ty::Array)
  }

  pub fn is_wrapped(&self) -> bool {
    matches!(self.ty, Ty::Wrapped)
  }

  pub fn is_number(&self) -> bool {
    matches!(self.ty, Ty::Number)
  }

  pub fn is_bigint(&self) -> bool {
    matches!(self.ty, Ty::BigInt)
  }

  pub fn is_template_string(&self) -> bool {
    matches!(self.ty, Ty::TemplateString)
  }

  pub fn is_regexp(&self) -> bool {
    matches!(self.ty, Ty::RegExp)
  }

  pub fn is_compile_time_value(&self) -> bool {
    matches!(
      self.ty,
      Ty::Undefined
        | Ty::Null
        | Ty::String
        | Ty::Number
        | Ty::Boolean
        | Ty::RegExp
        | Ty::ConstArray
        | Ty::BigInt
    )
  }

  pub fn is_nullish(&self) -> Option<bool> {
    self.nullish
  }

  pub fn is_primitive_type(&self) -> Option<bool> {
    match self.ty {
      Ty::Undefined
      | Ty::Null
      | Ty::String
      | Ty::Number
      | Ty::Boolean
      | Ty::BigInt
      | Ty::Wrapped
      | Ty::TemplateString => Some(true),
      Ty::RegExp | Ty::Array | Ty::ConstArray => Some(false),
      _ => None,
    }
  }

  pub fn as_string(&self) -> Option<String> {
    if self.is_bool() {
      if self.bool() {
        Some(String::from("true"))
      } else {
        Some(String::from("false"))
      }
    } else if self.is_null() {
      Some(String::from("null"))
    } else if self.is_string() {
      Some(self.string().clone())
    } else {
      None
    }
  }

  pub fn as_bool(&self) -> Option<Boolean> {
    if self.truthy {
      Some(true)
    } else if self.falsy || self.nullish == Some(true) || self.is_null() || self.is_undefined() {
      Some(false)
    } else {
      self.boolean
    }
  }

  pub fn as_nullish(&self) -> Option<bool> {
    let nullish = self.is_nullish();
    if nullish == Some(true) || self.is_null() || self.is_undefined() {
      Some(true)
    } else if nullish == Some(false)
      || self.is_bool()
      || self.is_string()
      || self.is_template_string()
    {
      Some(false)
    } else {
      None
    }
  }

  pub fn compare_compile_time_value(&self, b: &BasicEvaluatedExpression) -> bool {
    if self.ty != b.ty {
      false
    } else {
      match self.ty {
        Ty::Undefined => matches!(b.ty, Ty::Undefined),
        Ty::Null => matches!(b.ty, Ty::Null),
        Ty::String => {
          b.string.as_ref().expect("should not empty")
            == self.string.as_ref().expect("should not empty")
        }
        Ty::Number => {
          b.number.as_ref().expect("should not empty")
            == self.number.as_ref().expect("should not empty")
        }
        Ty::Boolean => {
          b.boolean.as_ref().expect("should not empty")
            == self.boolean.as_ref().expect("should not empty")
        }
        Ty::RegExp => false, // FIXME: maybe we can use std::ptr::eq
        // Ty::ConstArray => {
        // },
        Ty::BigInt => {
          b.bigint.as_ref().expect("should not empty")
            == self.bigint.as_ref().expect("should not empty")
        }
        _ => unreachable!("can only compare compile-time values"),
      }
    }
  }

  pub fn could_have_side_effects(&self) -> bool {
    self.side_effects
  }

  pub fn set_side_effects(&mut self, side_effects: bool) {
    self.side_effects = side_effects
  }

  pub fn set_null(&mut self) {
    self.ty = Ty::Null;
    self.side_effects = false
  }

  pub fn set_undefined(&mut self) {
    self.ty = Ty::Undefined;
    self.side_effects = false;
  }

  pub fn set_number(&mut self, number: Number) {
    self.ty = Ty::Number;
    self.number = Some(number);
    self.side_effects = false;
  }

  pub fn set_truthy(&mut self) {
    self.falsy = false;
    self.truthy = true;
    self.nullish = Some(false);
  }

  pub fn set_falsy(&mut self) {
    self.falsy = true;
    self.truthy = false;
  }

  pub fn set_nullish(&mut self, nullish: bool) {
    self.nullish = Some(nullish);
    if nullish {
      self.set_falsy()
    }
  }

  pub fn set_items(&mut self, items: Vec<BasicEvaluatedExpression>) {
    self.ty = Ty::Array;
    self.side_effects = items.iter().any(|item| item.could_have_side_effects());
    self.items = Some(items);
  }

  pub fn options(&self) -> &Vec<BasicEvaluatedExpression> {
    self.options.as_ref().expect("options should not empty")
  }

  pub fn set_options(&mut self, options: Option<Vec<BasicEvaluatedExpression>>) {
    self.ty = Ty::Conditional;
    self.options = options;
    self.side_effects = true;
  }

  pub fn add_options(&mut self, options: Vec<BasicEvaluatedExpression>) {
    if let Some(old) = &mut self.options {
      old.extend(options);
    } else {
      self.ty = Ty::Conditional;
      self.options = Some(options);
      self.side_effects = true;
    }
  }

  pub fn set_identifier(&mut self, name: String, root_info: ExportedVariableInfo) {
    self.ty = Ty::Identifier;
    self.identifier = Some(name);
    self.root_info = Some(root_info);
    self.side_effects = true;
  }

  pub fn set_bool(&mut self, boolean: Boolean) {
    self.ty = Ty::Boolean;
    self.boolean = Some(boolean);
    self.side_effects = true
  }

  pub fn set_range(&mut self, start: u32, end: u32) {
    self.range = Some(DependencyLocation::new(start, end))
  }

  pub fn set_template_string(
    &mut self,
    quasis: Vec<BasicEvaluatedExpression>,
    parts: Vec<BasicEvaluatedExpression>,
    kind: TemplateStringKind,
  ) {
    self.ty = Ty::TemplateString;
    self.quasis = Some(quasis);
    self.side_effects = parts.iter().any(|p| p.side_effects);
    self.parts = Some(parts);
    self.template_string_kind = Some(kind);
  }

  pub fn set_string(&mut self, string: String) {
    self.ty = Ty::String;
    self.string = Some(string);
    self.side_effects = false;
  }

  pub fn set_regexp(&mut self, regexp: String, flags: String) {
    self.ty = Ty::RegExp;
    self.regexp = Some((regexp, flags));
    self.side_effects = false;
  }

  pub fn set_wrapped(
    &mut self,
    prefix: Option<BasicEvaluatedExpression>,
    postfix: Option<BasicEvaluatedExpression>,
    inner_expressions: Vec<BasicEvaluatedExpression>,
  ) {
    self.ty = Ty::Wrapped;
    self.prefix = prefix.map(Box::new);
    self.postfix = postfix.map(Box::new);
    self.wrapped_inner_expressions = Some(inner_expressions);
    self.side_effects = true;
  }

  pub fn string(&self) -> &String {
    self.string.as_ref().expect("make sure string exist")
  }

  pub fn identifier(&self) -> &String {
    assert!(self.is_identifier());
    self
      .identifier
      .as_ref()
      .expect("make sure identifier exist")
  }

  pub fn root_info(&self) -> &ExportedVariableInfo {
    assert!(self.is_identifier());
    self.root_info.as_ref().expect("make sure identifier exist")
  }

  pub fn regexp(&self) -> &Regexp {
    self.regexp.as_ref().expect("make sure regexp exist")
  }

  pub fn bool(&self) -> Boolean {
    self.boolean.expect("make sure bool exist")
  }

  pub fn range(&self) -> (u32, u32) {
    let range = self.range.expect("range should not empty");
    (range.start(), range.end())
  }

  pub fn prefix(&self) -> Option<&BasicEvaluatedExpression> {
    assert!(self.is_wrapped(), "prefix is only used in wrapped");
    self.prefix.as_deref()
  }

  pub fn postfix(&self) -> Option<&BasicEvaluatedExpression> {
    assert!(self.is_wrapped(), "postfix is only used in wrapped");
    self.postfix.as_deref()
  }

  pub fn template_string_kind(&self) -> TemplateStringKind {
    assert!(self.is_template_string());
    self
      .template_string_kind
      .expect("make sure template string exist")
  }

  pub fn parts(&self) -> &Vec<BasicEvaluatedExpression> {
    assert!(self.is_template_string());
    self
      .parts
      .as_ref()
      .expect("make sure template string exist")
  }

  pub fn quasis(&self) -> &Vec<BasicEvaluatedExpression> {
    assert!(self.is_template_string(),);
    self
      .quasis
      .as_ref()
      .expect("quasis must exists for template string")
  }

  pub fn number(&self) -> Number {
    assert!(self.is_number());
    self.number.expect("number must exists in ty::number")
  }
}

pub fn evaluate_to_string(value: String, start: u32, end: u32) -> BasicEvaluatedExpression {
  let mut eval = BasicEvaluatedExpression::with_range(start, end);
  eval.set_string(value);
  eval
}

pub fn evaluate_to_identifier(
  identifier: String,
  root_info: String,
  truthy: Option<bool>,
  start: u32,
  end: u32,
) -> BasicEvaluatedExpression {
  let mut eval = BasicEvaluatedExpression::with_range(start, end);
  eval.set_identifier(identifier, ExportedVariableInfo::Name(root_info));
  eval.set_side_effects(false);
  match truthy {
    Some(v) => {
      if v {
        eval.set_truthy();
      } else {
        eval.set_falsy();
      }
    }
    None => eval.set_nullish(true),
  };
  eval
}

bitflags! {
  struct RegExpFlag: u8 {
    const FLAG_Y = 1 << 0;
    const FLAG_M = 1 << 1;
    const FLAG_I = 1 << 2;
    const FLAG_G = 1 << 3;
  }
}

pub fn is_valid_reg_exp_flags(flags: &str) -> bool {
  if flags.is_empty() {
    true
  } else if flags.len() > 4 {
    false
  } else {
    let mut remaining = RegExpFlag::empty();
    for c in flags.as_bytes() {
      match *c {
        b'g' => {
          if remaining.contains(RegExpFlag::FLAG_G) {
            return false;
          }
          remaining.insert(RegExpFlag::FLAG_G);
        }
        b'i' => {
          if remaining.contains(RegExpFlag::FLAG_I) {
            return false;
          }
          remaining.insert(RegExpFlag::FLAG_I);
        }
        b'm' => {
          if remaining.contains(RegExpFlag::FLAG_M) {
            return false;
          }
          remaining.insert(RegExpFlag::FLAG_M);
        }
        b'y' => {
          if remaining.contains(RegExpFlag::FLAG_Y) {
            return false;
          }
          remaining.insert(RegExpFlag::FLAG_Y);
        }
        _ => return false,
      }
    }
    true
  }
}
