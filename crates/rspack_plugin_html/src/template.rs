use std::path::PathBuf;

use anyhow::{Context, anyhow};
use itertools::Itertools;
use rspack_core::{Compilation, Mode};
use rspack_dojang::{Dojang, Operand, dojang::DojangOptions};
use rspack_error::{AnyhowResultToRspackResultExt, Result, ToStringResultToRspackResultExt, error};
use rspack_paths::AssertUtf8;
use serde_json::Value;

use crate::{
  asset::HtmlPluginAssets,
  config::{HtmlRspackPluginOptions, TemplateParameters},
  tag::HtmlPluginTag,
};

#[derive(Debug)]
pub enum TemplateRender {
  Template(String),
  Function,
}

#[derive(Debug)]
pub struct HtmlTemplate {
  pub render: TemplateRender,
  pub url: String,
  pub filename: String,
  pub file_dependencies: Vec<PathBuf>,
  pub parameters: Option<Value>,
}

impl HtmlTemplate {
  pub async fn new(config: &HtmlRspackPluginOptions, compilation: &Compilation) -> Result<Self> {
    if let Some(content) = &config.template_content {
      Ok(Self {
        render: if config.template_fn.is_some() {
          TemplateRender::Function
        } else {
          TemplateRender::Template(content.clone())
        },
        url: "template_content.html".to_string(),
        filename: "template_content.html".to_string(),
        file_dependencies: vec![],
        parameters: None,
      })
    } else if let Some(template) = &config.template {
      // TODO: support loader query form
      let resolved_template = path_clean::clean(
        compilation
          .options
          .context
          .as_path()
          .join(template.as_str()),
      )
      .assert_utf8();
      let url = resolved_template.as_str().to_string();

      if config.template_fn.is_some() {
        Ok(Self {
          render: TemplateRender::Function,
          url,
          filename: template.clone(),
          file_dependencies: vec![],
          parameters: None,
        })
      } else {
        compilation
          .input_filesystem
          .read_to_string(&resolved_template)
          .await
          .map_err(|err| anyhow!(err))
          .context(format!(
            "HtmlRspackPlugin: could not load file `{}` from `{}`",
            template, &compilation.options.context
          ))
          .map(|content| Self {
            render: TemplateRender::Template(content),
            url,
            filename: template.clone(),
            file_dependencies: vec![resolved_template.into_std_path_buf()],
            parameters: None,
          })
          .to_rspack_result_from_anyhow()
      }
    } else {
      let default_src_template =
        path_clean::clean(compilation.options.context.as_path().join("src/index.ejs"))
          .assert_utf8();

      if let Ok(content) = compilation
        .input_filesystem
        .read_to_string(&default_src_template)
        .await
      {
        Ok(Self {
          render: TemplateRender::Template(content),
          url: default_src_template.as_str().to_string(),
          filename: "src/index.ejs".to_string(),
          file_dependencies: vec![default_src_template.into_std_path_buf()],
          parameters: None,
        })
      } else {
        Ok(Self {
          render: TemplateRender::Template(default_template().to_owned()),
          url: "default.html".to_string(),
          filename: "default.html".to_string(),
          file_dependencies: vec![],
          parameters: None,
        })
      }
    }
  }

  pub async fn create_parameters(
    &mut self,
    filename: &str,
    config: &HtmlRspackPluginOptions,
    head_tags: &Vec<HtmlPluginTag>,
    body_tags: &Vec<HtmlPluginTag>,
    assets: &HtmlPluginAssets,
    compilation: &Compilation,
  ) -> Result<()> {
    if matches!(config.template_parameters, TemplateParameters::Disabled) {
      self.parameters = Some(serde_json::json!({}));
      Ok(())
    } else {
      let mut res = serde_json::json!({});

      merge_json(
        &mut res,
        serde_json::json!({
          "htmlRspackPlugin": {
            "tags": {
              "headTags": &head_tags,
              "bodyTags": &body_tags,
            },
            "files": &assets,
            "options": &config,
          },
        }),
      );

      // only support "mode" and some fields of "output"
      merge_json(
        &mut res,
        serde_json::json!({
          "rspackConfig": {
            "mode": match compilation.options.mode {
              Mode::Development => "development",
              Mode::Production => "production",
              Mode::None => "none",
            },
            "output": {
              "publicPath": config.get_public_path(compilation, filename).await,
              "crossOriginLoading": compilation.options.output.cross_origin_loading.to_string(),
            }
          },
        }),
      );

      match &config.template_parameters {
        TemplateParameters::Map(data) => {
          merge_json(&mut res, serde_json::json!(&data));
        }
        TemplateParameters::Function(func) => {
          let func_res = (func.inner)(
            serde_json::to_string(&res).unwrap_or_else(|_| panic!("invalid json to_string")),
          )
          .await;
          match func_res {
            Ok(new_data) => match serde_json::from_str(&new_data) {
              Ok(data) => res = data,
              Err(err) => {
                return Err(error!(
                  "HtmlRspackPlugin: failed to parse template parameters: {}",
                  err
                ));
              }
            },
            Err(err) => {
              return Err(error!(
                "HtmlRspackPlugin: failed to generate template parameters: {}",
                err
              ));
            }
          }
        }
        TemplateParameters::Disabled => {}
      };

      self.parameters = Some(res);
      Ok(())
    }
  }

  pub async fn render(&mut self, config: &HtmlRspackPluginOptions) -> Result<String> {
    let parameters = self.parameters.to_owned().expect("should have parameters");
    match &self.render {
      TemplateRender::Template(content) => {
        // process with template parameters
        let mut dj = Dojang::new();
        // align escape | unescape with lodash.template syntax https://lodash.com/docs/4.17.15#template which is html-webpack-plugin's default behavior
        dj.with_options(DojangOptions {
          escape: "-".to_string(),
          unescape: "=".to_string(),
        });

        dj.add_function_1("toHtml".into(), render_tag)
          .expect("failed to add template function `renderTag`");

        dj.add_with_option(self.url.clone(), content.clone())
          .expect("failed to add template");

        dj.render(&self.url, parameters)
          .to_rspack_result_with_message(|e| {
            format!("HtmlRspackPlugin: failed to render template from string: {e}")
          })
      }
      TemplateRender::Function => (config
        .template_fn
        .as_ref()
        .unwrap_or_else(|| unreachable!())
        .inner)(
        serde_json::to_string(&parameters).unwrap_or_else(|_| panic!("invalid json to_string")),
      )
      .await
      .to_rspack_result_with_message(|e| {
        format!("HtmlRspackPlugin: failed to render template from function: {e}")
      }),
    }
  }
}

fn default_template() -> &'static str {
  r#"<!DOCTYPE html>
<html>
  <head>
    <meta charset="utf-8">
    <title>rspack</title>
  </head>
  <body>
  </body>
</html>"#
}

pub fn merge_json(a: &mut Value, b: Value) {
  match (a, b) {
    (a @ &mut Value::Object(_), Value::Object(b)) => {
      let a = a
        .as_object_mut()
        .unwrap_or_else(|| panic!("merged json is not an object"));
      for (k, v) in b {
        merge_json(a.entry(k).or_insert(Value::Null), v);
      }
    }
    (a, b) => *a = b,
  }
}

pub fn render_tag(op: Operand) -> Operand {
  match op {
    Operand::Value(obj) => match serde_json::from_value::<HtmlPluginTag>(obj) {
      Ok(tag) => Operand::Value(Value::from(tag.to_string().as_str())),
      Err(_) => Operand::Value(Value::from("")),
    },
    Operand::Array(obj) => Operand::Value(Value::from(
      obj
        .iter()
        .map(|val| match render_tag(val.to_owned()) {
          Operand::Value(val) => val.as_str().unwrap_or_default().to_string(),
          _ => "".to_string(),
        })
        .join(""),
    )),
    _ => Operand::Value(Value::from("")),
  }
}
