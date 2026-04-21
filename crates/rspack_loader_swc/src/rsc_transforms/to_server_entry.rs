use indoc::formatdoc;
use rspack_core::{Module as RspackCoreModule, NormalModule, RscModuleType};
use rspack_error::Result;
use rspack_util::json_stringify_str;
use swc::atoms::Wtf8Atom;

fn to_cjs_server_entry(resource: &str, server_refs: &[Wtf8Atom]) -> Result<String> {
  let mut cjs_source =
    "const { createServerEntry } = require(\"react-server-dom-rspack/server\");\n".to_string();

  for server_ref in server_refs {
    match server_ref.as_str() {
      Some("default") => {
        cjs_source.push_str(&formatdoc! {
          r#"
            module.exports = createServerEntry(
              require({request}),
              {resource}
            );
          "#,
          request = json_stringify_str(&format!("{resource}?rsc-server-entry-proxy=true")),
          resource = json_stringify_str(resource)
        });
      }
      Some(ident) => {
        cjs_source.push_str(&formatdoc! {
          r#"
            exports.{ident} = createServerEntry(
              require({request}).{ident},
              {resource}
            );
          "#,
          ident = ident,
          request = json_stringify_str(&format!("{resource}?rsc-server-entry-proxy=true")),
          resource = json_stringify_str(resource)
        });
      }
      _ => {}
    }
  }

  Ok(cjs_source)
}

fn to_esm_server_entry(resource: &str, server_refs: &[Wtf8Atom]) -> Result<String> {
  let mut esm_source =
    "import { createServerEntry } from \"react-server-dom-rspack/server\";\n".to_string();

  for server_ref in server_refs {
    match server_ref.as_str() {
      Some("default") => {
        esm_source.push_str(&formatdoc! {
          r#"
            import _default from {request};
            export default createServerEntry(
              _default,
              {resource}
            );
          "#,
          request = json_stringify_str(&format!("{resource}?rsc-server-entry-proxy=true")),
          resource = json_stringify_str(resource)
        });
      }
      Some(ident) => {
        esm_source.push_str(&formatdoc! {
          r#"
            import {{ {ident} as _original_{ident} }} from {request};
            export const {ident} = createServerEntry(
              _original_{ident},
              {resource}
            );
          "#,
          ident = ident,
          request = json_stringify_str(&format!("{resource}?rsc-server-entry-proxy=true")),
          resource = json_stringify_str(resource)
        });
      }
      _ => {}
    }
  }

  Ok(esm_source)
}

pub fn to_server_entry(module: &NormalModule) -> Result<Option<String>> {
  if module
    .get_layer()
    .is_none_or(|layer| layer != "react-server-components")
  {
    return Ok(None);
  }

  let Some(rsc) = module.build_info().rsc.as_ref() else {
    return Ok(None);
  };

  match rsc.module_type {
    RscModuleType::ServerEntry => {
      if rsc
        .server_refs
        .iter()
        .any(|server_ref| server_ref.as_str() == Some("*"))
      {
        return Err(rspack_error::error!(
          r#"It's currently unsupported to use "export *" in a server entry. Please use named exports instead."#
        ));
      }

      Ok(Some(if rsc.is_cjs {
        to_cjs_server_entry(module.resource_resolved_data().resource(), &rsc.server_refs)?
      } else {
        to_esm_server_entry(module.resource_resolved_data().resource(), &rsc.server_refs)?
      }))
    }
    RscModuleType::Client => {
      if rsc
        .client_refs
        .iter()
        .any(|client_ref| client_ref.as_str() == Some("*"))
      {
        Err(rspack_error::error!(
          r#"It's currently unsupported to use "export *" in a client boundary. Please use named exports instead."#
        ))
      } else {
        Ok(None)
      }
    }
    _ => Ok(None),
  }
}
