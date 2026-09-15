use rspack_core::{
  BoxPlugin, ExternalItem, ExternalItemFnCtx, ExternalItemFnResult, ExternalItemValue, PluginExt,
};

use crate::ExternalsPlugin;

pub fn http_externals_rspack_plugin(web_async: bool) -> BoxPlugin {
  if web_async {
    ExternalsPlugin::new(
      "import".to_owned(),
      vec![http_external_item_web_async()],
      false,
    )
    .boxed()
  } else {
    ExternalsPlugin::new("module".to_owned(), vec![http_external_item_web()], false).boxed()
  }
}

pub fn css_http_externals_rspack_plugin() -> BoxPlugin {
  ExternalsPlugin::new("module".to_owned(), vec![css_http_external_item()], false).boxed()
}

fn css_http_external_item() -> ExternalItem {
  ExternalItem::Fn(Box::new(move |ctx: ExternalItemFnCtx| {
    Box::pin(async move {
      if is_css_issuer(&ctx.context_info.issuer) && is_external_http_request(&ctx.request) {
        if ctx.dependency_type == "url" {
          return Ok(ExternalItemFnResult {
            external_type: Some("asset".to_owned()),
            result: Some(ExternalItemValue::String(ctx.request)),
          });
        } else if is_external_css_import_dependency(&ctx.dependency_type) {
          return Ok(ExternalItemFnResult {
            external_type: Some("css-import".to_owned()),
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

fn http_external_item_web() -> ExternalItem {
  ExternalItem::Fn(Box::new(move |ctx: ExternalItemFnCtx| {
    Box::pin(async move {
      if ctx.dependency_type == "url" {
        if is_external_http_request(&ctx.request) {
          return Ok(ExternalItemFnResult {
            external_type: Some("asset".to_owned()),
            result: Some(ExternalItemValue::String(ctx.request)),
          });
        }
      } else if is_external_css_import_dependency(&ctx.dependency_type) {
        if is_external_http_request(&ctx.request) {
          return Ok(ExternalItemFnResult {
            external_type: Some("css-import".to_owned()),
            result: Some(ExternalItemValue::String(ctx.request)),
          });
        }
      } else if is_external_http_std_request(&ctx.request) {
        if is_external_css_request(&ctx.request) {
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

fn http_external_item_web_async() -> ExternalItem {
  ExternalItem::Fn(Box::new(move |ctx: ExternalItemFnCtx| {
    Box::pin(async move {
      if ctx.dependency_type == "url" {
        if is_external_http_request(&ctx.request) {
          return Ok(ExternalItemFnResult {
            external_type: Some("asset".to_owned()),
            result: Some(ExternalItemValue::String(ctx.request)),
          });
        }
      } else if is_external_css_import_dependency(&ctx.dependency_type) {
        if is_external_http_request(&ctx.request) {
          return Ok(ExternalItemFnResult {
            external_type: Some("css-import".to_owned()),
            result: Some(ExternalItemValue::String(ctx.request)),
          });
        }
      } else if is_external_http_std_request(&ctx.request) {
        if is_external_css_request(&ctx.request) {
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

fn is_external_css_import_dependency(input: &str) -> bool {
  matches!(
    input,
    "css-import" | "css-import-local-module" | "css-import-global-module"
  )
}

fn is_css_issuer(input: &str) -> bool {
  input.ends_with(".css") || input.contains(".css?")
}
