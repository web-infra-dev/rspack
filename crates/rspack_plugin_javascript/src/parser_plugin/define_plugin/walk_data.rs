use std::{
  borrow::Cow,
  sync::{Arc, LazyLock},
};

use itertools::Itertools as _;
use regex::Regex;
use rspack_error::Diagnostic;
use rustc_hash::FxHashMap;
use serde_json::{Map, Value, json};
use swc_core::common::Span;

use super::{
  ConflictingValuesError, DefineValue,
  utils::{code_to_string, gen_const_dep},
};
use crate::{utils::eval::BasicEvaluatedExpression, visitors::JavascriptParser};

static TYPEOF_OPERATOR_REGEXP: LazyLock<Regex> =
  LazyLock::new(|| Regex::new("^typeof\\s+").expect("should init `TYPEOF_OPERATOR_REGEXP`"));

type OnEvaluateIdentifier = dyn Fn(
    &DefineRecord,
    &mut JavascriptParser,
    &str, /* Ident */
    u32,  /* start */
    u32,  /* end */
  ) -> Option<BasicEvaluatedExpression<'static>>
  + Send
  + Sync;

type OnEvaluateTypeof = dyn Fn(
    &DefineRecord,
    &mut JavascriptParser,
    u32, /* start */
    u32, /* end */
  ) -> Option<BasicEvaluatedExpression<'static>>
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

pub struct DefineRecord {
  code: Value,
  pub on_evaluate_identifier: Option<Box<OnEvaluateIdentifier>>,
  pub on_evaluate_typeof: Option<Box<OnEvaluateTypeof>>,
  pub on_expression: Option<Box<OnExpression>>,
  pub on_typeof: Option<Box<OnTypeof>>,
}

impl std::fmt::Debug for DefineRecord {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    macro_rules! debug_fn_field {
      ($field:expr) => {{
        if $field.is_some() {
          &"Some(..)"
        } else {
          &"None"
        }
      }};
    }
    f.debug_struct("DefineRecord")
      .field("code", &self.code)
      .field(
        "on_evaluate_identifier",
        debug_fn_field!(self.on_evaluate_identifier),
      )
      .field(
        "on_evaluate_typeof",
        debug_fn_field!(self.on_evaluate_typeof),
      )
      .field("on_expression", debug_fn_field!(self.on_expression))
      .field("on_typeof", debug_fn_field!(self.on_typeof))
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
pub struct ObjectDefineRecord {
  object: Value,
  pub on_evaluate_identifier: Option<Box<OnObjectEvaluateIdentifier>>,
  pub on_expression: Option<Box<OnObjectExpression>>,
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
  ) -> Option<BasicEvaluatedExpression<'static>>
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
pub struct WalkData {
  pub tiling_definitions: FxHashMap<String, String>,
  pub diagnostics: Vec<Diagnostic>,
  pub can_rename: FxHashMap<Arc<str>, Option<Arc<str>>>,
  pub define_record: FxHashMap<Arc<str>, DefineRecord>,
  pub typeof_define_record: FxHashMap<Arc<str>, DefineRecord>,
  pub object_define_record: FxHashMap<Arc<str>, ObjectDefineRecord>,
}

impl WalkData {
  pub fn new(definitions: &DefineValue) -> Self {
    let mut data = Self::default();
    data.setup_value_cache(definitions.iter(), "".into());
    data.setup_record(definitions);
    data
  }

  fn setup_value_cache<'d, 's>(
    &mut self,
    definitions: impl Iterator<Item = (&'d String, &'d Value)>,
    prefix: Cow<'s, str>,
  ) {
    definitions.for_each(|(key, value)| {
      let name = format!("{prefix}{key}");
      let value_str = value.to_string();
      if let Some(prev) = self.tiling_definitions.get(&name)
        && !prev.eq(&value_str)
      {
        self.diagnostics.push(
          ConflictingValuesError(format!("{prefix}{key}"), prev.clone(), value_str)
            .into_diagnostic(),
        );
      } else {
        self.tiling_definitions.insert(name, value_str);
      }
      if let Some(value) = value.as_object() {
        self.setup_value_cache(value.iter(), Cow::Owned(format!("{prefix}{key}.")))
      } else if let Some(value) = value.as_array() {
        let indexes = (0..value.len())
          .map(|index| {
            let mut index_buffer = rspack_util::itoa::Buffer::new();
            index_buffer.format(index).to_string()
          })
          .collect_vec();
        let iter = indexes.iter().zip(value.iter());
        self.setup_value_cache(iter, Cow::Owned(format!("{prefix}{key}.")))
      }
    });
  }

  fn setup_record(&mut self, definitions: &DefineValue) {
    fn apply_define_key(prefix: Cow<str>, key: Cow<str>, walk_data: &mut WalkData) {
      let splitted: Vec<&str> = key.split('.').collect();
      if !splitted.is_empty() {
        let iter = (0..splitted.len() - 1).map(|i| {
          let full_key = Arc::<str>::from(format!("{prefix}{}", splitted[0..i + 1].join(".")));
          let first_key = Some(Arc::<str>::from(splitted[0]));
          (full_key, first_key)
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
        walk_data.can_rename.insert(key.clone(), None);
        define_record = define_record
          .with_on_evaluate_identifier(Box::new(move |record, parser, _ident, start, end| {
            parser
              .evaluate(
                code_to_string(&record.code, None, None).into_owned(),
                "DefinePlugin",
              )
              .map(|mut evaluated| {
                evaluated.set_range(start, end);
                evaluated
              })
          }))
          .with_on_expression(Box::new(
            move |record, parser, span, start, end, for_name| {
              let code = code_to_string(&record.code, Some(!parser.is_asi_position(span.lo)), None);
              parser.add_presentational_dependency(Box::new(gen_const_dep(
                parser, code, for_name, start, end,
              )));
              Some(true)
            },
          ));
      }

      define_record = define_record
        .with_on_evaluate_typeof(Box::new(move |record, parser, start, end| {
          let code = code_to_string(&record.code, None, None);
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
          let code = code_to_string(&record.code, None, None);
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
              parser.add_presentational_dependency(Box::new(gen_const_dep(
                parser,
                Cow::Owned(format!("{}", json!(evaluated.string()))),
                "",
                start,
                end,
              )));
              Some(true)
            })
        }));

      if is_typeof {
        walk_data.typeof_define_record.insert(key, define_record);
      } else {
        walk_data.define_record.insert(key, define_record);
      }
    }

    fn object_evaluate_identifier(start: u32, end: u32) -> BasicEvaluatedExpression<'static> {
      let mut evaluated = BasicEvaluatedExpression::new();
      evaluated.set_truthy();
      evaluated.set_side_effects(false);
      evaluated.set_range(start, end);
      evaluated
    }

    fn apply_array_define(key: Cow<str>, obj: &[Value], walk_data: &mut WalkData) {
      let key = Arc::<str>::from(key);
      walk_data.can_rename.insert(key.clone(), None);
      let define_record = ObjectDefineRecord::from_code(Value::Array(obj.to_owned()))
        .with_on_evaluate_identifier(Box::new(move |_, _, _, start, end| {
          Some(object_evaluate_identifier(start, end))
        }))
        .with_on_expression(Box::new(
          move |record, parser, span, start, end, for_name| {
            let code = code_to_string(&record.object, Some(!parser.is_asi_position(span.lo)), None);
            parser.add_presentational_dependency(Box::new(gen_const_dep(
              parser, code, for_name, start, end,
            )));
            Some(true)
          },
        ));
      walk_data.object_define_record.insert(key, define_record);
    }

    fn apply_object_define(key: Cow<str>, obj: &Map<String, Value>, walk_data: &mut WalkData) {
      let key = Arc::<str>::from(key);
      walk_data.can_rename.insert(key.clone(), None);
      let define_record = ObjectDefineRecord::from_code(Value::Object(obj.clone()))
        .with_on_evaluate_identifier(Box::new(move |_, _, _, start, end| {
          Some(object_evaluate_identifier(start, end))
        }))
        .with_on_expression(Box::new(
          move |record, parser, span, start, end, for_name| {
            let code = code_to_string(
              &record.object,
              Some(!parser.is_asi_position(span.lo)),
              parser.destructuring_assignment_properties.get(&span),
            );
            parser.add_presentational_dependency(Box::new(gen_const_dep(
              parser, code, for_name, start, end,
            )));
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
        walk_code(code, prefix.clone(), Cow::Owned(key.clone()), walk_data)
      })
    }

    let object = definitions.clone().into_iter().collect();
    walk_object(&object, "".into(), self);
  }
}
