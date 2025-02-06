use std::{
  collections::HashMap,
  path::{Path, PathBuf},
};

use indexmap::IndexSet;
use itertools::Itertools;
use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::RunnerContext;
use rspack_error::Result;
use rspack_loader_runner::{Identifiable, Identifier, Loader, LoaderContext};
use serde::{Deserialize, Serialize};
use url::form_urlencoded;

use crate::utils::{
  constants::RSC_SERVER_ACTION_ENTRY_RE,
  server_action::generate_action_id,
  shared_data::{SHARED_DATA, SHARED_SERVER_IMPORTS},
};

#[cacheable]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RSCServerActionServerLoaderOptions {
  root: String,
}

#[cacheable]
#[derive(Debug)]
pub struct RSCServerActionServerLoader {
  identifier: Identifier,
  options: RSCServerActionServerLoaderOptions,
}

#[derive(Debug, Clone, Default)]
struct QueryParsedRequest {
  pub chunk_name: String,
}

impl RSCServerActionServerLoader {
  pub fn new(options: RSCServerActionServerLoaderOptions) -> Self {
    Self {
      identifier: RSC_SERVER_ACTION_SERVER_LOADER_IDENTIFIER.into(),
      options: options.into(),
    }
  }

  async fn get_server_imports_by_name(&self, chunk_name: &str) -> Option<IndexSet<String>> {
    let all_server_imports = &SHARED_SERVER_IMPORTS.read().await;
    let server_imports: Option<_> = all_server_imports.get(&String::from(chunk_name)).cloned();
    server_imports
  }

  pub async fn get_server_refs_by_name(&self, chunk_name: &str) -> Vec<Vec<String>> {
    let server_imports = self.get_server_imports_by_name(chunk_name).await;
    let all_server_refs = &SHARED_DATA.read().await;
    let mut actions = vec![];
    if let Some(server_imports) = server_imports {
      for file_path in server_imports.iter() {
        let server_refs: Option<_> = all_server_refs
          .server_imports
          .get(&String::from(file_path))
          .cloned();
        if let Some(server_refs) = server_refs {
          for n in server_refs.names.iter() {
            let id = generate_action_id(file_path.as_str(), n);
            let item = vec![id.to_string(), file_path.to_string(), n.to_string()];
            actions.push(item);
          }
        }
      }
    }
    actions
  }

  pub fn format_server_ref_path(&self) -> PathBuf {
    let file_name = String::from("server-reference-manifest.json");
    Path::new(&self.options.root).join(file_name)
  }

  pub fn is_match(&self, resource_path: Option<&str>) -> bool {
    if let Some(resource_path) = resource_path {
      RSC_SERVER_ACTION_ENTRY_RE.is_match(resource_path)
    } else {
      false
    }
  }

  fn parse_query(&self, query: Option<&str>) -> QueryParsedRequest {
    if let Some(query) = query {
      let hash_query: HashMap<_, _> =
        form_urlencoded::parse(query.trim_start_matches('?').as_bytes())
          .into_owned()
          .collect();
      QueryParsedRequest {
        chunk_name: String::from(hash_query.get("name").unwrap_or(&String::from(""))),
      }
    } else {
      QueryParsedRequest::default()
    }
  }

  /// Panics:
  /// Panics if `identifier` passed in is not starting with `builtin:rsc-server-action-server-loader`.
  pub fn with_identifier(mut self, identifier: Identifier) -> Self {
    assert!(identifier.starts_with(RSC_SERVER_ACTION_SERVER_LOADER_IDENTIFIER));
    self.identifier = identifier;
    self
  }
}

pub const RSC_SERVER_ACTION_SERVER_LOADER_IDENTIFIER: &str =
  "builtin:rsc-server-action-server-loader";

#[cacheable_dyn]
#[async_trait::async_trait]
impl Loader<RunnerContext> for RSCServerActionServerLoader {
  async fn run(&self, loader_context: &mut LoaderContext<RunnerContext>) -> Result<()> {
    let Some(content) = loader_context.take_content() else {
      return Ok(());
    };
    let mut source = content.try_into_string()?;
    let resource_path = loader_context
      .resource_path()
      .and_then(|f| Some(f.as_str()));
    let query = loader_context.resource_query();

    if self.is_match(resource_path) {
      let parsed = self.parse_query(query);
      let chunk_name = parsed.chunk_name;
      let server_ref_path = self.format_server_ref_path();
      loader_context
        .missing_dependencies
        .insert(server_ref_path.clone());
      let server_refs = self.get_server_refs_by_name(&chunk_name).await;
      let actions = server_refs
        .iter()
        .map(|f| {
          return format!(
            r#""{}": async () => import(/* webpackMode: "eager" */ "{}").then(mod => mod["{}"]),"#,
            f[0], f[1], f[2]
          );
        })
        .join("\n");
      source = format!(
        r#"
        const actions = {{{}}};
        export default actions
        "#,
        actions
      );
    }

    loader_context.finish_with(source);
    Ok(())
  }
}

impl Identifiable for RSCServerActionServerLoader {
  fn identifier(&self) -> Identifier {
    self.identifier
  }
}
