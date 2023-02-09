use std::collections::HashMap;
use std::path::PathBuf;
use std::pin::Pin;

use napi::bindgen_prelude::*;
use napi::NapiRaw;
use rspack_core::rspack_sources::SourceExt;
use rspack_napi_utils::NapiResultExt;

use crate::{
  CompatSource, JsAsset, JsAssetInfo, JsChunkGroup, JsCompatSource, JsStats, ToJsCompatSource,
};

#[napi]
pub struct JsCompilation {
  inner: Pin<&'static mut rspack_core::Compilation>,
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
      .as_mut()
      .update_asset(&filename, |original_source, original_info| {
        let napi_result: napi::Result<()> = try {
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
          *original_source = new_source;
        };
        napi_result.into_rspack_result()?;

        let napi_result: napi::Result<()> = try {
          if let Some(asset_info_update_or_function) = asset_info_update_or_function {
            let asset_info = match asset_info_update_or_function {
              Either::A(asset_info) => asset_info,
              Either::B(asset_info_fn) => {
                let asset_info = unsafe {
                  call_js_function_with_napi_objects!(
                    env,
                    asset_info_fn,
                    Into::<JsAssetInfo>::into(original_info.clone())
                  )
                }?;

                unsafe {
                  convert_raw_napi_value_to_napi_value!(env, JsAssetInfo, asset_info.raw())
                }?
              }
            };

            *original_info = asset_info.into();
          }
        };
        napi_result.into_rspack_result()?;
        Ok(())
      })
      .map_err(|err| napi::Error::from_reason(err.to_string()))
  }

  #[napi(ts_return_type = "Readonly<JsAsset>[]")]
  pub fn get_assets(&self) -> Result<Vec<JsAsset>> {
    let mut assets = Vec::<JsAsset>::with_capacity(self.inner.assets.len());

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
    match self.inner.assets.get(&name) {
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
      .assets
      .get(&name)
      .and_then(|v| v.source.as_ref().map(|s| s.to_js_compat_source()))
      .transpose()
  }

  #[napi]
  pub fn set_asset_source(&mut self, name: String, source: JsCompatSource) {
    let source = CompatSource::from(source).boxed();
    match unsafe { self.inner_mut() }.assets.entry(name) {
      std::collections::hash_map::Entry::Occupied(mut e) => e.get_mut().set_source(Some(source)),
      std::collections::hash_map::Entry::Vacant(e) => {
        e.insert(rspack_core::CompilationAsset::with_source(source));
      }
    };
  }

  #[napi]
  pub fn delete_asset_source(&mut self, name: String) {
    unsafe { self.inner_mut() }
      .assets
      .entry(name)
      .and_modify(|a| a.set_source(None));
  }

  #[napi]
  pub fn get_asset_filenames(&self) -> Result<Vec<String>> {
    let filenames = self
      .inner
      .assets
      .iter()
      .filter(|(_, asset)| asset.get_source().is_some())
      .map(|(filename, _)| filename)
      .cloned()
      .collect();
    Ok(filenames)
  }

  #[napi]
  pub fn has_asset(&self, name: String) -> Result<bool> {
    Ok(self.inner.assets.contains_key(&name))
  }

  #[napi]
  pub fn emit_asset(
    &mut self,
    filename: String,
    source: JsCompatSource,
    asset_info: JsAssetInfo,
  ) -> Result<()> {
    let compat_source: CompatSource = source.into();

    // Safety: It is safe as modify for the asset will never move Compilation.
    unsafe {
      self.inner.as_mut().get_unchecked_mut().emit_asset(
        filename,
        rspack_core::CompilationAsset::new(Some(compat_source.boxed()), asset_info.into()),
      )
    };

    Ok(())
  }

  #[napi]
  pub fn delete_asset(&mut self, filename: String) {
    // Safety: It is safe as modify for the asset will never move Compilation.
    unsafe {
      self.inner_mut().delete_asset(&filename);
    };
  }

  #[napi(getter)]
  pub fn entrypoints(&self) -> HashMap<String, JsChunkGroup> {
    let entrypoints = self.inner.entrypoints();
    entrypoints
      .iter()
      .map(|(n, _)| {
        (
          n.clone(),
          JsChunkGroup::from_chunk_group(self.inner.entrypoint_by_name(n), &self.inner),
        )
      })
      .collect()
  }

  #[napi(getter)]
  pub fn hash(&self) -> String {
    self.inner.hash.to_string()
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
    unsafe {
      self.inner_mut().push_diagnostic(diagnostic);
    };
  }

  #[napi]
  pub fn get_stats(&self, reference: Reference<JsCompilation>, env: Env) -> Result<JsStats> {
    Ok(JsStats::new(reference.share_with(env, |compilation| {
      Ok(compilation.inner.get_stats())
    })?))
  }

  #[napi]
  pub fn add_file_dependencies(&mut self, deps: Vec<String>) {
    unsafe {
      self
        .inner_mut()
        .file_dependencies
        .extend(deps.into_iter().map(|i| PathBuf::from(i)))
    };
  }

  #[napi]
  pub fn add_context_dependencies(&mut self, deps: Vec<String>) {
    unsafe {
      self
        .inner_mut()
        .context_dependencies
        .extend(deps.into_iter().map(|i| PathBuf::from(i)))
    };
  }

  #[napi]
  pub fn add_missing_dependencies(&mut self, deps: Vec<String>) {
    unsafe {
      self
        .inner_mut()
        .missing_dependencies
        .extend(deps.into_iter().map(|i| PathBuf::from(i)))
    };
  }

  #[napi]
  pub fn add_build_dependencies(&mut self, deps: Vec<String>) {
    unsafe {
      self
        .inner_mut()
        .build_dependencies
        .extend(deps.into_iter().map(|i| PathBuf::from(i)))
    };
  }
}

impl JsCompilation {
  pub fn from_compilation(inner: Pin<&'static mut rspack_core::Compilation>) -> Self {
    Self { inner }
  }

  pub unsafe fn inner_mut(&mut self) -> &mut rspack_core::Compilation {
    self.inner.as_mut().get_unchecked_mut()
  }
}
