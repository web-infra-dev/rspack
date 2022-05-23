mod clean;
mod mapping;
mod transform;
use async_trait::async_trait;
use clean::clean;
use core::fmt::Debug;
use rspack_core::PluginTransformHookOutput;
pub static PLUGIN_NAME: &str = "rspack_svgr";
use rspack_core::{BundleContext, LoadArgs, LoadedSource, Loader, Plugin, PluginLoadHookOutput};
use std::path::Path;
// #[macro_use]
// extern crate lazy_static;
use regex::Regex;
use std::fs::read_to_string;
use std::path::Path;
use transform::transform;

pub static PLUGIN_NAME: &'static str = "rspack_svgr";
#[derive(Debug)]
pub struct SvgrPlugin {}
impl SvgrPlugin {}
#[async_trait]
impl Plugin for SvgrPlugin {
  fn name(&self) -> &'static str {
    PLUGIN_NAME
  }
  #[inline]
  async fn load(&self, _ctx: &BundleContext, args: &LoadArgs) -> PluginLoadHookOutput {
    let query_start = args
      .id
      .find(|c: char| c == '?')
      .or(Some(args.id.len()))
      .unwrap();
    let file_path = Path::new(&args.id[..query_start]);
    let ext = file_path
      .extension()
      .and_then(|ext| ext.to_str())
      .unwrap_or("js");

    if ext == "svg" {
      let loader = Some(Loader::Js);
      let content =
        Some(read_to_string(file_path).unwrap_or_else(|_| panic!("file not exits {:?}", args.id)));
      Some(LoadedSource { loader, content })
    } else {
      None
    }
  }
  fn transform(
    &self,
    _ctx: &BundleContext,
    id: &str,
    loader: &mut Option<Loader>,
    raw: String,
  ) -> PluginTransformHookOutput {
    let query_start = id.find(|c: char| c == '?').or(Some(id.len())).unwrap();
    let file_path = Path::new(&id[..query_start]);
    let ext = file_path
      .extension()
      .and_then(|ext| ext.to_str())
      .unwrap_or("js");

    if ext == "svg" {
      if !_ctx.options.svgr {
        return raw;
      }

      let use_raw = id[query_start..].contains("raw");
      let format = "base64";
      let data_uri = format!(
        "data:{};{},{}",
        "image/svg+xml",
        format,
        base64::encode(&raw)
      );

      if use_raw {
        *loader = Some(Loader::Js);
        return format!(
          "
          var img = \"{}\";
          export default img;
          ",
          data_uri
        )
        .trim()
        .to_string();
      }

      *loader = Some(Loader::Jsx);
      let result = clean(&raw);
      let result = transform(result);
      return format!(
        r#"
        import * as React from "react";
        const SvgComponent = (props) => {{
           return {};
        }};
        export default SvgComponent;
        "#,
        result
      )
      .trim()
      .to_string();
    }
    raw
  }
}
