use rspack_core::CompilationAsset;
use serde::Deserialize;
use sha2::{Digest, Sha256, Sha384, Sha512};

use crate::visitors::asset::{HTMLPluginTag, HtmlPluginAttribute};

#[derive(Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum HtmlSriHashFunction {
  Sha256,
  Sha384,
  Sha512,
}

pub fn create_digest_from_asset(
  hash_func: &HtmlSriHashFunction,
  asset: &CompilationAsset,
) -> String {
  let byte_content = asset.buffer();
  match hash_func {
    HtmlSriHashFunction::Sha384 => {
      let mut hasher = Sha384::new();
      hasher.update(byte_content);
      let digest = &hasher.finalize()[..];
      format!("sha384-{}", base64::encode(digest))
    }
    HtmlSriHashFunction::Sha256 => {
      let mut hasher = Sha256::new();
      hasher.update(byte_content);
      let digest = &hasher.finalize()[..];
      format!("sha256-{}", base64::encode(digest))
    }
    HtmlSriHashFunction::Sha512 => {
      let mut hasher = Sha512::new();
      hasher.update(byte_content);
      let digest = &hasher.finalize()[..];
      format!("sha512-{}", base64::encode(digest))
    }
  }
}

pub fn add_sri(tag: &mut HTMLPluginTag, sri: &String) {
  let mut has_crossorigin = false;
  let mut has_integrity = false;
  tag.attributes.iter_mut().for_each(|attribute| {
    if attribute.attr_name.eq("integrity") {
      has_integrity = true;
      attribute.attr_value = Some(sri.to_string());
    }
    if attribute.attr_name.eq("crossorigin") {
      has_crossorigin = true;
      attribute.attr_value = Some("anonymous".to_string());
    }
  });
  if !has_crossorigin {
    tag.attributes.push(HtmlPluginAttribute {
      attr_name: "crossorigin".to_string(),
      attr_value: Some("anonymous".to_string()),
    });
  }
  if !has_integrity {
    tag.attributes.push(HtmlPluginAttribute {
      attr_name: "integrity".to_string(),
      attr_value: Some(sri.to_string()),
    });
  }
}
