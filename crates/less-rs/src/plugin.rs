//! Extensibility: host-provided functions and the owned [`Value`] type that
//! crosses the plugin boundary.

use std::fmt;

use crate::{
  css,
  error::{LResult, LessError},
  eval::EvalCtx,
  tree::*,
};

/// The result of a plugin function.
pub type FnResult = Result<Value, String>;

/// An owned Less value (converted at the plugin boundary).
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
  Dimension {
    value: f64,
    unit: String,
  },
  Color {
    rgb: [f64; 3],
    alpha: f64,
  },
  Keyword(String),
  Quoted {
    value: String,
    quote: Option<char>,
    escaped: bool,
  },
  Anonymous(String),
  Url(Box<Value>),
  List {
    items: Vec<Value>,
    comma: bool,
  },
  Bool(bool),
  Null,
}

impl Value {
  pub fn dimension(value: f64, unit: impl Into<String>) -> Value {
    Value::Dimension {
      value,
      unit: unit.into(),
    }
  }
  pub fn keyword(s: impl Into<String>) -> Value {
    Value::Keyword(s.into())
  }
  pub fn as_number(&self) -> Option<f64> {
    match self {
      Value::Dimension { value, .. } => Some(*value),
      _ => None,
    }
  }
}

impl fmt::Display for Value {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Value::Dimension { value, unit } => write!(f, "{}{}", crate::num::to_js_string(*value), unit),
      Value::Color { rgb, alpha } => {
        if *alpha < 1.0 {
          write!(
            f,
            "rgba({}, {}, {}, {})",
            rgb[0].round(),
            rgb[1].round(),
            rgb[2].round(),
            alpha
          )
        } else {
          f.write_str(&crate::color::to_hex(rgb))
        }
      }
      Value::Keyword(s) | Value::Anonymous(s) => f.write_str(s),
      Value::Quoted {
        value,
        quote,
        escaped,
      } => match (quote, escaped) {
        (Some(q), false) => write!(f, "{}{}{}", q, value, q),
        _ => f.write_str(value),
      },
      Value::Url(v) => write!(f, "url({})", v),
      Value::List { items, comma } => {
        let parts: Vec<String> = items.iter().map(|i| i.to_string()).collect();
        f.write_str(&parts.join(if *comma { ", " } else { " " }))
      }
      Value::Bool(b) => write!(f, "{}", b),
      Value::Null => Ok(()),
    }
  }
}

/// A function callable from Less code (`name(args...)`).
/// A dimension argument (see [`ValueArgs::dimension`]).
#[derive(Clone, Debug, PartialEq)]
pub struct DimensionArg {
  pub value: f64,
  pub unit: String,
}

/// Typed access to function arguments.
pub trait ValueArgs {
  /// Argument `i` as a dimension, or an "argument must be a number" error.
  fn dimension(&self, i: usize) -> Result<DimensionArg, String>;
  /// Argument `i` as a string (quoted, keyword or anonymous text).
  fn string(&self, i: usize) -> Result<String, String>;
}

impl ValueArgs for [Value] {
  fn dimension(&self, i: usize) -> Result<DimensionArg, String> {
    match self.get(i) {
      Some(Value::Dimension { value, unit }) => Ok(DimensionArg {
        value: *value,
        unit: unit.clone(),
      }),
      _ => Err(format!("argument {} must be a number", i + 1)),
    }
  }
  fn string(&self, i: usize) -> Result<String, String> {
    match self.get(i) {
      Some(Value::Quoted { value, .. })
      | Some(Value::Keyword(value))
      | Some(Value::Anonymous(value)) => Ok(value.clone()),
      _ => Err(format!("argument {} must be a string", i + 1)),
    }
  }
}

pub trait LessFunction: Send + Sync {
  fn call(&self, args: &[Value]) -> FnResult;
  /// `false` to receive unevaluated arguments.
  fn eval_args(&self) -> bool {
    true
  }
}

impl<F> LessFunction for F
where
  F: Fn(&[Value]) -> FnResult + Send + Sync,
{
  fn call(&self, args: &[Value]) -> FnResult {
    self(args)
  }
}

/// Transforms source text before it is parsed (root file and every import),
/// like less.js `PreProcessor`s.
pub trait PreProcessor: Send + Sync {
  fn process(&self, source: &str, filename: &str) -> String;
}

/// Transforms the generated CSS, like less.js `PostProcessor`s.
pub trait PostProcessor: Send + Sync {
  fn process(&self, css: String) -> String;
}

/// A plugin installs functions and processors into the registry.
pub trait Plugin: Send + Sync {
  fn install(&self, registry: &mut crate::functions::Registry);
  fn min_version(&self) -> Option<(u32, u32, u32)> {
    None
  }
}

pub(crate) fn to_value(n: Node<'_>) -> Value {
  match n {
    Node::Dimension(d) => Value::Dimension {
      value: d.value,
      unit: css::unit_to_string(d.unit),
    },
    Node::Color(c) => Value::Color {
      rgb: c.rgb,
      alpha: c.alpha,
    },
    Node::Keyword(k) => Value::Keyword(k.value.to_string()),
    Node::Quoted(q) => Value::Quoted {
      value: q.value.to_string(),
      quote: q.quote.chars().next(),
      escaped: q.escaped,
    },
    Node::Url(u) => Value::Url(Box::new(to_value(u.value))),
    Node::Expression(e) => Value::List {
      items: e.value.iter().map(|x| to_value(*x)).collect(),
      comma: false,
    },
    Node::Value(v) => Value::List {
      items: v.value.iter().map(|x| to_value(*x)).collect(),
      comma: true,
    },
    other => Value::Anonymous(css::to_css_plain(other).unwrap_or_default()),
  }
}

pub(crate) fn from_value<'a>(a: Alloc<'a>, v: &Value) -> Option<Node<'a>> {
  Some(match v {
    Value::Dimension { value, unit } => a.dimension_str(*value, a.str(unit)),
    Value::Color { rgb, alpha } => Node::Color(a.color(*rgb, *alpha, None)),
    Value::Keyword(s) => a.keyword(a.str(s)),
    Value::Quoted {
      value,
      quote,
      escaped,
    } => {
      let q = quote.map(|c| a.str(&c.to_string())).unwrap_or("");
      Node::Quoted(a.quoted(q, a.str(value), *escaped, Pos::NONE))
    }
    Value::Anonymous(s) => a.anonymous(a.str(s), Pos::NONE),
    Value::Url(inner) => {
      let n = from_value(a, inner)?;
      Node::Url(a.alloc(Url {
        value: n,
        pos: Pos::NONE,
        is_evald: true,
      }))
    }
    Value::List { items, comma } => {
      let nodes: Vec<Node<'a>> = items.iter().filter_map(|i| from_value(a, i)).collect();
      if *comma {
        a.value(&nodes)
      } else {
        Node::Expression(a.expression(&nodes, None))
      }
    }
    Value::Bool(b) => a.keyword(if *b { "true" } else { "false" }),
    Value::Null => return None,
  })
}

pub(crate) fn call_custom<'a>(
  ctx: &mut EvalCtx<'a, '_>,
  _pos: Pos,
  f: &dyn LessFunction,
  args: &[Node<'a>],
) -> LResult<Option<Node<'a>>> {
  let values: Vec<Value> = args.iter().map(|n| to_value(*n)).collect();
  match f.call(&values) {
    Ok(v) => Ok(from_value(ctx.a, &v)),
    Err(msg) => Err(LessError::new("Runtime", msg)),
  }
}
