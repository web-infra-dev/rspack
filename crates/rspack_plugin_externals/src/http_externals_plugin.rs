use std::sync::LazyLock;

use regex::Regex;
use rspack_core::{
  BoxPlugin, ExternalItem, ExternalItemFnCtx, ExternalItemFnResult, ExternalItemValue, PluginExt,
};

use crate::ExternalsPlugin;

static EXTERNAL_HTTP_REQUEST: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"^(//|https?://|#)").expect("Invalid regex"));
static EXTERNAL_HTTP_STD_REQUEST: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"^(//|https?://|std:)").expect("Invalid regex"));
static EXTERNAL_CSS_REQUEST: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"^\.css(\?|$)").expect("Invalid regex"));

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
        if EXTERNAL_HTTP_REQUEST.is_match(&ctx.request) {
          return Ok(ExternalItemFnResult {
            external_type: Some("asset".to_owned()),
            result: Some(ExternalItemValue::String(ctx.request)),
          });
        }
      } else if css && ctx.dependency_type == "css-import" {
        if EXTERNAL_HTTP_REQUEST.is_match(&ctx.request) {
          return Ok(ExternalItemFnResult {
            external_type: Some("css-import".to_owned()),
            result: Some(ExternalItemValue::String(ctx.request)),
          });
        }
      } else if EXTERNAL_HTTP_STD_REQUEST.is_match(&ctx.request) {
        if css && EXTERNAL_CSS_REQUEST.is_match(&ctx.request) {
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
        if EXTERNAL_HTTP_REQUEST.is_match(&ctx.request) {
          return Ok(ExternalItemFnResult {
            external_type: Some("asset".to_owned()),
            result: Some(ExternalItemValue::String(ctx.request)),
          });
        }
      } else if css && ctx.dependency_type == "css-import" {
        if EXTERNAL_HTTP_REQUEST.is_match(&ctx.request) {
          return Ok(ExternalItemFnResult {
            external_type: Some("css-import".to_owned()),
            result: Some(ExternalItemValue::String(ctx.request)),
          });
        }
      } else if EXTERNAL_HTTP_STD_REQUEST.is_match(&ctx.request) {
        if css && EXTERNAL_CSS_REQUEST.is_match(&ctx.request) {
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
