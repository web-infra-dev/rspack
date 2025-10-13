use std::{error::Error, fmt, sync::Arc};

use rustc_hash::FxHashMap as HashMap;

type HelperFn = Arc<dyn Fn(&str) -> String + Send + Sync>;

#[derive(Clone, Debug)]
pub struct Template {
  segments: Vec<Segment>,
}

#[derive(Clone, Debug)]
enum Segment {
  Text(String),
  Value(Value),
}

#[derive(Clone, Debug)]
enum Value {
  Variable(String),
  Helper { helper: String, argument: String },
}

#[derive(Debug, Clone)]
pub enum TemplateError {
  UnclosedTag { position: usize },
  InvalidExpression { expression: String },
  UnknownHelper { name: String },
  MissingValue { name: String },
  TemplateNotFound { name: String },
}

impl fmt::Display for TemplateError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      TemplateError::UnclosedTag { position } => {
        write!(f, "Unclosed tag starting at byte {}", position)
      }
      TemplateError::InvalidExpression { expression } => {
        write!(f, "Invalid expression `{{{{ {} }}}}`", expression)
      }
      TemplateError::UnknownHelper { name } => write!(f, "Helper not found {}", name),
      TemplateError::MissingValue { name } => write!(f, "Missing value for `{}`", name),
      TemplateError::TemplateNotFound { name } => write!(f, "Template `{}` not found", name),
    }
  }
}

impl Error for TemplateError {}

impl Template {
  pub fn parse(input: &str) -> Result<Self, TemplateError> {
    let mut segments = Vec::new();
    let mut cursor = 0usize;
    let s = input;

    while let Some(start) = find_subsequence(s, cursor, "{{") {
      if start > cursor {
        segments.push(Segment::Text(input[cursor..start].to_string()));
      }

      let tag_start = start + 2;
      let end = find_subsequence(s, tag_start, "}}")
        .ok_or(TemplateError::UnclosedTag { position: start })?;

      let expression = input[tag_start..end].trim();
      if expression.is_empty() {
        return Err(TemplateError::InvalidExpression {
          expression: expression.to_string(),
        });
      }

      let segment = Value::parse(expression)?;
      segments.push(Segment::Value(segment));

      cursor = end + 2;
    }

    if cursor < input.len() {
      segments.push(Segment::Text(input[cursor..].to_string()));
    }

    Ok(Self { segments })
  }

  fn render(
    &self,
    engine: &TemplateEngine,
    ctx: &HashMap<&'static str, String>,
  ) -> Result<String, TemplateError> {
    let mut output = String::new();
    for segment in &self.segments {
      match segment {
        Segment::Text(text) => output.push_str(text),
        Segment::Value(value) => match value {
          Value::Variable(name) => {
            let value = ctx
              .get(name.as_str())
              .ok_or_else(|| TemplateError::MissingValue { name: name.clone() })?;
            output.push_str(value);
          }
          Value::Helper { helper, argument } => {
            let value = ctx
              .get(argument.as_str())
              .ok_or_else(|| TemplateError::MissingValue {
                name: argument.clone(),
              })?;
            let helper_fn =
              engine
                .helpers
                .get(helper.as_str())
                .ok_or_else(|| TemplateError::UnknownHelper {
                  name: helper.clone(),
                })?;
            let rendered = helper_fn(value);
            output.push_str(&rendered);
          }
        },
      }
    }
    Ok(output)
  }
}

impl Value {
  fn parse(expression: &str) -> Result<Value, TemplateError> {
    let mut parts = expression.split_whitespace();
    let first = parts.next().unwrap();
    let second = parts.next();

    if parts.next().is_some() {
      return Err(TemplateError::InvalidExpression {
        expression: expression.to_string(),
      });
    }

    if let Some(arg) = second {
      Ok(Value::Helper {
        helper: first.to_string(),
        argument: arg.to_string(),
      })
    } else {
      Ok(Value::Variable(first.to_string()))
    }
  }
}

#[derive(Clone, Default)]
pub struct TemplateEngine {
  helpers: HashMap<&'static str, HelperFn>,
  templates: HashMap<String, Template>,
}

impl TemplateEngine {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn register_helper<F>(&mut self, name: &'static str, helper: F)
  where
    F: Fn(&str) -> String + Send + Sync + 'static,
  {
    self.helpers.insert(name, Arc::new(helper));
  }

  pub fn register_template(&mut self, name: String, template: Template) {
    self.templates.insert(name, template);
  }

  pub fn render(
    &self,
    name: &str,
    ctx: &HashMap<&'static str, String>,
  ) -> Result<String, TemplateError> {
    let template = self
      .templates
      .get(name)
      .ok_or_else(|| TemplateError::TemplateNotFound {
        name: name.to_string(),
      })?;
    template.render(self, ctx)
  }
}

fn find_subsequence(haystack: &str, from: usize, needle: &str) -> Option<usize> {
  haystack[from..].find(needle).map(|off| off + from)
}

#[cfg(test)]
mod tests {
  use heck::ToKebabCase;

  use super::*;

  fn simple_ctx(value: &str) -> HashMap<&'static str, String> {
    let mut map = HashMap::default();
    map.insert("member", value.to_string());
    map
  }

  #[test]
  fn render_plain_and_variable() {
    let template = Template::parse("foo {{ member }} bar").unwrap();
    let mut engine = TemplateEngine::new();
    engine.register_template("test".to_string(), template);

    let rendered = engine.render("test", &simple_ctx("MyButton")).unwrap();
    assert_eq!(rendered, "foo MyButton bar");
  }

  #[test]
  fn render_with_helper() {
    let template = Template::parse("foo/{{ kebabCase member }}").unwrap();
    let mut engine = TemplateEngine::new();
    engine.register_helper("kebabCase", |value| value.to_kebab_case());
    engine.register_template("test".to_string(), template);

    let rendered = engine.render("test", &simple_ctx("MyButton")).unwrap();
    assert_eq!(rendered, "foo/my-button");
  }

  #[test]
  fn render_with_extra_whitespace() {
    let template = Template::parse("foo/{{    kebabCase   member   }}").unwrap();
    let mut engine = TemplateEngine::new();
    engine.register_helper("kebabCase", |value| value.to_kebab_case());
    engine.register_template("test".to_string(), template);

    let rendered = engine.render("test", &simple_ctx("MyButton")).unwrap();
    assert_eq!(rendered, "foo/my-button");
  }

  #[test]
  fn render_variable_with_padding() {
    let template = Template::parse("{{    member   }}").unwrap();
    let mut engine = TemplateEngine::new();
    engine.register_template("test".to_string(), template);

    let rendered = engine.render("test", &simple_ctx("Button")).unwrap();
    assert_eq!(rendered, "Button");
  }

  #[test]
  fn error_on_missing_value() {
    let template = Template::parse("{{ member }}").unwrap();
    let mut engine = TemplateEngine::new();
    engine.register_template("test".to_string(), template);

    let result = engine.render("test", &HashMap::default());
    assert!(matches!(result, Err(TemplateError::MissingValue { .. })));
  }

  #[test]
  fn error_on_unknown_helper() {
    let template = Template::parse("{{ snakeCase member }}").unwrap();
    let mut engine = TemplateEngine::new();
    engine.register_template("test".to_string(), template);

    let result = engine.render("test", &simple_ctx("MyButton"));
    assert!(matches!(result, Err(TemplateError::UnknownHelper { .. })));
  }

  #[test]
  fn error_on_unclosed_tag() {
    let result = Template::parse("foo {{ member");
    assert!(matches!(result, Err(TemplateError::UnclosedTag { .. })));
  }

  #[test]
  fn error_on_invalid_expression() {
    let result = Template::parse("{{ }}");
    assert!(matches!(
      result,
      Err(TemplateError::InvalidExpression { .. })
    ));
  }
}
