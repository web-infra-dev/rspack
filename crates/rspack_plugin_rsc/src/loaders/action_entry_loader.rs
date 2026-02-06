use std::sync::Arc;

use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_collections::Identifier;
use rspack_core::RunnerContext;
use rspack_error::{Result, ToStringResultToRspackResultExt};
use rspack_loader_runner::{Loader, LoaderContext};
use serde::{Deserialize, Serialize};
use simd_json::base::{ValueAsArray, ValueAsObject, ValueAsScalar};

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct ActionEntry {
  pub id: String,
  pub path: Arc<str>,
  pub exported_name: String,
}

pub(crate) const ACTION_ENTRY_LOADER_IDENTIFIER: &str = "builtin:rsc-action-entry-loader";

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

pub(crate) fn parse_action_entries(v: String) -> Result<Option<Vec<ActionEntry>>> {
  let mut action_entries = vec![];

  let mut bytes = v.into_bytes();
  let borrowed_value = simd_json::to_borrowed_value(&mut bytes).to_rspack_result()?;
  if let Some(object) = borrowed_value.as_object() {
    for (path, value) in object.iter() {
      let Some(item) = value.as_array() else {
        return Ok(None);
      };
      let path: Arc<str> = Arc::from(path.to_string());
      for tuple in item {
        let Some(tuple) = tuple.as_array() else {
          return Ok(None);
        };
        let id = match tuple.first().and_then(|v| v.as_str()) {
          Some(id) => id,
          None => continue,
        };
        let exported_name = match tuple.get(1).and_then(|v| v.as_str()) {
          Some(v) => v,
          None => continue,
        };
        action_entries.push(ActionEntry {
          id: id.to_string(),
          path: path.clone(),
          exported_name: exported_name.to_string(),
        });
      }
    }
  }
  Ok(Some(action_entries))
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
    resource = loader_context.resource(),
  ))]
  async fn run(&self, loader_context: &mut LoaderContext<RunnerContext>) -> Result<()> {
    let Some(loader_query) = loader_context.current_loader().query() else {
      loader_context.finish_with(String::new());
      return Ok(());
    };

    let loader_options = form_urlencoded::parse(&loader_query.as_bytes()[1..]);
    let mut individual_actions: Vec<ActionEntry> = vec![];
    for (k, v) in loader_options {
      if k == "actions" {
        individual_actions = parse_action_entries(v.into_owned())?.unwrap_or_default();
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
          Ok(format!(
            "export {{ {} as \"{}\" }} from {}",
            exported_name,
            id,
            serde_json::to_string(path).to_rspack_result()?
          ))
        },
      )
      .collect::<Result<Vec<String>>>()?
      .join("\n");

    loader_context.finish_with(code);

    Ok(())
  }
}
