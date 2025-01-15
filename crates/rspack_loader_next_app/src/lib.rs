#![feature(let_chains)]

mod create_tree_code_from_path;
mod load_entrypoint;
mod options;
mod util;

use std::{
  borrow::Cow,
  path::{PathBuf, MAIN_SEPARATOR},
};

use load_entrypoint::load_next_js_template;
use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{Loader, LoaderContext, RunnerContext};
use rspack_error::{error, Result};
use rspack_loader_runner::{Identifiable, Identifier};
use rspack_paths::Utf8PathBuf;
use rspack_util::fx_hash::FxIndexMap;
use util::{normalize_app_path, normalize_underscore};

pub use crate::options::Options;

fn create_absolute_path(app_dir: &str, path_to_turn_absolute: &str) -> String {
  let path_with_os_separator = path_to_turn_absolute.replace("/", &MAIN_SEPARATOR.to_string());
  let absolute_path = path_with_os_separator.replace("private-next-app-dir", app_dir);
  absolute_path
}

pub const NEXT_APP_LOADER_IDENTIFIER: &str = "builtin:next-app-loader";

#[cacheable]
#[derive(Debug)]
pub struct NextAppLoader {
  id: Identifier,
}

impl NextAppLoader {
  pub fn new(ident: &str) -> Self {
    Self { id: ident.into() }
  }

  async fn loader_impl(&self, loader_context: &mut LoaderContext<RunnerContext>) -> Result<()> {
    let loader = &*loader_context.current_loader();
    let Some(options) = loader.query() else {
      return Ok(());
    };
    let Some(options) = options.strip_prefix('?') else {
      return Ok(());
    };

    let Options {
      name,
      page_path,
      app_dir,
      app_paths,
      page_extensions,
      base_path,
      next_config_output_path,
      middleware_config,
      project_root,
      preferred_region,
    } = serde_querystring::from_str::<Options>(options, serde_querystring::ParseMode::Duplicate)
      .map_err(|e| error!(e.to_string()))?;
    let page = name.strip_prefix("app").unwrap_or(&name);
    let app_paths = app_paths.unwrap_or_default();
    let middleware_config = json::parse(
      &String::from_utf8(
        base64_simd::STANDARD
          .decode_to_vec(middleware_config.as_bytes())
          .map_err(|e| error!(e.to_string()))?,
      )
      .map_err(|e| error!(e.to_string()))?,
    )
    .map_err(|e| error!(e.to_string()))?;

    let mut route = json::object::Object::new();
    route.insert("page", json::JsonValue::String(page.to_string()));
    route.insert(
      "absolutePagePath",
      json::JsonValue::String(create_absolute_path(&app_dir, &page_path)),
    );
    if let Some(preferred_region) = preferred_region {
      route.insert("preferredRegion", json::JsonValue::String(preferred_region));
    }
    route.insert("middlewareConfig", middleware_config);
    route.insert("relatedModules", json::JsonValue::String(page.to_string()));

    loader_context
      .extra
      .insert("route", json::JsonValue::Object(route));

    let project_root = Utf8PathBuf::from(project_root);
    let pathname = normalize_app_path(page);
    let pathname = normalize_underscore(&pathname);
    let code = load_next_js_template(
      "app-page.js",
      &project_root,
      FxIndexMap::from_iter([
        ("VAR_DEFINITION_PAGE", page.to_string()),
        ("VAR_DEFINITION_PATHNAME", pathname),
        // ("VAR_MODULE_GLOBAL_ERROR", treeCodeResult.globalError),
      ]),
      FxIndexMap::from_iter([
        ("tree", "".to_string()),
        ("pages", "".to_string()),
        ("__next_app_require__", "__webpack_require__".to_string()),
        (
          "__next_app_load_chunk__",
          "() => Promise.resolve()".to_string(),
        ),
      ]),
      FxIndexMap::default(),
    )
    .await?;

    loader_context.finish_with("true".to_string());

    Ok(())
  }
}

impl Identifiable for NextAppLoader {
  fn identifier(&self) -> rspack_loader_runner::Identifier {
    self.id
  }
}

#[cacheable_dyn]
#[async_trait::async_trait]
impl Loader<RunnerContext> for NextAppLoader {
  async fn run(&self, loader_context: &mut LoaderContext<RunnerContext>) -> Result<()> {
    // for better diagnostic, as async_trait macro don't show beautiful error message
    self.loader_impl(loader_context).await
  }
}
