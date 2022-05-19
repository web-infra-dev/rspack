use async_trait::async_trait;
use core::fmt::Debug;
use rspack_core::PluginTransformHookOutput;
pub static PLUGIN_NAME: &'static str = "rspack_svgr";
use rspack_core::{
  BundleContext, LoadedSource, Loader, Plugin, PluginLoadHookOutput, PluginTransformAstHookOutput,
};
use std::{path::Path, sync::Arc};
// #[macro_use]
// extern crate lazy_static;
use regex::Captures;
use regex::Regex;
use std::fs;
extern crate lazy_static;
use lazy_static::*;

#[derive(Debug)]
pub struct SvgrPlugin {}
impl SvgrPlugin {}

fn clean_svgr(text: &str) -> String {
  lazy_static! {
    // todo: use ast map attrs
    static ref RE_REMOVE: Vec<(Regex, &'static str)> = {
      let v = vec![
        (Regex::new(r"(?s)<svg (.*?)>").unwrap(), "<svg $1 {...props}>"),
        (Regex::new(r"(?s)<\?xml (.*?)\?>").unwrap(), ""),
        (Regex::new(r"(?s)<!--(.*?)-->").unwrap(), ""),
        (Regex::new(r"(?s)<!DOCTYPE(.*?)>").unwrap(), ""),
        (Regex::new(r"(?s)<style(.*?)style>").unwrap(), ""),
        (Regex::new(r"xmlns:xlink").unwrap(), "xmlnsXlink"),
        (Regex::new(r"xml:space").unwrap(), "xmlSpace"),
      ];
      v
    };
  }

  let result = RE_REMOVE.iter().fold(text.to_string(), |text, re| {
    re.0.replace(&text, re.1).to_string()
  });

  result
}

#[async_trait]
impl Plugin for SvgrPlugin {
  fn name(&self) -> &'static str {
    PLUGIN_NAME
  }
  #[inline]
  async fn load(&self, _ctx: &BundleContext, id: &str) -> PluginLoadHookOutput {
    let query_start = id.find(|c: char| c == '?').or(Some(id.len())).unwrap();
    let file_path = Path::new(&id[..query_start]);
    let ext = file_path
      .extension()
      .and_then(|ext| ext.to_str())
      .unwrap_or("js");

    if ext == "svg" {
      let loader = Some(Loader::Js);
      let content = None;
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
      let result = clean_svgr(&raw);

      return format!(
        r#"
        import * as React from "react";
        const SvgComponent = (props) => (
           {}
        );
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
