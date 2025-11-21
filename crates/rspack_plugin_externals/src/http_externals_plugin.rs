use rspack_core::{
  BoxPlugin, ExternalItem, ExternalItemFnCtx, ExternalItemFnResult, ExternalItemValue, PluginExt,
};

use crate::ExternalsPlugin;

pub fn http_externals_rspack_plugin(css: bool, web_async: bool) -> BoxPlugin {
  if web_async {
    ExternalsPlugin::new(
      "import".to_owned(),
      vec![http_external_item_web_async(css)],
      false,
    )
    .boxed()
  } else {
    ExternalsPlugin::new(
      "module".to_owned(),
      vec![http_external_item_web(css)],
      false,
    )
    .boxed()
  }
}

fn http_external_item_web(css: bool) -> ExternalItem {
  ExternalItem::Fn(Box::new(move |ctx: ExternalItemFnCtx| {
    Box::pin(async move {
      if ctx.dependency_type == "url" {
        if is_external_http_request(&ctx.request) {
          return Ok(ExternalItemFnResult {
            external_type: Some("asset".to_owned()),
            result: Some(ExternalItemValue::String(ctx.request)),
          });
        }
      } else if css && ctx.dependency_type == "css-import" {
        if is_external_http_request(&ctx.request) {
          return Ok(ExternalItemFnResult {
            external_type: Some("css-import".to_owned()),
            result: Some(ExternalItemValue::String(ctx.request)),
          });
        }
      } else if is_external_http_std_request(&ctx.request) {
        if css && is_external_css_request(&ctx.request) {
          return Ok(ExternalItemFnResult {
            external_type: Some("css-import".to_owned()),
            result: Some(ExternalItemValue::String(ctx.request)),
          });
        } else {
          return Ok(ExternalItemFnResult {
            external_type: Some("module".to_owned()),
            result: Some(ExternalItemValue::String(ctx.request)),
          });
        }
      }
      Ok(ExternalItemFnResult {
        external_type: None,
        result: None,
      })
    })
  }))
}

fn http_external_item_web_async(css: bool) -> ExternalItem {
  ExternalItem::Fn(Box::new(move |ctx: ExternalItemFnCtx| {
    Box::pin(async move {
      if ctx.dependency_type == "url" {
        if is_external_http_request(&ctx.request) {
          return Ok(ExternalItemFnResult {
            external_type: Some("asset".to_owned()),
            result: Some(ExternalItemValue::String(ctx.request)),
          });
        }
      } else if css && ctx.dependency_type == "css-import" {
        if is_external_http_request(&ctx.request) {
          return Ok(ExternalItemFnResult {
            external_type: Some("css-import".to_owned()),
            result: Some(ExternalItemValue::String(ctx.request)),
          });
        }
      } else if is_external_http_std_request(&ctx.request) {
        if css && is_external_css_request(&ctx.request) {
          return Ok(ExternalItemFnResult {
            external_type: Some("css-import".to_owned()),
            result: Some(ExternalItemValue::String(ctx.request)),
          });
        } else {
          return Ok(ExternalItemFnResult {
            external_type: Some("import".to_owned()),
            result: Some(ExternalItemValue::String(ctx.request)),
          });
        }
      }
      Ok(ExternalItemFnResult {
        external_type: None,
        result: None,
      })
    })
  }))
}

fn is_external_http_request(input: &str) -> bool {
  input.starts_with("//")
    || input.starts_with("https://")
    || input.starts_with("http://")
    || input.starts_with('#')
}

fn is_external_http_std_request(input: &str) -> bool {
  input.starts_with("//")
    || input.starts_with("https://")
    || input.starts_with("http://")
    || input.starts_with("std:")
}

fn is_external_css_request(input: &str) -> bool {
  input == ".css" || input.starts_with(".css?")
}
