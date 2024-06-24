use std::{
  collections::HashMap,
  path::{Path, PathBuf},
};

use indexmap::set::IndexSet;
use itertools::Itertools;
use once_cell::sync::Lazy;
use regex::Regex;
use rspack_core::{Mode, RunnerContext};
use rspack_error::Result;
use rspack_loader_runner::{Identifiable, Identifier, Loader, LoaderContext};
use serde::{Deserialize, Serialize};
use url::form_urlencoded;

use crate::utils::shared_data::SHARED_CLIENT_IMPORTS;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RSCClientEntryLoaderOptions {
  root: String,
}

#[derive(Debug)]
pub struct RSCClientEntryLoader {
  identifier: Identifier,
  options: RSCClientEntryLoaderOptions,
}
#[derive(Debug, Clone, Default)]
struct QueryParsedRequest {
  pub is_client_entry: bool,
  pub is_route_entry: bool,
  pub chunk_name: String,
}

static RSC_CLIENT_ENTRY_RE: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"rsc-client-entry-loader").expect("regexp init failed"));

impl RSCClientEntryLoader {
  pub fn new(options: RSCClientEntryLoaderOptions) -> Self {
    Self {
      identifier: RSC_CLIENT_ENTRY_LOADER_IDENTIFIER.into(),
      options: options.into(),
    }
  }

  pub fn get_client_imports_by_name(&self, chunk_name: &str) -> Option<IndexSet<String>> {
    let all_client_imports = &SHARED_CLIENT_IMPORTS.lock().unwrap();
    let client_imports = all_client_imports.get(&String::from(chunk_name)).cloned();
    client_imports
  }

  pub fn format_client_imports(&self, chunk_name: &str) -> Option<PathBuf> {
    let file_name = format!("[{}]_client_imports.json", chunk_name);
    Some(Path::new(&self.options.root).join(file_name))
  }

  fn parse_query(&self, query: Option<&str>) -> QueryParsedRequest {
    if let Some(query) = query {
      let hash_query: HashMap<_, _> =
        form_urlencoded::parse(query.trim_start_matches('?').as_bytes())
          .into_owned()
          .collect();
      QueryParsedRequest {
        chunk_name: String::from(hash_query.get("name").unwrap_or(&String::from(""))),
        is_client_entry: hash_query
          .get("from")
          .unwrap_or(&String::from(""))
          .eq("client-entry"),
        is_route_entry: hash_query
          .get("from")
          .unwrap_or(&String::from(""))
          .eq("route-entry"),
      }
    } else {
      QueryParsedRequest::default()
    }
  }

  pub fn is_match(&self, resource_path: Option<&str>) -> bool {
    if let Some(resource_path) = resource_path {
      RSC_CLIENT_ENTRY_RE.is_match(resource_path)
    } else {
      false
    }
  }

  /// Panics:
  /// Panics if `identifier` passed in is not starting with `builtin:swc-loader`.
  pub fn with_identifier(mut self, identifier: Identifier) -> Self {
    assert!(identifier.starts_with(RSC_CLIENT_ENTRY_LOADER_IDENTIFIER));
    self.identifier = identifier;
    self
  }
}

pub const RSC_CLIENT_ENTRY_LOADER_IDENTIFIER: &str = "builtin:rsc-client-entry-loader";

#[async_trait::async_trait]
impl Loader<RunnerContext> for RSCClientEntryLoader {
  async fn run(&self, loader_context: &mut LoaderContext<RunnerContext>) -> Result<()> {
    let content = std::mem::take(&mut loader_context.content).expect("Content should be available");
    let resource_path = loader_context.resource_path().to_str();
    let mut source = content.try_into_string()?;
    let query = loader_context.resource_query();

    if self.is_match(resource_path) {
      let parsed: QueryParsedRequest = self.parse_query(query);
      let chunk_name = parsed.chunk_name;
      let is_client_entry = parsed.is_client_entry;
      let is_route_entry = parsed.is_route_entry;
      let mut hmr = String::from("");
      let development =
        Some(Mode::is_development(&loader_context.context.options.mode)).unwrap_or(false);
      let client_imports_path = self.format_client_imports(&chunk_name);

      if development {
        if let Some(client_imports_path) = client_imports_path {
          // HMR
          if !client_imports_path.exists() {
            loader_context
              .missing_dependencies
              .insert(client_imports_path.clone());
          } else {
            // If client_imports.json not found, connect resource with client_imports.json will throw resolve error
            hmr = format!(r#"import {:?};"#, client_imports_path.into_os_string());
          }
        }
      }
      // Entrypoint
      if is_client_entry {
        let client_imports = self.get_client_imports_by_name(&chunk_name);
        if let Some(client_imports) = client_imports {
          let code = client_imports
            .iter()
            .map(|i| format!(r#"import(/* webpackMode: "eager" */ "{}");"#, i))
            .join("\n");
          source = format!("{}{}", code, source);
        }
        source = format!("{}{}", hmr, source);
      }

      // Route
      if is_route_entry {
        let client_imports = self.get_client_imports_by_name(&chunk_name);
        if let Some(client_imports) = client_imports {
          let code = client_imports
            .iter()
            .map(|i| {
              format!(
                r#"import(/* webpackChunkName: "{}" */ "{}");"#,
                chunk_name, i
              )
            })
            .join("\n");
          source = format!("{}{}", code, source);
        }
        source = format!("{}{}", hmr, source);
      }
    }
    loader_context.content = Some(source.into());
    Ok(())
  }
}

impl Identifiable for RSCClientEntryLoader {
  fn identifier(&self) -> Identifier {
    self.identifier
  }
}
