/*
 * The following code is modified based on
 * https://github.com/swc-project/plugins/tree/main/packages/relay
 *
 * Copyright (c) 2021 kdy1(Donny/강동윤), kwonoj(OJ Kwon), XiNiHa(Cosmo Shin (신의하)), beaumontjonathan(Jonathan Beaumont)
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */
use std::path::{Path, PathBuf};

use once_cell::sync::Lazy;
use regex::Regex;
use rspack_core::{RelayConfig, RelayLanguageConfig};
use swc_core::{
  common::{Mark, DUMMY_SP},
  ecma::{
    ast::*,
    atoms::JsWord,
    utils::ExprFactory,
    visit::{Fold, FoldWith},
  },
};

struct Relay<'a> {
  unresolved_mark: Mark,
  root_dir: PathBuf,
  file_name: &'a Path,
  config: &'a RelayConfig,
}

fn pull_first_operation_name_from_tpl(tpl: &TaggedTpl) -> Option<String> {
  tpl.tpl.quasis.iter().find_map(|quasis| {
    #[allow(clippy::unwrap_in_result)]
    static OPERATION_REGEX: Lazy<Regex> = Lazy::new(|| {
      Regex::new(r"(fragment|mutation|query|subscription) (\w+)").expect("This cannot fail")
    });

    let capture_group = OPERATION_REGEX.captures_iter(&quasis.raw).next();

    capture_group.map(|capture_group| capture_group[2].to_string())
  })
}

fn build_require_expr_from_path(path: &str, unresolved_mark: Mark) -> Expr {
  Expr::Call(CallExpr {
    span: Default::default(),
    callee: create_require(unresolved_mark).as_callee(),
    args: vec![Lit::Str(Str {
      span: Default::default(),
      value: JsWord::from(path),
      raw: None,
    })
    .as_arg()],
    type_args: None,
  })
}

fn create_require(unresolved_mark: Mark) -> Ident {
  Ident {
    span: DUMMY_SP.apply_mark(unresolved_mark),
    sym: "require".into(),
    optional: false,
  }
}

impl<'a> Fold for Relay<'a> {
  fn fold_expr(&mut self, expr: Expr) -> Expr {
    let expr = expr.fold_children_with(self);

    match &expr {
      Expr::TaggedTpl(tpl) => {
        if let Some(built_expr) = self.build_call_expr_from_tpl(tpl) {
          built_expr
        } else {
          expr
        }
      }
      _ => expr,
    }
  }
}

#[derive(Debug)]
#[allow(dead_code)]
enum BuildRequirePathError {
  FileNameNotReal,
}

impl<'a> Relay<'a> {
  #[allow(clippy::unwrap_in_result)]
  fn path_for_artifact(
    &self,
    real_file_name: &Path,
    definition_name: &str,
  ) -> Result<PathBuf, BuildRequirePathError> {
    let filename = match self.config.language {
      RelayLanguageConfig::Flow | RelayLanguageConfig::JavaScript => {
        format!("{definition_name}.graphql.js")
      }
      RelayLanguageConfig::TypeScript => {
        format!("{definition_name}.graphql.ts")
      }
    };

    if let Some(artifact_directory) = &self.config.artifact_directory {
      Ok(self.root_dir.join(artifact_directory).join(filename))
    } else {
      Ok(
        real_file_name
          .parent()
          .expect("")
          .join("__generated__")
          .join(filename),
      )
    }
  }

  fn build_require_path(&mut self, operation_name: &str) -> Result<PathBuf, BuildRequirePathError> {
    self.path_for_artifact(self.file_name, operation_name)
  }

  fn build_call_expr_from_tpl(&mut self, tpl: &TaggedTpl) -> Option<Expr> {
    if let Expr::Ident(ident) = &*tpl.tag {
      if &*ident.sym != "graphql" {
        return None;
      }
    }

    let operation_name = pull_first_operation_name_from_tpl(tpl);

    match operation_name {
      None => None,
      Some(operation_name) => match self.build_require_path(operation_name.as_str()) {
        Ok(final_path) => Some(build_require_expr_from_path(
          final_path.to_str()?,
          self.unresolved_mark,
        )),
        Err(_err) => {
          // let base_error = "Could not transform GraphQL template to a Relay import.";
          // let error_message = match err {
          //     BuildRequirePathError::FileNameNotReal => {
          //         "Source file was not a real file.".to_string()
          //     }
          // };

          // HANDLER.with(|handler| {
          //     handler.span_err(
          //         tpl.span,
          //         format!("{} {}", base_error, error_message).as_str(),
          //     );
          // });

          None
        }
      },
    }
  }
}

pub fn relay<'a>(
  config: &'a RelayConfig,
  file_name: &'a Path,
  root_dir: PathBuf,
  unresolved_mark: Mark,
) -> impl Fold + 'a {
  Relay {
    root_dir,
    file_name,
    config,
    unresolved_mark,
  }
}
