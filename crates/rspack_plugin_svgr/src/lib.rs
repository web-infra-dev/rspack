use async_trait::async_trait;
use core::fmt::Debug;
pub static PLUGIN_NAME: &'static str = "rspack_svgr";
use rspack_core::{
  BundleContext, LoadedSource, Loader, Plugin, PluginLoadHookOutput, PluginTransformRawHookOutput,
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

#[async_trait]
impl Plugin for SvgrPlugin {
  fn name(&self) -> &'static str {
    PLUGIN_NAME
  }

  fn transform_raw(
    &self,
    _ctx: &BundleContext,
    id: &str,
    loader: &mut Loader,
    raw: String,
  ) -> PluginTransformRawHookOutput {
    let n = id.find(|c: char| c == '?').or(Some(id.len())).unwrap();
    let p = Path::new(&id[..n]);
    let ext = p.extension().and_then(|ext| ext.to_str()).unwrap_or("js");

    if ext == "svg" {
      let use_svgr = id[n..].contains("svgr");
      let format = "base64";
      let data_uri = format!(
        "data:{};{},{}",
        "image/svg+xml",
        format,
        base64::encode(&raw)
      );

      if !use_svgr {
        *loader = Loader::Js;
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

      *loader = Loader::Jsx;

      lazy_static! {
        static ref RE: Regex = Regex::new(r"<svg (.*?)>").unwrap();
      }
      let result = RE.replace(&raw, |caps: &Captures| {
        format!("<svg {} {{...props}}>", &caps[1])
      });

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
