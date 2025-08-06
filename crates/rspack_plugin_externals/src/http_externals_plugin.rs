use rspack_core::{
  BoxPlugin, ExternalItem, ExternalItemFnCtx, ExternalItemFnResult, ExternalItemValue, PluginExt,
};

use crate::ExternalsPlugin;

fn is_http_request(request: &str) -> bool {
  request.starts_with("//") || request.starts_with("http://") || request.starts_with("https://") || request.starts_with('#')
}

fn is_http_std_request(request: &str) -> bool {
  request.starts_with("//") || request.starts_with("http://") || request.starts_with("https://") || request.starts_with("std:")
}

fn is_css_request(request: &str) -> bool {
  request.starts_with(".css") && (request == ".css" || request.starts_with(".css?"))
}

pub fn http_externals_rspack_plugin(css: bool, web_async: bool) -> BoxPlugin {
  if web_async {
    ExternalsPlugin::new("import".to_owned(), vec![http_external_item_web_async(css)]).boxed()
  } else {
    ExternalsPlugin::new("module".to_owned(), vec![http_external_item_web(css)]).boxed()
  }
}

fn http_external_item_web(css: bool) -> ExternalItem {
  ExternalItem::Fn(Box::new(move |ctx: ExternalItemFnCtx| {
    Box::pin(async move {
      if ctx.dependency_type == "url" {
        if is_http_request(&ctx.request) {
          return Ok(ExternalItemFnResult {
            external_type: Some("asset".to_owned()),
            result: Some(ExternalItemValue::String(ctx.request)),
          });
        }
      } else if css && ctx.dependency_type == "css-import" {
        if is_http_request(&ctx.request) {
          return Ok(ExternalItemFnResult {
            external_type: Some("css-import".to_owned()),
            result: Some(ExternalItemValue::String(ctx.request)),
          });
        }
      } else if is_http_std_request(&ctx.request) {
        if css && is_css_request(&ctx.request) {
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
        if is_http_request(&ctx.request) {
          return Ok(ExternalItemFnResult {
            external_type: Some("asset".to_owned()),
            result: Some(ExternalItemValue::String(ctx.request)),
          });
        }
      } else if css && ctx.dependency_type == "css-import" {
        if is_http_request(&ctx.request) {
          return Ok(ExternalItemFnResult {
            external_type: Some("css-import".to_owned()),
            result: Some(ExternalItemValue::String(ctx.request)),
          });
        }
      } else if is_http_std_request(&ctx.request) {
        if css && is_css_request(&ctx.request) {
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
