use std::{borrow::Cow, sync::Arc};

use itertools::Itertools as _;
use once_cell::sync::Lazy;
use regex::Regex;
use rspack_core::{ConstDependency, RuntimeGlobals, SpanExt as _};
use rustc_hash::{FxHashMap, FxHashSet};
use serde_json::{json, Map, Value};
use swc_core::common::{Span, Spanned as _};

use super::DefineValue;
use crate::{
  utils::eval::{evaluate_to_string, BasicEvaluatedExpression},
  visitors::JavascriptParser,
  JavascriptParserPlugin,
};

static TYPEOF_OPERATOR_REGEXP: Lazy<Regex> =
  Lazy::new(|| Regex::new("^typeof\\s+").expect("should init `TYPEOF_OPERATOR_REGEXP`"));
static WEBPACK_REQUIRE_FUNCTION_REGEXP: Lazy<Regex> = Lazy::new(|| {
  Regex::new("__webpack_require__\\s*(!?\\.)")
    .expect("should init `WEBPACK_REQUIRE_FUNCTION_REGEXP`")
});
static WEBPACK_REQUIRE_IDENTIFIER_REGEXP: Lazy<Regex> = Lazy::new(|| {
  Regex::new("__webpack_require__").expect("should init `WEBPACK_REQUIRE_IDENTIFIER_REGEXP`")
});

type OnEvaluateIdentifier = dyn Fn(
    &DefineRecord,
    &mut JavascriptParser,
    &str, /* Ident */
    u32,  /* start */
    u32,  /* end */
  ) -> Option<BasicEvaluatedExpression>
  + Send
  + Sync;

type OnEvaluateTypeof = dyn Fn(
    &DefineRecord,
    &mut JavascriptParser,
    u32, /* start */
    u32, /* end */
  ) -> Option<BasicEvaluatedExpression>
  + Send
  + Sync;

type OnExpression = dyn Fn(
    &DefineRecord,
    &mut JavascriptParser,
    Span,
    u32,  /* replace start */
    u32,  /* replace end */
    &str, /* for name */
  ) -> Option<bool>
  + Send
  + Sync;

type OnTypeof = dyn Fn(&DefineRecord, &mut JavascriptParser, u32 /* start */, u32 /* end */) -> Option<bool>
  + Send
  + Sync;

struct DefineRecord {
  code: Value,
  on_evaluate_identifier: Option<Box<OnEvaluateIdentifier>>,
  on_evaluate_typeof: Option<Box<OnEvaluateTypeof>>,
  on_expression: Option<Box<OnExpression>>,
  on_typeof: Option<Box<OnTypeof>>,
}

impl std::fmt::Debug for DefineRecord {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("DefineRecord")
      .field("code", &self.code)
      .finish_non_exhaustive()
  }
}

impl DefineRecord {
  fn from_code(code: Value) -> DefineRecord {
    Self {
      code,
      on_evaluate_identifier: None,
      on_evaluate_typeof: None,
      on_expression: None,
      on_typeof: None,
    }
  }

  fn with_on_evaluate_identifier(
    mut self,
    on_evaluate_identifier: Box<OnEvaluateIdentifier>,
  ) -> Self {
    self.on_evaluate_identifier = Some(on_evaluate_identifier);
    self
  }

  fn with_on_evaluate_typeof(mut self, on_evaluate_typeof: Box<OnEvaluateTypeof>) -> Self {
    self.on_evaluate_typeof = Some(on_evaluate_typeof);
    self
  }

  fn with_on_expression(mut self, on_expression: Box<OnExpression>) -> Self {
    self.on_expression = Some(on_expression);
    self
  }

  fn with_on_typeof(mut self, on_typeof: Box<OnTypeof>) -> Self {
    self.on_typeof = Some(on_typeof);
    self
  }
}

#[derive(Default)]
struct ObjectDefineRecord {
  object: Value,
  on_evaluate_identifier: Option<Box<OnObjectEvaluateIdentifier>>,
  on_expression: Option<Box<OnObjectExpression>>,
}

impl std::fmt::Debug for ObjectDefineRecord {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("ObjectDefineRecord")
      .field("object", &self.object)
      .finish_non_exhaustive()
  }
}

type OnObjectEvaluateIdentifier = dyn Fn(
    &ObjectDefineRecord,
    &mut JavascriptParser,
    &str, /* Ident */
    u32,  /* start */
    u32,  /* end */
  ) -> Option<BasicEvaluatedExpression>
  + Send
  + Sync;

type OnObjectExpression = dyn Fn(
    &ObjectDefineRecord,
    &mut JavascriptParser,
    Span,
    u32,  /* replace start */
    u32,  /* replace end */
    &str, /* for name */
  ) -> Option<bool>
  + Send
  + Sync;

impl ObjectDefineRecord {
  fn from_code(obj: Value) -> Self {
    assert!(matches!(obj, Value::Array(_) | Value::Object(_)));
    Self {
      object: obj,
      on_evaluate_identifier: None,
      on_expression: None,
    }
  }

  fn with_on_evaluate_identifier(
    mut self,
    on_evaluate_identifier: Box<OnObjectEvaluateIdentifier>,
  ) -> Self {
    self.on_evaluate_identifier = Some(on_evaluate_identifier);
    self
  }

  fn with_on_expression(mut self, on_expression: Box<OnObjectExpression>) -> Self {
    self.on_expression = Some(on_expression);
    self
  }
}

#[derive(Debug, Default)]
pub(super) struct WalkData {
  can_rename: FxHashSet<Arc<str>>,
  define_record: FxHashMap<Arc<str>, DefineRecord>,
  object_define_record: FxHashMap<Arc<str>, ObjectDefineRecord>,
}

pub(super) fn walk_definitions(definitions: &DefineValue) -> WalkData {
  let mut data = WalkData::default();

  fn apply_define_key(prefix: Cow<str>, key: Cow<str>, walk_data: &mut WalkData) {
    let splitted: Vec<&str> = key.split('.').collect();
    if !splitted.is_empty() {
      let iter = (0..splitted.len() - 1).map(|i| {
        Arc::from(
          core::iter::once(&&*prefix)
            .chain(&splitted[0..i + 1])
            .join("."),
        )
      });
      walk_data.can_rename.extend(iter)
    }
  }

  fn apply_define(key: Cow<str>, code: &Value, walk_data: &mut WalkData) {
    let is_typeof = TYPEOF_OPERATOR_REGEXP.is_match(&key);
    let original_key = key;
    let key = if is_typeof {
      TYPEOF_OPERATOR_REGEXP.replace(&original_key, "")
    } else {
      original_key
    };
    let key = Arc::<str>::from(key);
    let mut define_record = DefineRecord::from_code(code.clone());
    if !is_typeof {
      walk_data.can_rename.insert(key.clone());
      define_record = define_record
        .with_on_evaluate_identifier(Box::new(move |record, parser, _ident, start, end| {
          let evaluated = parser
            .evaluate(to_code(&record.code, None).into_owned(), "DefinePlugin")
            .map(|mut evaluated| {
              evaluated.set_range(start, end);
              evaluated
            });
          evaluated
        }))
        .with_on_expression(Box::new(
          move |record, parser, span, start, end, for_name| {
            let code = to_code(&record.code, Some(!parser.is_asi_position(span.lo)));
            parser
              .presentational_dependencies
              .push(Box::new(dep(parser, code, for_name, start, end)));
            Some(true)
          },
        ));
    }

    define_record = define_record
      .with_on_evaluate_typeof(Box::new(move |record, parser, start, end| {
        let code = to_code(&record.code, None);
        let typeof_code = if is_typeof {
          code
        } else {
          Cow::Owned(format!("typeof ({code})"))
        };
        parser
          .evaluate(typeof_code.into_owned(), "DefinePlugin")
          .map(|mut evaluated| {
            evaluated.set_range(start, end);
            evaluated
          })
      }))
      .with_on_typeof(Box::new(move |record, parser, start, end| {
        let code = to_code(&record.code, None);
        let typeof_code = if is_typeof {
          code
        } else {
          Cow::Owned(format!("typeof ({code})"))
        };
        parser
          .evaluate(typeof_code.to_string(), "DefinePlugin")
          .and_then(|evaluated| {
            if !evaluated.is_string() {
              return None;
            }
            debug_assert!(!parser.in_short_hand);
            parser.presentational_dependencies.push(Box::new(dep(
              parser,
              Cow::Owned(format!("{}", json!(evaluated.string()))),
              "",
              start,
              end,
            )));
            Some(true)
          })
      }));

    walk_data.define_record.insert(key, define_record);
  }

  fn object_evaluate_identifier(start: u32, end: u32) -> BasicEvaluatedExpression {
    let mut evaluated = BasicEvaluatedExpression::new();
    evaluated.set_truthy();
    evaluated.set_side_effects(false);
    evaluated.set_range(start, end);
    evaluated
  }

  fn apply_array_define(key: Cow<str>, obj: &[Value], walk_data: &mut WalkData) {
    let key = Arc::<str>::from(key);
    walk_data.can_rename.insert(key.clone());
    let define_record = ObjectDefineRecord::from_code(Value::Array(obj.to_owned()))
      .with_on_evaluate_identifier(Box::new(move |_, _, _, start, end| {
        Some(object_evaluate_identifier(start, end))
      }))
      .with_on_expression(Box::new(
        move |record, parser, span, start, end, for_name| {
          let code = to_code(&record.object, Some(!parser.is_asi_position(span.lo)));
          parser
            .presentational_dependencies
            .push(Box::new(dep(parser, code, for_name, start, end)));
          Some(true)
        },
      ));
    walk_data.object_define_record.insert(key, define_record);
  }

  fn apply_object_define(key: Cow<str>, obj: &Map<String, Value>, walk_data: &mut WalkData) {
    let key = Arc::<str>::from(key);
    walk_data.can_rename.insert(key.clone());
    let define_record = ObjectDefineRecord::from_code(Value::Object(obj.clone()))
      .with_on_evaluate_identifier(Box::new(move |_, _, _, start, end| {
        Some(object_evaluate_identifier(start, end))
      }))
      .with_on_expression(Box::new(
        move |record, parser, span, start, end, for_name| {
          let code = to_code(&record.object, Some(!parser.is_asi_position(span.lo)));
          parser
            .presentational_dependencies
            .push(Box::new(dep(parser, code, for_name, start, end)));
          Some(true)
        },
      ));
    walk_data.object_define_record.insert(key, define_record);
  }

  fn walk_code(code: &Value, prefix: Cow<str>, key: Cow<str>, walk_data: &mut WalkData) {
    let prefix_for_object = || Cow::Owned(format!("{prefix}{key}."));
    if let Some(array) = code.as_array() {
      walk_array(array, prefix_for_object(), walk_data);
      apply_array_define(Cow::Owned(format!("{prefix}{key}")), array, walk_data);
    } else if let Some(obj) = code.as_object() {
      walk_object(obj, prefix_for_object(), walk_data);
      apply_object_define(Cow::Owned(format!("{prefix}{key}")), obj, walk_data);
    } else {
      apply_define_key(prefix.clone(), Cow::Owned(key.to_string()), walk_data);
      apply_define(Cow::Owned(format!("{prefix}{key}")), code, walk_data);
    }
  }

  fn walk_array(arr: &[Value], prefix: Cow<str>, walk_data: &mut WalkData) {
    arr.iter().enumerate().for_each(|(key, code)| {
      walk_code(code, prefix.clone(), Cow::Owned(key.to_string()), walk_data)
    })
  }

  fn walk_object(obj: &Map<String, Value>, prefix: Cow<str>, walk_data: &mut WalkData) {
    obj.iter().for_each(|(key, code)| {
      walk_code(code, prefix.clone(), Cow::Owned(key.to_string()), walk_data)
    })
  }

  let object = definitions.clone().into_iter().collect();
  walk_object(&object, "".into(), &mut data);

  data
}

pub(super) struct DefineParserPlugin {
  pub(super) walk_data: WalkData,
}

impl JavascriptParserPlugin for DefineParserPlugin {
  fn can_rename(&self, _: &mut JavascriptParser, str: &str) -> Option<bool> {
    self.walk_data.can_rename.contains(str).then_some(true)
  }

  fn evaluate_typeof(
    &self,
    parser: &mut JavascriptParser,
    expr: &swc_core::ecma::ast::UnaryExpr,
    for_name: &str,
  ) -> Option<BasicEvaluatedExpression> {
    if let Some(record) = self.walk_data.define_record.get(for_name)
      && let Some(on_evaluate_typeof) = &record.on_evaluate_typeof
    {
      return on_evaluate_typeof(record, parser, expr.span.real_lo(), expr.span.hi.0);
    } else if self.walk_data.object_define_record.get(for_name).is_some() {
      return Some(evaluate_to_string(
        "object".to_string(),
        expr.span.real_lo(),
        expr.span.hi.0,
      ));
    }
    None
  }

  fn evaluate_identifier(
    &self,
    parser: &mut JavascriptParser,
    ident: &str,
    start: u32,
    end: u32,
  ) -> Option<crate::utils::eval::BasicEvaluatedExpression> {
    if let Some(record) = self.walk_data.define_record.get(ident)
      && let Some(on_evaluate_identifier) = &record.on_evaluate_identifier
    {
      return on_evaluate_identifier(record, parser, ident, start, end);
    } else if let Some(record) = self.walk_data.object_define_record.get(ident)
      && let Some(on_evaluate_identifier) = &record.on_evaluate_identifier
    {
      return on_evaluate_identifier(record, parser, ident, start, end);
    }
    None
  }

  fn r#typeof(
    &self,
    parser: &mut JavascriptParser,
    expr: &swc_core::ecma::ast::UnaryExpr,
    for_name: &str,
  ) -> Option<bool> {
    if let Some(record) = self.walk_data.define_record.get(for_name)
      && let Some(on_typeof) = &record.on_typeof
    {
      return on_typeof(record, parser, expr.span.real_lo(), expr.span.real_hi());
    } else if self.walk_data.object_define_record.get(for_name).is_some() {
      debug_assert!(!parser.in_short_hand);
      parser.presentational_dependencies.push(Box::new(dep(
        parser,
        Cow::Borrowed(r#""object""#),
        for_name,
        expr.span.real_lo(),
        expr.span.real_hi(),
      )));
      return Some(true);
    }
    None
  }

  fn call(
    &self,
    parser: &mut JavascriptParser,
    expr: &swc_core::ecma::ast::CallExpr,
    for_name: &str,
  ) -> Option<bool> {
    if let Some(record) = self.walk_data.define_record.get(for_name)
      && let Some(on_expression) = &record.on_expression
    {
      return on_expression(
        record,
        parser,
        expr.span,
        expr.callee.span().real_lo(),
        expr.callee.span().real_hi(),
        for_name,
      )
      .map(|_| {
        // FIXME: webpack use `walk_expression` here
        parser.walk_expr_or_spread(&expr.args);
        true
      });
    } else if let Some(record) = self.walk_data.object_define_record.get(for_name)
      && let Some(on_expression) = &record.on_expression
    {
      return on_expression(
        record,
        parser,
        expr.span,
        expr.callee.span().real_lo(),
        expr.callee.span().real_hi(),
        for_name,
      )
      .map(|_| {
        // FIXME: webpack use `walk_expression` here
        parser.walk_expr_or_spread(&expr.args);
        true
      });
    }
    None
  }

  fn member(
    &self,
    parser: &mut JavascriptParser,
    expr: &swc_core::ecma::ast::MemberExpr,
    for_name: &str,
  ) -> Option<bool> {
    if let Some(record) = self.walk_data.define_record.get(for_name)
      && let Some(on_expression) = &record.on_expression
    {
      return on_expression(
        record,
        parser,
        expr.span,
        expr.span.real_lo(),
        expr.span.real_hi(),
        for_name,
      );
    } else if let Some(record) = self.walk_data.object_define_record.get(for_name)
      && let Some(on_expression) = &record.on_expression
    {
      return on_expression(
        record,
        parser,
        expr.span,
        expr.span.real_lo(),
        expr.span.real_hi(),
        for_name,
      );
    }
    None
  }

  fn identifier(
    &self,
    parser: &mut JavascriptParser,
    ident: &swc_core::ecma::ast::Ident,
    for_name: &str,
  ) -> Option<bool> {
    if let Some(record) = self.walk_data.define_record.get(for_name)
      && let Some(on_expression) = &record.on_expression
    {
      return on_expression(
        record,
        parser,
        ident.span,
        ident.span.real_lo(),
        ident.span.real_hi(),
        for_name,
      );
    } else if let Some(record) = self.walk_data.object_define_record.get(for_name)
      && let Some(on_expression) = &record.on_expression
    {
      return on_expression(
        record,
        parser,
        ident.span,
        ident.span.real_lo(),
        ident.span.real_hi(),
        for_name,
      );
    }
    None
  }
}

fn dep(
  parser: &JavascriptParser,
  code: Cow<str>,
  for_name: &str,
  start: u32,
  end: u32,
) -> ConstDependency {
  let code = if parser.in_short_hand {
    format!("{for_name}: {code}")
  } else {
    code.into_owned()
  };

  let to_const_dep = |requirements: Option<RuntimeGlobals>| {
    ConstDependency::new(start, end, code.clone().into_boxed_str(), requirements)
  };

  if WEBPACK_REQUIRE_FUNCTION_REGEXP.is_match(&code) {
    to_const_dep(Some(RuntimeGlobals::REQUIRE))
  } else if WEBPACK_REQUIRE_IDENTIFIER_REGEXP.is_match(&code) {
    to_const_dep(Some(RuntimeGlobals::REQUIRE_SCOPE))
  } else {
    to_const_dep(None)
  }
}

fn to_code(code: &Value, asi_safe: Option<bool>) -> Cow<str> {
  fn wrap_ansi(code: Cow<str>, is_arr: bool, asi_safe: Option<bool>) -> Cow<str> {
    match asi_safe {
      Some(true) if is_arr => code,
      Some(true) => Cow::Owned(format!("({code})")),
      Some(false) if is_arr => Cow::Owned(format!(";{code}")),
      Some(false) => Cow::Owned(format!(";({code})")),
      None => code,
    }
  }

  match code {
    Value::Null => Cow::Borrowed("null"),
    Value::String(s) => Cow::Borrowed(s),
    Value::Bool(b) => Cow::Borrowed(if *b { "true" } else { "false" }),
    Value::Number(n) => Cow::Owned(n.to_string()),
    Value::Array(arr) => {
      let elements = arr.iter().map(|code| to_code(code, None)).join(",");
      wrap_ansi(Cow::Owned(format!("[{elements}]")), true, asi_safe)
    }
    Value::Object(obj) => {
      let elements = obj
        .iter()
        .map(|(key, value)| format!("{}:{}", json!(key), to_code(value, None)))
        .join(",");
      wrap_ansi(Cow::Owned(format!("{{ {elements} }}")), false, asi_safe)
    }
  }
}
