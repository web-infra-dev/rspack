use std::sync::Arc;

use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_collections::Identifier;
use rspack_core::RunnerContext;
use rspack_error::{Result, ToStringResultToRspackResultExt};
use rspack_loader_runner::{Loader, LoaderContext};
use serde::{Deserialize, Serialize};
use simd_json::{
  BorrowedValue,
  base::{ValueAsArray, ValueAsObject, ValueAsScalar},
  derived::ValueTryAsArray,
};

#[derive(Debug, Deserialize, Serialize)]
pub struct ActionEntry {
  id: String,
  path: Arc<str>,
  exported_name: String,
}

pub const ACTION_ENTRY_LOADER_IDENTIFIER: &str = "builtin:action-entry-loader";

#[cacheable]
#[derive(Debug)]
#[cfg_attr(allocative, derive(allocative::Allocative))]
pub struct ActionEntryLoader {
  identifier: Identifier,
}

impl ActionEntryLoader {
  pub fn new() -> Self {
    Self {
      identifier: ACTION_ENTRY_LOADER_IDENTIFIER.into(),
    }
  }

  pub fn with_identifier<T: Into<Identifier>>(mut self, identifier: T) -> Self {
    let identifier = identifier.into();
    assert!(identifier.starts_with(ACTION_ENTRY_LOADER_IDENTIFIER));
    self.identifier = identifier;
    self
  }
}

fn parse_action_entries(object: &BorrowedValue) -> Option<Vec<ActionEntry>> {
  let tuple = object.as_array()?;
  let path: Arc<str> = Arc::from(tuple.get(0)?.as_str()?.to_string());
  let mut action_entries = vec![];
  for action_from_module in tuple.get(1)?.try_as_array().ok()? {
    if let Some(object) = action_from_module.as_object() {
      if let Some(id) = object.get("id")
        && let Some(id) = id.as_str()
        && let Some(exported_name) = object.get("exportedName")
        && let Some(exported_name) = exported_name.as_str()
      {
        action_entries.push(ActionEntry {
          id: id.to_string(),
          path: path.clone(),
          exported_name: exported_name.to_string(),
        });
      }
    }
  }
  Some(action_entries)
}

#[cacheable_dyn]
#[async_trait::async_trait]
impl Loader<RunnerContext> for ActionEntryLoader {
  fn identifier(&self) -> Identifier {
    self.identifier
  }

  #[tracing::instrument("loader:action-entry-loader", skip_all, fields(
    perfetto.track_name = "loader:action-entry-loader",
    perfetto.process_name = "Loader Analysis",
    resource =loader_context.resource(),
  ))]
  async fn run(&self, loader_context: &mut LoaderContext<RunnerContext>) -> Result<()> {
    let Some(loader_query) = loader_context.current_loader().query() else {
      loader_context.finish_with("".to_string());
      return Ok(());
    };

    let loader_options = form_urlencoded::parse(loader_query[1..].as_bytes());
    let mut individual_actions: Vec<ActionEntry> = vec![];
    for (k, v) in loader_options {
      if k == "actions" {
        let mut bytes = v.to_string().into_bytes();
        let borrowed_value = simd_json::to_borrowed_value(&mut bytes).to_rspack_result()?;
        if let Some(array) = borrowed_value.as_array() {
          for item in array.iter() {
            if let Some(entries) = parse_action_entries(item) {
              individual_actions.extend(entries);
            }
          }
        }
      }
    }

    let code = individual_actions
      .iter()
      .map(
        |ActionEntry {
           id,
           path,
           exported_name,
         }| {
          format!(
            "export {{ {} as \"{}\" }} from {}",
            exported_name,
            id,
            serde_json::to_string(path).unwrap_or_else(|_| format!("\"{}\"", path))
          )
        },
      )
      .collect::<Vec<String>>()
      .join("\n");

    loader_context.finish_with(code);

    Ok(())
  }
}
