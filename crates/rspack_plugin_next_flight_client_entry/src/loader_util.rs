use std::collections::HashMap;

use lazy_regex::Lazy;
use regex::Regex;
use rspack_core::Module;

use crate::get_module_build_info::{get_module_rsc_information, RSC_MODULE_TYPES};

// Gives { id: name } record of actions from the build info.
pub fn get_actions_from_build_info(module: &dyn Module) -> Option<HashMap<String, String>> {
  let rsc = get_module_rsc_information(module)?;
  rsc.action_ids
}

pub static REGEX_CSS: Lazy<Regex> = Lazy::new(|| Regex::new(r"\.(css|scss|sass)(\?.*)?$").unwrap());

// This function checks if a module is able to emit CSS resources. You should
// never only rely on a single regex to do that.
pub fn is_css_mod(module: &dyn Module) -> bool {
  if module.module_type().as_str() == "css/mini-extract" {
    return true;
  }
  let Some(module) = module.as_normal_module() else {
    return false;
  };
  REGEX_CSS.is_match(&module.resource_resolved_data().resource)
    || module.loaders().iter().any(|loader| {
      let loader_ident = loader.identifier();
      loader_ident.contains("next-style-loader/index.js")
        || loader_ident.contains("rspack.CssExtractRspackPlugin.loader")
        || loader_ident.contains("mini-css-extract-plugin/loader.js")
        || loader_ident.contains("@vanilla-extract/webpack-plugin/loader/")
    })
}

pub static IMAGE_REGEX: Lazy<Regex> = Lazy::new(|| {
  let image_extensions = vec!["jpg", "jpeg", "png", "webp", "avif", "ico", "svg"];
  Regex::new(&format!(r"\.({})$", image_extensions.join("|"))).unwrap()
});

pub fn is_client_component_entry_module(module: &dyn Module) -> bool {
  let rsc = get_module_rsc_information(module);
  let has_client_directive = matches!(rsc, Some(rsc) if rsc.is_client_ref);
  let is_action_layer_entry = is_action_client_layer_module(module);
  let is_image = if let Some(module) = module.as_normal_module() {
    IMAGE_REGEX.is_match(&module.resource_resolved_data().resource)
  } else {
    false
  };
  has_client_directive || is_action_layer_entry || is_image
}

// Determine if the whole module is client action, 'use server' in nested closure in the client module
fn is_action_client_layer_module(module: &dyn Module) -> bool {
  let rsc = get_module_rsc_information(module);
  matches!(&rsc, Some(rsc) if rsc.actions.is_some())
    && matches!(&rsc, Some(rsc) if rsc.r#type == RSC_MODULE_TYPES.client)
}
