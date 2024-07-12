use napi_derive::napi;
use rspack_plugin_rsc::rsc_client_entry_rspack_plugin::RSCClientEntryRspackPluginOptions;
use rspack_plugin_rsc::rsc_client_reference_manifest_rspack_plugin::RSCClientReferenceManifestRspackPluginOptions;
use rspack_plugin_rsc::ReactRoute;
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

#[derive(Deserialize, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawRSCClientReferenceManifestRspackPluginOptions {
  pub routes: Option<Vec<RawReactRoute>>,
}

impl From<RawRSCClientReferenceManifestRspackPluginOptions>
  for RSCClientReferenceManifestRspackPluginOptions
{
  fn from(value: RawRSCClientReferenceManifestRspackPluginOptions) -> Self {
    let raw_routes = value.routes.unwrap_or_default();
    let routes: Vec<ReactRoute> = raw_routes
      .into_iter()
      .map(|route| ReactRoute {
        name: route.name,
        import: route.import,
      })
      .collect();
    RSCClientReferenceManifestRspackPluginOptions { routes }
  }
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