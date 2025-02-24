use rspack_core::Module;

const CLIENT_MODULE_LABEL: &str =
  r"/\* __next_internal_client_entry_do_not_use__ ([^ ]*) (cjs|auto) \*/";
const ACTION_MODULE_LABEL: &str = r"/\* __next_internal_action_entry_do_not_use__ (\{[^}]+\}) \*/";

const CLIENT_DIRECTIVE: &str = "use client";
const SERVER_ACTION_DIRECTIVE: &str = "use server";

pub type RSCModuleType = &'static str;
pub const RSC_MODULE_TYPES: RSCModuleTypes = RSCModuleTypes {
  client: "client",
  server: "server",
};

pub struct RSCModuleTypes {
  pub client: RSCModuleType,
  pub server: RSCModuleType,
}

#[derive(Default)]
pub struct RSCMeta {
  pub r#type: RSCModuleType,
  pub actions: Option<Vec<String>>,
  pub action_ids: Option<std::collections::HashMap<String, String>>,
  pub client_refs: Option<Vec<String>>,
  pub client_entry_type: Option<String>,
  pub is_client_ref: bool,
}

fn get_rsc_module_information(source: &str, is_react_server_layer: bool) -> RSCMeta {
  let actions_json = regex::Regex::new(ACTION_MODULE_LABEL)
    .unwrap()
    .captures(source)
    .and_then(|caps| caps.get(1).map(|m| m.as_str()));
  let parsed_actions_meta = actions_json
    .map(|json| serde_json::from_str::<std::collections::HashMap<String, String>>(json).unwrap());
  let actions = parsed_actions_meta
    .as_ref()
    .map(|meta| meta.values().cloned().collect());

  let client_info_match = regex::Regex::new(CLIENT_MODULE_LABEL)
    .unwrap()
    .captures(source);
  let is_client_ref = client_info_match.is_some();

  if !is_react_server_layer {
    return RSCMeta {
      r#type: RSC_MODULE_TYPES.client,
      actions,
      action_ids: parsed_actions_meta,
      client_refs: None,
      client_entry_type: None,
      is_client_ref,
    };
  }

  let client_refs_string = client_info_match
    .as_ref()
    .and_then(|caps| caps.get(1).map(|m| m.as_str()));
  let client_refs = client_refs_string.map(|s| s.split(',').map(String::from).collect());
  let client_entry_type = client_info_match
    .as_ref()
    .and_then(|caps| caps.get(2).map(|m| m.as_str().to_string()));

  let r#type = if client_info_match.is_some() {
    RSC_MODULE_TYPES.client
  } else {
    RSC_MODULE_TYPES.server
  };

  RSCMeta {
    r#type,
    actions,
    action_ids: parsed_actions_meta,
    client_refs,
    client_entry_type,
    is_client_ref,
  }
}

pub struct BuildInfo {
  pub rsc: RSCMeta,
}

pub fn get_module_build_info(module: &dyn Module) -> BuildInfo {
  let is_react_server_layer = module
    .get_layer()
    .is_some_and(|layer| layer == "react-server");

  let rsc = module
    .original_source()
    .map(|s| get_rsc_module_information(s.source().as_ref(), is_react_server_layer))
    .unwrap_or_default();

  BuildInfo { rsc }
}
