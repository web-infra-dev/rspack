use indoc::formatdoc;
use rspack_core::{Module, NormalModule, RscModuleType};
use rspack_error::{Result, ToStringResultToRspackResultExt};
use swc::atoms::Wtf8Atom;

fn to_cjs_server_entry(resource: &str, server_refs: &[Wtf8Atom]) -> String {
  let mut cjs_source =
    "const { createServerEntry } = require(\"react-server-dom-rspack/server\");\n".to_string();

  for server_ref in server_refs {
    match server_ref.as_str() {
      Some("default") => {
        cjs_source.push_str(&formatdoc! {
          r#"
            const _default = require("{}?skip-rsc-transform");
            module.exports = createServerEntry(
              _default,
              "{}",
            );
          "#,
          resource,
          resource
        });
      }
      Some(ident) => {
        cjs_source.push_str(&formatdoc! {
          r#"
            const _original_{ident} = require("{resource}?skip-rsc-transform").{ident};
            exports.{ident} = createServerEntry(
              _original_{ident},
              "{resource}",
            );
          "#,
          ident = ident,
          resource = resource
        });
      }
      _ => {}
    }
  }
  cjs_source
}

fn to_esm_server_entry(resource: &str, server_refs: &[Wtf8Atom]) -> String {
  let mut esm_source =
    "import { createServerEntry } from \"react-server-dom-rspack/server\";\n".to_string();

  for server_ref in server_refs {
    match server_ref.as_str() {
      Some("default") => {
        esm_source.push_str(&formatdoc! {
          r#"
            import _default from "{}?skip-rsc-transform";
            export default createServerEntry(
              _default,
              "{}",
            )
          "#,
          resource,
          resource
        });
      }
      Some(ident) => {
        esm_source.push_str(&formatdoc! {
          r#"
            import {{ {ident} as _original_{ident} }} from "{resource}?skip-rsc-transform";
            export const {ident} = createServerEntry(
              _original_{ident},
              "{resource}",
            )
          "#,
          ident = ident,
          resource = resource,
        });
      }
      _ => {}
    }
  }
  esm_source
}

fn to_esm_client_entry(resource: &str, client_refs: &[Wtf8Atom]) -> Result<String> {
  let mut esm_source =
    String::from("import { registerClientReference } from \"react-server-dom-rspack/server\"\n");

  let call_error = format!(
    "Attempted to call the default export of {} from \
    the server, but it's on the client. It's not possible to invoke a \
    client function from the server, it can only be rendered as a \
    Component or passed to props of a Client Component.",
    serde_json::to_string(resource).to_rspack_result()?
  );

  for client_ref in client_refs {
    match client_ref.as_str() {
      Some("default") => {
        esm_source.push_str(&formatdoc! {
          r#"
            export default registerClientReference(
            function() {{ throw new Error({call_error}) }},
              "{resource}",
              "default",
            )
          "#,
          resource = resource,
          call_error = serde_json::to_string(&call_error).to_rspack_result()?
        });
      }
      Some(ident) => {
        esm_source.push_str(&formatdoc! {
          r#"
            export const {ident} = registerClientReference(
            function() {{ throw new Error({call_error}) }},
              "{resource}",
              "{ident}",
            )
          "#,
          ident = ident,
          resource = resource,
          call_error = serde_json::to_string(&call_error).to_rspack_result()?
        });
      }
      _ => {}
    }
  }
  Ok(esm_source)
}

fn to_cjs_client_entry(resource: &str, client_refs: &[Wtf8Atom]) -> Result<String> {
  let mut cjs_source = String::from(
    "const { registerClientReference } = require(\"react-server-dom-rspack/server\");\n",
  );

  let call_error = format!(
    "Attempted to call the default export of {} from \
    the server, but it's on the client. It's not possible to invoke a \
    client function from the server, it can only be rendered as a \
    Component or passed to props of a Client Component.",
    serde_json::to_string(resource).to_rspack_result()?
  );

  for client_ref in client_refs {
    match client_ref.as_str() {
      Some("default") => {
        cjs_source.push_str(&formatdoc! {
          r#"
            module.exports = registerClientReference(
              function() {{ throw new Error({call_error}) }},
              "{resource}",
              "default",
            );
          "#,
          resource = resource,
          call_error = serde_json::to_string(&call_error).to_rspack_result()?
        });
      }
      Some(ident) => {
        cjs_source.push_str(&formatdoc! {
          r#"
            exports.{ident} = registerClientReference(
              function() {{ throw new Error({call_error}) }},
              "{resource}",
              "{ident}",
            );
          "#,
          ident = ident,
          resource = resource,
          call_error = serde_json::to_string(&call_error).to_rspack_result()?
        });
      }
      _ => {}
    }
  }
  Ok(cjs_source)
}

pub fn to_module_ref(module: &NormalModule) -> Result<Option<String>> {
  let is_react_server_layer = module
    .get_layer()
    .is_some_and(|layer| layer == "react-server-components");
  if !is_react_server_layer {
    return Ok(None);
  }

  let Some(rsc) = module.build_info().rsc.as_ref() else {
    return Ok(None);
  };

  let resource = module.resource_resolved_data().resource();

  if rsc.module_type.contains(RscModuleType::ServerEntry) {
    if rsc
      .server_refs
      .iter()
      .any(|server_ref| server_ref.as_str() == Some("*"))
    {
      return Err(rspack_error::error!(
        r#"It's currently unsupported to use "export *" in a server entry. Please use named exports instead."#
      ));
    }
    if rsc.is_cjs {
      return Ok(Some(to_cjs_server_entry(resource, &rsc.server_refs)));
    } else {
      return Ok(Some(to_esm_server_entry(resource, &rsc.server_refs)));
    }
  }

  if rsc.module_type.contains(RscModuleType::Client) {
    if rsc
      .client_refs
      .iter()
      .any(|client_ref| client_ref.as_str() == Some("*"))
    {
      return Err(rspack_error::error!(
        r#"It's currently unsupported to use "export *" in a client boundary. Please use named exports instead."#
      ));
    }
    if rsc.is_cjs {
      return Ok(Some(to_cjs_client_entry(resource, &rsc.client_refs)?));
    } else {
      return Ok(Some(to_esm_client_entry(resource, &rsc.client_refs)?));
    }
  }

  Ok(None)
}
