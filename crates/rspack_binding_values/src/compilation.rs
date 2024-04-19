use std::collections::HashMap;
use std::ops::Deref;
use std::ops::DerefMut;
use std::path::PathBuf;

use napi_derive::napi;
use rspack_core::get_chunk_from_ukey;
use rspack_core::rspack_sources::BoxSource;
use rspack_core::rspack_sources::SourceExt;
use rspack_core::AssetInfo;
use rspack_core::ModuleIdentifier;
use rspack_error::Diagnostic;
use rspack_napi::napi::bindgen_prelude::*;
use rspack_napi::NapiResultExt;

use super::module::ToJsModule;
use super::PathWithInfo;
use crate::utils::callbackify;
use crate::JsStatsOptimizationBailout;
use crate::LocalJsFilename;
use crate::{
  chunk::JsChunk, module::JsModule, CompatSource, JsAsset, JsAssetInfo, JsChunkGroup,
  JsCompatSource, JsPathData, JsStats, ToJsCompatSource,
};

#[napi(object_from_js = false)]
pub struct JsCompilation(pub(crate) &'static mut rspack_core::Compilation);

impl JsCompilation {
  /// Convert Rust `Compilation` to `JsCompilation`.
  ///
  /// ## JS Interoperable
  /// `JsCompilation` implements [napi::bindgen_prelude::ToNapiValue].
  /// It can be send to JavaScript.
  ///
  /// ## Safety
  /// Safety is guaranteed by the following contracts:
  /// 1. `Compiler` should not be moved. For example: store it on the heap.
  /// 2. The pointer should be valid for the entire lifetime of `JsCompilation`.
  /// 3. Caching old `Compilation` will result the program to undefined behavior and it's likely to crash.
  pub unsafe fn from_compilation(inner: &mut rspack_core::Compilation) -> Self {
    Self(unsafe {
      std::mem::transmute::<&'_ mut rspack_core::Compilation, &'static mut rspack_core::Compilation>(
        inner,
      )
    })
  }
}

impl Deref for JsCompilation {
  type Target = rspack_core::Compilation;

  fn deref(&self) -> &Self::Target {
    self.0
  }
}

impl DerefMut for JsCompilation {
  fn deref_mut(&mut self) -> &mut Self::Target {
    self.0
  }
}

#[napi(object)]
pub struct JsDiagnostic {
  #[napi(ts_type = "'error' | 'warning'")]
  pub severity: String,
  pub title: String,
  pub message: String,
}

#[napi]
impl JsCompilation {
  #[napi(
    ts_args_type = r#"filename: string, newSourceOrFunction: JsCompatSource | ((source: JsCompatSource) => JsCompatSource), assetInfoUpdateOrFunction?: JsAssetInfo | ((assetInfo: JsAssetInfo) => JsAssetInfo)"#
  )]
  pub fn update_asset(
    &mut self,
    filename: String,
    new_source_or_function: Either<JsCompatSource, JsFunction>,
    asset_info_update_or_function: Option<Either<JsAssetInfo, JsFunction>>,
  ) -> Result<()> {
    self
      .0
      .update_asset(&filename, |original_source, original_info| {
        let new_source: napi::Result<BoxSource> = try {
          let new_source = match new_source_or_function {
            Either::A(new_source) => Into::<CompatSource>::into(new_source).boxed(),
            Either::B(new_source_fn) => {
              let compat_source: CompatSource =
                new_source_fn.call1(original_source.to_js_compat_source())?;
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
              Either::B(asset_info_fn) => Ok(
                asset_info_fn
                  .call1::<JsAssetInfo, JsAssetInfo>(original_info.clone().into())?
                  .into(),
              ),
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
    let mut assets = Vec::<JsAsset>::with_capacity(self.0.assets().len());

    for (filename, asset) in self.0.assets() {
      assets.push(JsAsset {
        name: filename.clone(),
        info: asset.info.clone().into(),
      });
    }

    Ok(assets)
  }

  #[napi]
  pub fn get_asset(&self, name: String) -> Result<Option<JsAsset>> {
    match self.0.assets().get(&name) {
      Some(asset) => Ok(Some(JsAsset {
        name,
        info: asset.info.clone().into(),
      })),
      None => Ok(None),
    }
  }

  #[napi]
  pub fn get_asset_source(&self, name: String) -> Result<Option<JsCompatSource>> {
    self
      .0
      .assets()
      .get(&name)
      .and_then(|v| v.source.as_ref().map(|s| s.to_js_compat_source()))
      .transpose()
  }

  #[napi]
  pub fn get_modules(&self) -> Vec<JsModule> {
    self
      .0
      .get_module_graph()
      .modules()
      .values()
      .filter_map(|module| module.to_js_module().ok())
      .collect::<Vec<_>>()
  }

  #[napi]
  pub fn get_optimization_bailout(&self) -> Vec<JsStatsOptimizationBailout> {
    self
      .0
      .get_module_graph()
      .module_graph_modules()
      .values()
      .flat_map(|item| item.optimization_bailout.clone())
      .map(|item| JsStatsOptimizationBailout { inner: item })
      .collect::<Vec<_>>()
  }

  #[napi]
  pub fn get_chunks(&self) -> Vec<JsChunk> {
    self
      .0
      .chunk_by_ukey
      .values()
      .map(JsChunk::from)
      .collect::<Vec<_>>()
  }

  #[napi]
  pub fn get_named_chunk(&self, name: String) -> Option<JsChunk> {
    self
      .0
      .named_chunks
      .get(&name)
      .and_then(|c| get_chunk_from_ukey(c, &self.0.chunk_by_ukey).map(JsChunk::from))
  }

  #[napi]
  pub fn set_asset_source(&mut self, name: String, source: JsCompatSource) {
    let source = CompatSource::from(source).boxed();
    match self.0.assets_mut().entry(name) {
      std::collections::hash_map::Entry::Occupied(mut e) => e.get_mut().set_source(Some(source)),
      std::collections::hash_map::Entry::Vacant(e) => {
        e.insert(rspack_core::CompilationAsset::from(source));
      }
    };
  }

  #[napi]
  pub fn delete_asset_source(&mut self, name: String) {
    self
      .0
      .assets_mut()
      .entry(name)
      .and_modify(|a| a.set_source(None));
  }

  #[napi]
  pub fn get_asset_filenames(&self) -> Result<Vec<String>> {
    let filenames = self
      .0
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
    Ok(self.0.assets().contains_key(&name))
  }

  #[napi]
  pub fn emit_asset(
    &mut self,
    filename: String,
    source: JsCompatSource,
    asset_info: JsAssetInfo,
  ) -> Result<()> {
    let compat_source: CompatSource = source.into();

    self.0.emit_asset(
      filename,
      rspack_core::CompilationAsset::new(Some(compat_source.boxed()), asset_info.into()),
    );

    Ok(())
  }

  #[napi]
  pub fn delete_asset(&mut self, filename: String) {
    self.0.delete_asset(&filename);
  }

  #[napi]
  pub fn rename_asset(&mut self, filename: String, new_name: String) {
    self.0.rename_asset(&filename, new_name);
  }

  #[napi(getter)]
  pub fn entrypoints(&self) -> HashMap<String, JsChunkGroup> {
    let entrypoints = self.0.entrypoints();
    entrypoints
      .iter()
      .map(|(n, _)| {
        (
          n.clone(),
          JsChunkGroup::from_chunk_group(self.0.entrypoint_by_name(n), self.0),
        )
      })
      .collect()
  }

  #[napi(getter)]
  pub fn hash(&self) -> Option<String> {
    self.0.get_hash().map(|hash| hash.to_owned())
  }

  #[napi]
  pub fn get_file_dependencies(&self) -> Vec<String> {
    self
      .0
      .file_dependencies
      .iter()
      .map(|i| i.to_string_lossy().to_string())
      .collect()
  }

  #[napi]
  pub fn get_context_dependencies(&self) -> Vec<String> {
    self
      .0
      .context_dependencies
      .iter()
      .map(|i| i.to_string_lossy().to_string())
      .collect()
  }

  #[napi]
  pub fn get_missing_dependencies(&self) -> Vec<String> {
    self
      .0
      .missing_dependencies
      .iter()
      .map(|i| i.to_string_lossy().to_string())
      .collect()
  }

  #[napi]
  pub fn get_build_dependencies(&self) -> Vec<String> {
    self
      .0
      .build_dependencies
      .iter()
      .map(|i| i.to_string_lossy().to_string())
      .collect()
  }

  #[napi(ts_args_type = r#"severity: "error" | "warning", title: string, message: string"#)]
  pub fn push_diagnostic(&mut self, severity: String, title: String, message: String) {
    let diagnostic = match severity.as_str() {
      "warning" => rspack_error::Diagnostic::warn(title, message),
      _ => rspack_error::Diagnostic::error(title, message),
    };
    self.0.push_diagnostic(diagnostic);
  }

  #[napi]
  pub fn splice_diagnostic(&mut self, start: u32, end: u32, replace_with: Vec<JsDiagnostic>) {
    let diagnostics = replace_with
      .iter()
      .map(|item| match item.severity.as_str() {
        "warning" => rspack_error::Diagnostic::warn(item.title.clone(), item.message.clone()),
        _ => rspack_error::Diagnostic::error(item.title.clone(), item.message.clone()),
      })
      .collect();
    self
      .0
      .splice_diagnostic(start as usize, end as usize, diagnostics);
  }

  #[napi(ts_args_type = r#"diagnostics: ExternalObject<'Diagnostic[]'>"#)]
  pub fn push_native_diagnostics(&mut self, mut diagnostics: External<Vec<Diagnostic>>) {
    while let Some(diagnostic) = diagnostics.pop() {
      self.0.push_diagnostic(diagnostic);
    }
  }

  #[napi]
  pub fn get_stats(&self, reference: Reference<JsCompilation>, env: Env) -> Result<JsStats> {
    Ok(JsStats::new(reference.share_with(env, |compilation| {
      Ok(compilation.0.get_stats())
    })?))
  }

  #[napi]
  pub fn get_asset_path(
    &self,
    #[napi(ts_arg_type = "string | ((pathData: JsPathData, assetInfo?: JsAssetInfo) => string)")]
    filename: LocalJsFilename,
    data: JsPathData,
  ) -> napi::Result<String> {
    self
      .0
      .get_asset_path(&filename.into(), data.as_core_path_data())
  }

  #[napi]
  pub fn get_asset_path_with_info(
    &self,
    #[napi(ts_arg_type = "string | ((pathData: JsPathData, assetInfo?: JsAssetInfo) => string)")]
    filename: LocalJsFilename,
    data: JsPathData,
  ) -> napi::Result<PathWithInfo> {
    let path_and_asset_info = self
      .0
      .get_asset_path_with_info(&filename.into(), data.as_core_path_data())?;
    Ok(path_and_asset_info.into())
  }

  #[napi]
  pub fn get_path(
    &self,
    #[napi(ts_arg_type = "string | ((pathData: JsPathData, assetInfo?: JsAssetInfo) => string)")]
    filename: LocalJsFilename,
    data: JsPathData,
  ) -> napi::Result<String> {
    self.0.get_path(&filename.into(), data.as_core_path_data())
  }

  #[napi]
  pub fn get_path_with_info(
    &self,
    #[napi(ts_arg_type = "string | ((pathData: JsPathData, assetInfo?: JsAssetInfo) => string)")]
    filename: LocalJsFilename,
    data: JsPathData,
  ) -> napi::Result<PathWithInfo> {
    let path_and_asset_info = self
      .0
      .get_path_with_info(&filename.into(), data.as_core_path_data())?;
    Ok(path_and_asset_info.into())
  }

  #[napi]
  pub fn add_file_dependencies(&mut self, deps: Vec<String>) {
    self
      .0
      .file_dependencies
      .extend(deps.into_iter().map(PathBuf::from))
  }

  #[napi]
  pub fn add_context_dependencies(&mut self, deps: Vec<String>) {
    self
      .0
      .context_dependencies
      .extend(deps.into_iter().map(PathBuf::from))
  }

  #[napi]
  pub fn add_missing_dependencies(&mut self, deps: Vec<String>) {
    self
      .0
      .missing_dependencies
      .extend(deps.into_iter().map(PathBuf::from))
  }

  #[napi]
  pub fn add_build_dependencies(&mut self, deps: Vec<String>) {
    self
      .0
      .build_dependencies
      .extend(deps.into_iter().map(PathBuf::from))
  }

  #[napi]
  pub fn rebuild_module(
    &'static mut self,
    env: Env,
    module_identifiers: Vec<String>,
    f: JsFunction,
  ) -> Result<()> {
    callbackify(env, f, async {
      let modules = self
        .0
        .rebuild_module(
          rustc_hash::FxHashSet::from_iter(
            module_identifiers.into_iter().map(ModuleIdentifier::from),
          ),
          |modules| {
            modules
              .into_iter()
              .filter_map(|item| item.to_js_module().ok())
              .collect::<Vec<_>>()
          },
        )
        .await
        .map_err(|e| Error::new(napi::Status::GenericFailure, format!("{e}")))?;
      Ok(modules)
    })
  }

  #[allow(clippy::too_many_arguments)]
  #[napi]
  pub fn import_module(
    &'static mut self,
    env: Env,
    request: String,
    public_path: Option<String>,
    base_uri: Option<String>,
    _original_module: Option<String>,
    original_module_context: Option<String>,
    callback: JsFunction,
  ) -> Result<()> {
    let options = self.0.options.clone();
    let plugin_driver = self.0.plugin_driver.clone();
    let resolver_factory = self.0.resolver_factory.clone();
    let loader_resolver_factory = self.0.loader_resolver_factory.clone();
    let cache = self.0.cache.clone();
    let dependency_factories = self.0.dependency_factories.clone();

    callbackify(env, callback, async {
      let module_executor = self
        .0
        .module_executor
        .as_mut()
        .expect("should have module executor");
      let result = module_executor
        .import_module(
          options,
          plugin_driver,
          resolver_factory,
          loader_resolver_factory,
          cache,
          dependency_factories,
          request,
          public_path,
          base_uri,
          original_module_context.map(rspack_core::Context::new),
        )
        .await;
      match result {
        Ok(res) => {
          let js_result = JsExecuteModuleResult {
            file_dependencies: res
              .file_dependencies
              .into_iter()
              .map(|d| d.to_string_lossy().to_string())
              .collect(),
            context_dependencies: res
              .context_dependencies
              .into_iter()
              .map(|d| d.to_string_lossy().to_string())
              .collect(),
            build_dependencies: res
              .build_dependencies
              .into_iter()
              .map(|d| d.to_string_lossy().to_string())
              .collect(),
            missing_dependencies: res
              .missing_dependencies
              .into_iter()
              .map(|d| d.to_string_lossy().to_string())
              .collect(),
            assets: res.assets.keys().cloned().collect(),
            id: res.id,
          };
          for (filename, asset) in res.assets {
            self.0.emit_asset(filename, asset)
          }
          Ok(js_result)
        }
        Err(e) => Err(Error::new(napi::Status::GenericFailure, format!("{e}"))),
      }
    })
  }
}

#[napi(object)]
pub struct JsExecuteModuleResult {
  pub file_dependencies: Vec<String>,
  pub context_dependencies: Vec<String>,
  pub build_dependencies: Vec<String>,
  pub missing_dependencies: Vec<String>,
  pub assets: Vec<String>,
  pub id: u32,
}

#[napi(object)]
#[derive(Clone, Debug)]
pub struct JsBuildTimeExecutionOption {
  pub public_path: Option<String>,
  pub base_uri: Option<String>,
}
