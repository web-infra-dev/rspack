use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_collections::Identifier;
use rspack_core::RunnerContext;
use rspack_error::{Result, ToStringResultToRspackResultExt};
use rspack_loader_runner::{Loader, LoaderContext};
use serde::{Deserialize, Serialize};
use simd_json::{
  BorrowedValue,
  base::{ValueAsObject, ValueAsScalar},
  derived::ValueTryAsArray,
};

use crate::constants::REGEX_CSS;

#[derive(Debug, Deserialize, Serialize)]
pub struct ClientEntry {
  request: String,
  ids: Vec<String>,
}

pub const CLIENT_ENTRY_LOADER_IDENTIFIER: &str = "builtin:client-entry-loader";

#[cacheable]
#[derive(Debug)]
#[cfg_attr(allocative, derive(allocative::Allocative))]
pub struct ClientEntryLoader {
  identifier: Identifier,
}

impl ClientEntryLoader {
  pub fn new() -> Self {
    Self {
      identifier: CLIENT_ENTRY_LOADER_IDENTIFIER.into(),
    }
  }

  pub fn with_identifier<T: Into<Identifier>>(mut self, identifier: T) -> Self {
    let identifier = identifier.into();
    assert!(identifier.starts_with(CLIENT_ENTRY_LOADER_IDENTIFIER));
    self.identifier = identifier;
    self
  }
}

fn parse_client_entry(object: &BorrowedValue) -> Option<ClientEntry> {
  let object = object.as_object()?;
  let request = object.get("request")?.as_str()?.to_string();
  let ids_array = object.get("ids")?.try_as_array().ok()?;
  let ids = ids_array
    .iter()
    .filter_map(|id_value| id_value.as_str().map(String::from))
    .collect::<Vec<String>>();
  Some(ClientEntry { request, ids })
}

#[cacheable_dyn]
#[async_trait::async_trait]
impl Loader<RunnerContext> for ClientEntryLoader {
  fn identifier(&self) -> Identifier {
    self.identifier
  }

  #[tracing::instrument("loader:client-entry-loader", skip_all, fields(
    perfetto.track_name = "loader:client-entry-loader",
    perfetto.process_name = "Loader Analysis",
    resource = loader_context.resource(),
  ))]
  async fn run(&self, loader_context: &mut LoaderContext<RunnerContext>) -> Result<()> {
    let Some(loader_query) = loader_context.current_loader().query() else {
      loader_context.finish_with("".to_string());
      return Ok(());
    };

    let loader_options = form_urlencoded::parse(loader_query[1..].as_bytes());
    let mut modules: Vec<ClientEntry> = vec![];
    let mut is_server: bool = false;
    for (k, v) in loader_options {
      if k == "modules" {
        let mut bytes = v.to_string().into_bytes();
        let borrowed_value = simd_json::to_borrowed_value(&mut bytes).to_rspack_result()?;
        match borrowed_value.try_as_array() {
          Ok(array) => {
            for item in array.iter() {
              if let Some(component) = parse_client_entry(item) {
                modules.push(component);
              }
            }
          }
          Err(_) => {
            if let Some(component) = parse_client_entry(&borrowed_value) {
              modules.push(component);
            }
          }
        }
      } else if k == "server" {
        if v == "true" {
          is_server = true;
        }
      }
    }

    let code = modules
      .iter()
      .filter(|client_component| {
        if is_server {
          !REGEX_CSS.is_match(&client_component.request)
        } else {
          true
        }
      })
      .map(|client_component| {
        // When we cannot determine the export names, we use eager mode to include the whole module.
        // Otherwise, we use eager mode with webpackExports to only include the necessary exports.
        // If we have '*' in the ids, we include all the imports
        let import_path = simd_json::to_string(&client_component.request).unwrap();
        if client_component.ids.len() == 0 || client_component.ids.iter().any(|id| id == "*") {
          if is_server {
            format!("import(/* webpackMode: \"eager\" */ {});\n", import_path)
          } else {
            format!("import({});\n", import_path)
          }
        } else {
          let webpack_exports = simd_json::to_string(&client_component.ids).unwrap();

          if is_server {
            format!(
              "import(/* webpackMode: \"eager\" */ /* webpackExports: {} */ {});\n",
              webpack_exports, import_path
            )
          } else {
            format!(
              "import(/* webpackExports: {} */ {});\n",
              webpack_exports, import_path
            )
          }
        }
      })
      .collect::<Vec<String>>()
      .join("\n");

    loader_context.finish_with(code);

    Ok(())
  }
}
