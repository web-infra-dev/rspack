use std::fmt::Debug;

use crate::Environment;

pub struct RuntimeTemplate {
  environment: Environment,
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

  pub fn returning_function(&self, return_value: &str, args: &str) -> String {
    if self.environment.supports_arrow_function() {
      format!("({args}) => ({return_value})")
    } else {
      format!("function({args}) {{ return {return_value}; }}")
    }
  }

  pub fn basic_function(&self, args: &str, body: &str) -> String {
    if self.environment.supports_arrow_function() {
      format!("({args}) => {{\n {body} \n}}")
    } else {
      format!("function({args}) {{\n {body} \n}}")
    }
  }
}
