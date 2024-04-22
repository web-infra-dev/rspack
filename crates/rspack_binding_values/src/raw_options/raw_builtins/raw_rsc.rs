use napi_derive::napi;
use rspack_plugin_rsc::rsc_client_entry_rspack_plugin::{
  RSCClientEntryRspackPluginOptions, ReactRoute,
};
use serde::Deserialize;
use serde::Serialize;

type ChunkName = String;
type RoutePath = String;

#[derive(Deserialize, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawReactRoute {
  pub name: ChunkName,
  pub import: RoutePath,
}

#[derive(Deserialize, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawRSCClientEntryRspackPluginOptions {
  pub routes: Option<Vec<RawReactRoute>>,
}

impl From<RawRSCClientEntryRspackPluginOptions> for RSCClientEntryRspackPluginOptions {
  fn from(value: RawRSCClientEntryRspackPluginOptions) -> Self {
    let raw_routes = value.routes.unwrap_or_default();
    let routes: Vec<ReactRoute> = raw_routes
      .into_iter()
      .map(|route| ReactRoute {
        name: route.name,
        import: route.import,
      })
      .collect();
    RSCClientEntryRspackPluginOptions { routes }
  }
}
