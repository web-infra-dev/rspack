use std::collections::HashMap;
use std::path::PathBuf;

use napi::bindgen_prelude::*;
use napi::NapiRaw;
use rspack_core::rspack_sources::BoxSource;
use rspack_core::AssetInfo;
use rspack_core::{rspack_sources::SourceExt, AstOrSource, NormalModuleAstOrSource};
use rspack_identifier::Identifier;
use rspack_napi_shared::NapiResultExt;

use super::module::ToJsModule;
use super::PathWithInfo;
use crate::{
  js_values::{chunk::JsChunk, module::JsModule, PathData},
  CompatSource, JsAsset, JsAssetInfo, JsChunkGroup, JsCompatSource, JsStats, ToJsCompatSource,
};

#[napi]
pub struct JsCompilation {
  inner: &'static mut rspack_core::Compilation,
}

#[napi]
impl JsCompilation {
  #[napi(
    ts_args_type = r#"filename: string, newSourceOrFunction: JsCompatSource | ((source: JsCompatSource) => JsCompatSource), assetInfoUpdateOrFunction?: JsAssetInfo | ((assetInfo: JsAssetInfo) => JsAssetInfo)"#
  )]
  pub fn update_asset(
    &mut self,
    env: Env,
    filename: String,
    new_source_or_function: Either<JsCompatSource, JsFunction>,
    asset_info_update_or_function: Option<Either<JsAssetInfo, JsFunction>>,
  ) -> Result<()> {
    self
      .inner
      .update_asset(&filename, |original_source, original_info| {
        let new_source: napi::Result<BoxSource> = try {
          let new_source = match new_source_or_function {
            Either::A(new_source) => Into::<CompatSource>::into(new_source).boxed(),
            Either::B(new_source_fn) => {
              let js_source = unsafe {
                call_js_function_with_napi_objects!(
                  env,
                  new_source_fn,
                  original_source.to_js_compat_source()
                )
              }?;

              let compat_source: CompatSource = unsafe {
                convert_raw_napi_value_to_napi_value!(env, JsCompatSource, js_source.raw())
              }?
              .into();

              compat_source.boxed()
            }
          };
          new_source
        };
        let new_source = new_source.into_rspack_result()?;

        let new_info: napi::Result<Option<AssetInfo>> = asset_info_update_or_function
          .map(
            |asset_info_update_or_function| match asset_info_update_or_function {
              Either::A(asset_info) => Ok(asset_info.into()),
              Either::B(asset_info_fn) => {
                let asset_info = unsafe {
                  call_js_function_with_napi_objects!(
                    env,
                    asset_info_fn,
                    Into::<JsAssetInfo>::into(original_info.clone())
                  )
                }?;

                let js_asset_info = unsafe {
                  convert_raw_napi_value_to_napi_value!(env, JsAssetInfo, asset_info.raw())
                }?;
                Ok(js_asset_info.into())
              }
            },
          )
          .transpose();
        let new_info = new_info.into_rspack_result()?;
        Ok((new_source, new_info.unwrap_or(original_info)))
      })
      .map_err(|err| napi::Error::from_reason(err.to_string()))
  }

  #[napi(ts_return_type = "Readonly<JsAsset>[]")]
  pub fn get_assets(&self) -> Result<Vec<JsAsset>> {
    let mut assets = Vec::<JsAsset>::with_capacity(self.inner.assets().len());

    for (filename, asset) in self.inner.assets() {
      assets.push(JsAsset {
        name: filename.clone(),
        source: asset
          .source
          .as_ref()
          .map(|s| s.to_js_compat_source())
          .transpose()?,
        info: asset.info.clone().into(),
      });
    }

    Ok(assets)
  }

  #[napi]
  pub fn get_asset(&self, name: String) -> Result<Option<JsAsset>> {
    match self.inner.assets().get(&name) {
      Some(asset) => Ok(Some(JsAsset {
        name,
        source: asset
          .source
          .as_ref()
          .map(|s| s.to_js_compat_source())
          .transpose()?,
        info: asset.info.clone().into(),
      })),
      None => Ok(None),
    }
  }

  #[napi]
  pub fn get_asset_source(&self, name: String) -> Result<Option<JsCompatSource>> {
    self
      .inner
      .assets()
      .get(&name)
      .and_then(|v| v.source.as_ref().map(|s| s.to_js_compat_source()))
      .transpose()
  }

  #[napi]
  pub fn get_modules(&self) -> Vec<JsModule> {
    self
      .inner
      .module_graph
      .modules()
      .values()
      .filter_map(|module| module.to_js_module().ok())
      .collect::<Vec<_>>()
  }

  #[napi]
  pub fn get_chunks(&self) -> Vec<JsChunk> {
    self
      .inner
      .chunk_by_ukey
      .values()
      .map(JsChunk::from)
      .collect::<Vec<_>>()
  }

  #[napi]
  /// Only available for those none Js and Css source,
  /// return true if set module source successfully, false if failed.
  pub fn set_none_ast_module_source(
    &mut self,
    module_identifier: String,
    source: JsCompatSource,
  ) -> bool {
    match self
      .inner
      .module_graph
      .module_by_identifier_mut(&Identifier::from(module_identifier.as_str()))
    {
      Some(module) => match module.as_normal_module_mut() {
        Some(module) => {
          let compat_source = CompatSource::from(source).boxed();
          *module.ast_or_source_mut() =
            NormalModuleAstOrSource::new_built(AstOrSource::Source(compat_source), &[]);
          true
        }
        None => false,
      },
      None => false,
    }
  }

  #[napi]
  pub fn set_asset_source(&mut self, name: String, source: JsCompatSource) {
    let source = CompatSource::from(source).boxed();
    match self.inner.assets_mut().entry(name) {
      std::collections::hash_map::Entry::Occupied(mut e) => e.get_mut().set_source(Some(source)),
      std::collections::hash_map::Entry::Vacant(e) => {
        e.insert(rspack_core::CompilationAsset::from(source));
      }
    };
  }

  #[napi]
  pub fn delete_asset_source(&mut self, name: String) {
    self
      .inner
      .assets_mut()
      .entry(name)
      .and_modify(|a| a.set_source(None));
  }

  #[napi]
  pub fn get_asset_filenames(&self) -> Result<Vec<String>> {
    let filenames = self
      .inner
      .assets()
      .iter()
      .filter(|(_, asset)| asset.get_source().is_some())
      .map(|(filename, _)| filename)
      .cloned()
      .collect();
    Ok(filenames)
  }

  #[napi]
  pub fn has_asset(&self, name: String) -> Result<bool> {
    Ok(self.inner.assets().contains_key(&name))
  }

  #[napi]
  pub fn emit_asset(
    &mut self,
    filename: String,
    source: JsCompatSource,
    asset_info: JsAssetInfo,
  ) -> Result<()> {
    let compat_source: CompatSource = source.into();

    self.inner.emit_asset(
      filename,
      rspack_core::CompilationAsset::new(Some(compat_source.boxed()), asset_info.into()),
    );

    Ok(())
  }

  #[napi]
  pub fn delete_asset(&mut self, filename: String) {
    self.inner.delete_asset(&filename);
  }

  #[napi(getter)]
  pub fn entrypoints(&self) -> HashMap<String, JsChunkGroup> {
    let entrypoints = self.inner.entrypoints();
    entrypoints
      .iter()
      .map(|(n, _)| {
        (
          n.clone(),
          JsChunkGroup::from_chunk_group(self.inner.entrypoint_by_name(n), self.inner),
        )
      })
      .collect()
  }

  #[napi(getter)]
  pub fn hash(&self) -> Option<String> {
    self.inner.get_hash().map(|hash| hash.to_owned())
  }

  #[napi]
  pub fn get_file_dependencies(&self) -> Vec<String> {
    self
      .inner
      .file_dependencies
      .iter()
      .map(|i| i.to_string_lossy().to_string())
      .collect()
  }

  #[napi]
  pub fn get_context_dependencies(&self) -> Vec<String> {
    self
      .inner
      .context_dependencies
      .iter()
      .map(|i| i.to_string_lossy().to_string())
      .collect()
  }

  #[napi]
  pub fn get_missing_dependencies(&self) -> Vec<String> {
    self
      .inner
      .missing_dependencies
      .iter()
      .map(|i| i.to_string_lossy().to_string())
      .collect()
  }

  #[napi]
  pub fn get_build_dependencies(&self) -> Vec<String> {
    self
      .inner
      .build_dependencies
      .iter()
      .map(|i| i.to_string_lossy().to_string())
      .collect()
  }

  #[napi(ts_args_type = r#"severity: "error" | "warning", title: string, message: string"#)]
  pub fn push_diagnostic(&mut self, severity: String, title: String, message: String) {
    let diagnostic = match severity.as_str() {
      "warning" => rspack_error::Diagnostic::warn(title, message, 0, 0),
      _ => rspack_error::Diagnostic::error(title, message, 0, 0),
    };
    self.inner.push_diagnostic(diagnostic);
  }

  #[napi]
  pub fn get_stats(&self, reference: Reference<JsCompilation>, env: Env) -> Result<JsStats> {
    Ok(JsStats::new(reference.share_with(env, |compilation| {
      Ok(compilation.inner.get_stats())
    })?))
  }

  #[napi]
  pub fn get_asset_path(&self, filename: String, data: PathData) -> String {
    self.inner.get_asset_path(
      &rspack_core::Filename::from(filename),
      data.as_core_path_data(),
    )
  }

  #[napi]
  pub fn get_asset_path_with_info(&self, filename: String, data: PathData) -> PathWithInfo {
    self
      .inner
      .get_asset_path_with_info(
        &rspack_core::Filename::from(filename),
        data.as_core_path_data(),
      )
      .into()
  }

  #[napi]
  pub fn get_path(&self, filename: String, data: PathData) -> String {
    self.inner.get_path(
      &rspack_core::Filename::from(filename),
      data.as_core_path_data(),
    )
  }

  #[napi]
  pub fn get_path_with_info(&self, filename: String, data: PathData) -> PathWithInfo {
    self
      .inner
      .get_path_with_info(
        &rspack_core::Filename::from(filename),
        data.as_core_path_data(),
      )
      .into()
  }

  #[napi]
  pub fn add_file_dependencies(&mut self, deps: Vec<String>) {
    self
      .inner
      .file_dependencies
      .extend(deps.into_iter().map(PathBuf::from))
  }

  #[napi]
  pub fn add_context_dependencies(&mut self, deps: Vec<String>) {
    self
      .inner
      .context_dependencies
      .extend(deps.into_iter().map(PathBuf::from))
  }

  #[napi]
  pub fn add_missing_dependencies(&mut self, deps: Vec<String>) {
    self
      .inner
      .missing_dependencies
      .extend(deps.into_iter().map(PathBuf::from))
  }

  #[napi]
  pub fn add_build_dependencies(&mut self, deps: Vec<String>) {
    self
      .inner
      .build_dependencies
      .extend(deps.into_iter().map(PathBuf::from))
  }
}

impl JsCompilation {
  pub fn from_compilation(inner: &'static mut rspack_core::Compilation) -> Self {
    Self { inner }
  }
}
