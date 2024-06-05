use std::{
  collections::HashMap,
  path::{Path, PathBuf},
};

use indexmap::set::IndexSet;
use itertools::Itertools;
use rspack_core::{LoaderRunnerContext, Mode};
use rspack_error::Result;
use rspack_loader_runner::{Identifiable, Identifier, Loader, LoaderContext};
use serde::{Deserialize, Serialize};

use crate::{utils::shared_data::SHARED_CLIENT_IMPORTS, ReactRoute};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RSCClientEntryLoaderOptions {
  entry: HashMap<String, String>,
  root: String,
  routes: Option<Vec<ReactRoute>>,
}

#[derive(Debug)]
pub struct RSCClientEntryLoader {
  identifier: Identifier,
  options: RSCClientEntryLoaderOptions,
}

impl RSCClientEntryLoader {
  pub fn new(options: RSCClientEntryLoaderOptions) -> Self {
    Self {
      identifier: RSC_CLIENT_ENTRY_LOADER_IDENTIFIER.into(),
      options: options.into(),
    }
  }

  pub fn get_routes_code(&self) -> String {
    if let Some(routes) = self.options.routes.as_ref() {
      let code = routes
        .iter()
        .map(|f| {
          format!(
            r#"import(/* webpackChunkName: "{}" */ "{}")"#,
            f.name, f.import
          )
        })
        .join("\n");
      code
    } else {
      String::from("")
    }
  }

  pub fn get_entry_chunk_name(&self, resource_path: &str) -> Option<String> {
    let result = self
      .options
      .entry
      .clone()
      .into_iter()
      .find(|(_, path)| path == resource_path);
    let chunk_name = if let Some(result) = result {
      let resolved_name = if result.0 == "client-entry" {
        String::from("server-entry")
      } else {
        result.0
      };
      Some(resolved_name)
    } else {
      None
    };
    chunk_name
  }

  pub fn get_route_chunk_name(&self, resource_path: &str) -> Option<String> {
    if let Some(routes) = self.options.routes.as_ref() {
      let route = routes.into_iter().find(|f| f.import == resource_path);
      let chunk_name = if let Some(route) = route {
        Some(route.name.clone())
      } else {
        None
      };
      chunk_name
    } else {
      None
    }
  }

  pub fn get_client_imports_by_name(&self, chunk_name: &str) -> Option<IndexSet<String>> {
    let all_client_imports = &SHARED_CLIENT_IMPORTS.lock().unwrap();
    let client_imports = all_client_imports.get(&String::from(chunk_name)).cloned();
    client_imports
  }

  pub fn format_client_imports(
    &self,
    entry_chunk_name: Option<&String>,
    route_chunk_name: Option<&String>,
  ) -> Option<PathBuf> {
    let chunk_name = entry_chunk_name.or(route_chunk_name);
    if let Some(chunk_name) = chunk_name {
      let file_name = format!("[{}]_client_imports.json", chunk_name);
      Some(Path::new(&self.options.root).join(file_name))
    } else {
      None
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
impl Loader<LoaderRunnerContext> for RSCClientEntryLoader {
  async fn run(&self, loader_context: &mut LoaderContext<'_, LoaderRunnerContext>) -> Result<()> {
    let content = std::mem::take(&mut loader_context.content).expect("Content should be available");
    let resource_path = loader_context.resource_path().to_str();
    let mut source = content.try_into_string()?;

    if let Some(resource_path) = resource_path {
      let chunk_name = self.get_entry_chunk_name(resource_path);
      let route_chunk_name = self.get_route_chunk_name(resource_path);
      let client_imports_path =
        self.format_client_imports(chunk_name.as_ref(), route_chunk_name.as_ref());
      let mut hmr = String::from("");
      let development =
        Some(Mode::is_development(&loader_context.context.options.mode)).unwrap_or(false);
      if development {
        if let Some(client_imports_path) = client_imports_path {
          // HMR
          hmr = format!(r#"import {:?};"#, client_imports_path.into_os_string())
        }
      }

      // Entrypoint
      if let Some(chunk_name) = &chunk_name {
        let client_imports = self.get_client_imports_by_name(chunk_name);

        if let Some(client_imports) = client_imports {
          let code = client_imports
            .iter()
            .map(|i| format!(r#"import(/* webpackMode: "eager" */ "{}");"#, i))
            .join("\n");
          source = format!("{}{}", code, source);
        }
        let routes = self.get_routes_code();
        source = format!("{}{}{}", hmr, routes, source);
      }
      // Route
      if let Some(chunk_name) = &route_chunk_name {
        let client_imports = self.get_client_imports_by_name(chunk_name);

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
          source = format!("{}{}{}", hmr, code, source);
        }
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
