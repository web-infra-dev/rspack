mod dependencies;
mod entries;

use std::cell::RefCell;
use std::collections::HashMap;
use std::ptr::NonNull;

use dependencies::JsDependencies;
use entries::JsEntries;
use napi_derive::napi;
use rspack_collections::IdentifierSet;
use rspack_core::get_chunk_from_ukey;
use rspack_core::get_chunk_group_from_ukey;
use rspack_core::rspack_sources::BoxSource;
use rspack_core::AssetInfo;
use rspack_core::ChunkUkey;
use rspack_core::Compilation;
use rspack_core::CompilationId;
use rspack_core::ModuleIdentifier;
use rspack_error::Diagnostic;
use rspack_napi::napi::bindgen_prelude::*;
use rspack_napi::NapiResultExt;
use rspack_napi::OneShotRef;
use rspack_plugin_runtime::RuntimeModuleFromJs;

use super::{JsFilename, PathWithInfo};
use crate::utils::callbackify;
use crate::JsAddingRuntimeModule;
use crate::JsModuleWrapper;
use crate::JsStatsOptimizationBailout;
use crate::LocalJsFilename;
use crate::ToJsCompatSource;
use crate::{
  chunk::JsChunk, JsAsset, JsAssetInfo, JsChunkGroup, JsCompatSource, JsPathData, JsStats,
};
use crate::{JsRspackDiagnostic, JsRspackError};

#[napi]
pub struct JsCompilation(pub(crate) NonNull<rspack_core::Compilation>);

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
    let compilation = unsafe { self.0.as_mut() };

    compilation
      .update_asset(&filename, |original_source, mut original_info| {
        let new_source: napi::Result<BoxSource> = try {
          let new_source = match new_source_or_function {
            Either::A(new_source) => new_source.into(),
            Either::B(new_source_fn) => {
              let js_compat_source: JsCompatSource =
                new_source_fn.call1(original_source.to_js_compat_source())?;
              js_compat_source.into()
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
        if let Some(new_info) = new_info.into_rspack_result()? {
          original_info.merge_another_asset(new_info);
        }
        Ok((new_source, original_info))
      })
      .map_err(|err| napi::Error::from_reason(err.to_string()))
  }

  #[napi(ts_return_type = "Readonly<JsAsset>[]")]
  pub fn get_assets(&self) -> Result<Vec<JsAsset>> {
    let compilation = unsafe { self.0.as_ref() };

    let mut assets = Vec::<JsAsset>::with_capacity(compilation.assets().len());

    for (filename, asset) in compilation.assets() {
      assets.push(JsAsset {
        name: filename.clone(),
        info: asset.info.clone().into(),
      });
    }

    Ok(assets)
  }

  #[napi]
  pub fn get_asset(&self, name: String) -> Result<Option<JsAsset>> {
    let compilation = unsafe { self.0.as_ref() };

    match compilation.assets().get(&name) {
      Some(asset) => Ok(Some(JsAsset {
        name,
        info: asset.info.clone().into(),
      })),
      None => Ok(None),
    }
  }

  #[napi]
  pub fn get_asset_source(&self, name: String) -> Result<Option<JsCompatSource>> {
    let compilation = unsafe { self.0.as_ref() };

    compilation
      .assets()
      .get(&name)
      .and_then(|v| v.source.as_ref().map(|s| s.to_js_compat_source()))
      .transpose()
  }

  #[napi(getter, ts_return_type = "Array<JsModule>")]
  pub fn modules(&self) -> Vec<JsModuleWrapper> {
    let compilation = unsafe { self.0.as_ref() };

    compilation
      .get_module_graph()
      .modules()
      .keys()
      .filter_map(|module_id| {
        compilation
          .module_by_identifier(module_id)
          .map(|module| JsModuleWrapper::new(module.as_ref(), Some(self.0.as_ptr())))
      })
      .collect::<Vec<_>>()
  }

  #[napi(getter, ts_return_type = "Array<JsModule>")]
  pub fn built_modules(&self) -> Vec<JsModuleWrapper> {
    let compilation = unsafe { self.0.as_ref() };

    compilation
      .built_modules
      .iter()
      .filter_map(|module_id| {
        compilation
          .module_by_identifier(module_id)
          .map(|module| JsModuleWrapper::new(module.as_ref(), Some(self.0.as_ptr())))
      })
      .collect::<Vec<_>>()
  }

  #[napi]
  pub fn get_optimization_bailout(&self) -> Vec<JsStatsOptimizationBailout> {
    let compilation = unsafe { self.0.as_ref() };

    compilation
      .get_module_graph()
      .module_graph_modules()
      .values()
      .flat_map(|item| item.optimization_bailout.clone())
      .map(|item| JsStatsOptimizationBailout { inner: item })
      .collect::<Vec<_>>()
  }

  #[napi]
  pub fn get_chunks(&self) -> Vec<JsChunk> {
    let compilation = unsafe { self.0.as_ref() };

    compilation
      .chunk_by_ukey
      .values()
      .map(JsChunk::from)
      .collect::<Vec<_>>()
  }

  #[napi]
  pub fn get_named_chunk_keys(&self) -> Vec<String> {
    let compilation = unsafe { self.0.as_ref() };

    compilation.named_chunks.keys().cloned().collect::<Vec<_>>()
  }

  #[napi]
  pub fn get_named_chunk(&self, name: String) -> Option<JsChunk> {
    let compilation = unsafe { self.0.as_ref() };

    compilation
      .named_chunks
      .get(&name)
      .and_then(|c| get_chunk_from_ukey(c, &compilation.chunk_by_ukey).map(JsChunk::from))
  }

  #[napi]
  pub fn get_named_chunk_group_keys(&self) -> Vec<String> {
    let compilation = unsafe { self.0.as_ref() };

    compilation
      .named_chunk_groups
      .keys()
      .cloned()
      .collect::<Vec<_>>()
  }

  #[napi]
  pub fn get_named_chunk_group(&self, name: String) -> Option<JsChunkGroup> {
    let compilation = unsafe { self.0.as_ref() };

    compilation.named_chunk_groups.get(&name).and_then(|c| {
      get_chunk_group_from_ukey(c, &compilation.chunk_group_by_ukey)
        .map(|cg| JsChunkGroup::from_chunk_group(cg, compilation))
    })
  }

  #[napi]
  pub fn set_asset_source(&mut self, name: String, source: JsCompatSource) {
    let compilation = unsafe { self.0.as_mut() };

    let source: BoxSource = source.into();
    match compilation.assets_mut().entry(name) {
      std::collections::hash_map::Entry::Occupied(mut e) => e.get_mut().set_source(Some(source)),
      std::collections::hash_map::Entry::Vacant(e) => {
        e.insert(rspack_core::CompilationAsset::from(source));
      }
    };
  }

  #[napi]
  pub fn delete_asset_source(&mut self, name: String) {
    let compilation = unsafe { self.0.as_mut() };

    compilation
      .assets_mut()
      .entry(name)
      .and_modify(|a| a.set_source(None));
  }

  #[napi]
  pub fn get_asset_filenames(&self) -> Vec<String> {
    let compilation = unsafe { self.0.as_ref() };

    compilation
      .assets()
      .iter()
      .filter(|(_, asset)| asset.get_source().is_some())
      .map(|(filename, _)| filename)
      .cloned()
      .collect()
  }

  #[napi]
  pub fn has_asset(&self, name: String) -> bool {
    let compilation = unsafe { self.0.as_ref() };

    compilation.assets().contains_key(&name)
  }

  #[napi]
  pub fn emit_asset_from_loader(
    &mut self,
    filename: String,
    source: JsCompatSource,
    asset_info: JsAssetInfo,
    module: String,
  ) {
    let compilation = unsafe { self.0.as_mut() };

    self.emit_asset(filename.clone(), source, asset_info);
    compilation
      .module_assets
      .entry(ModuleIdentifier::from(module))
      .or_default()
      .insert(filename);
  }

  #[napi]
  pub fn emit_asset(&mut self, filename: String, source: JsCompatSource, asset_info: JsAssetInfo) {
    let compilation = unsafe { self.0.as_mut() };

    compilation.emit_asset(
      filename,
      rspack_core::CompilationAsset::new(Some(source.into()), asset_info.into()),
    );
  }

  #[napi]
  pub fn delete_asset(&mut self, filename: String) {
    let compilation = unsafe { self.0.as_mut() };

    compilation.delete_asset(&filename);
  }

  #[napi]
  pub fn rename_asset(&mut self, filename: String, new_name: String) {
    let compilation = unsafe { self.0.as_mut() };

    compilation.rename_asset(&filename, new_name);
  }

  #[napi(getter)]
  pub fn entrypoints(&self) -> HashMap<String, JsChunkGroup> {
    let compilation = unsafe { self.0.as_ref() };

    compilation
      .entrypoints()
      .iter()
      .map(|(n, _)| {
        (
          n.clone(),
          JsChunkGroup::from_chunk_group(compilation.entrypoint_by_name(n), compilation),
        )
      })
      .collect()
  }

  #[napi(getter)]
  pub fn chunk_groups(&self) -> Vec<JsChunkGroup> {
    let compilation = unsafe { self.0.as_ref() };

    compilation
      .chunk_group_by_ukey
      .values()
      .map(|cg| JsChunkGroup::from_chunk_group(cg, compilation))
      .collect::<Vec<JsChunkGroup>>()
  }

  #[napi(getter)]
  pub fn hash(&self) -> Option<String> {
    let compilation = unsafe { self.0.as_ref() };

    compilation.get_hash().map(|hash| hash.to_owned())
  }

  #[napi]
  pub fn dependencies(&'static self) -> JsDependencies {
    let compilation = unsafe { self.0.as_ref() };

    JsDependencies::new(compilation)
  }

  #[napi]
  pub fn push_diagnostic(&mut self, diagnostic: JsRspackDiagnostic) {
    let compilation = unsafe { self.0.as_mut() };

    compilation.push_diagnostic(diagnostic.into());
  }

  #[napi]
  pub fn splice_diagnostic(
    &mut self,
    start: u32,
    end: u32,
    replace_with: Vec<crate::JsRspackDiagnostic>,
  ) {
    let compilation = unsafe { self.0.as_mut() };

    let diagnostics = replace_with.into_iter().map(Into::into).collect();
    compilation.splice_diagnostic(start as usize, end as usize, diagnostics);
  }

  #[napi(ts_args_type = r#"diagnostic: ExternalObject<'Diagnostic'>"#)]
  pub fn push_native_diagnostic(&mut self, diagnostic: External<Diagnostic>) {
    let compilation = unsafe { self.0.as_mut() };

    compilation.push_diagnostic(diagnostic.clone());
  }

  #[napi(ts_args_type = r#"diagnostics: ExternalObject<'Diagnostic[]'>"#)]
  pub fn push_native_diagnostics(&mut self, mut diagnostics: External<Vec<Diagnostic>>) {
    let compilation = unsafe { self.0.as_mut() };

    while let Some(diagnostic) = diagnostics.pop() {
      compilation.push_diagnostic(diagnostic);
    }
  }

  #[napi]
  pub fn get_errors(&self) -> Vec<JsRspackError> {
    let compilation = unsafe { self.0.as_ref() };

    let colored = compilation.options.stats.colors;
    compilation
      .get_errors_sorted()
      .map(|d| {
        JsRspackError::try_from_diagnostic(d, colored)
          .expect("should convert diagnostic to `JsRspackError`")
      })
      .collect()
  }

  #[napi]
  pub fn get_warnings(&self) -> Vec<JsRspackError> {
    let compilation = unsafe { self.0.as_ref() };

    let colored = compilation.options.stats.colors;
    compilation
      .get_warnings_sorted()
      .map(|d| {
        JsRspackError::try_from_diagnostic(d, colored)
          .expect("should convert diagnostic to `JsRspackError`")
      })
      .collect()
  }

  #[napi]
  pub fn get_stats(&self, reference: Reference<JsCompilation>, env: Env) -> Result<JsStats> {
    Ok(JsStats::new(reference.share_with(env, |compilation| {
      let compilation = unsafe { compilation.0.as_ref() };

      Ok(compilation.get_stats())
    })?))
  }

  #[napi]
  pub fn get_asset_path(
    &self,
    filename: LocalJsFilename,
    data: JsPathData,
  ) -> napi::Result<String> {
    let compilation = unsafe { self.0.as_ref() };

    let chunk = data.chunk.as_ref().map(|c| c.to_chunk(compilation));
    compilation.get_asset_path(&filename.into(), data.to_path_data(chunk.as_ref()))
  }

  #[napi]
  pub fn get_asset_path_with_info(
    &self,
    filename: LocalJsFilename,
    data: JsPathData,
  ) -> napi::Result<PathWithInfo> {
    let compilation = unsafe { self.0.as_ref() };

    let chunk = data.chunk.as_ref().map(|c| c.to_chunk(compilation));
    let path_and_asset_info =
      compilation.get_asset_path_with_info(&filename.into(), data.to_path_data(chunk.as_ref()))?;
    Ok(path_and_asset_info.into())
  }

  #[napi]
  pub fn get_path(&self, filename: LocalJsFilename, data: JsPathData) -> napi::Result<String> {
    let compilation = unsafe { self.0.as_ref() };

    let chunk = data.chunk.as_ref().map(|c| c.to_chunk(compilation));
    compilation.get_path(&filename.into(), data.to_path_data(chunk.as_ref()))
  }

  #[napi]
  pub fn get_path_with_info(
    &self,
    filename: LocalJsFilename,
    data: JsPathData,
  ) -> napi::Result<PathWithInfo> {
    let compilation = unsafe { self.0.as_ref() };

    let chunk = data.chunk.as_ref().map(|c| c.to_chunk(compilation));
    let path_and_asset_info =
      compilation.get_path_with_info(&filename.into(), data.to_path_data(chunk.as_ref()))?;
    Ok(path_and_asset_info.into())
  }

  #[napi]
  pub fn add_file_dependencies(&mut self, deps: Vec<String>) {
    let compilation = unsafe { self.0.as_mut() };

    compilation
      .file_dependencies
      .extend(deps.into_iter().map(Into::into))
  }

  #[napi]
  pub fn add_context_dependencies(&mut self, deps: Vec<String>) {
    let compilation = unsafe { self.0.as_mut() };

    compilation
      .context_dependencies
      .extend(deps.into_iter().map(Into::into))
  }

  #[napi]
  pub fn add_missing_dependencies(&mut self, deps: Vec<String>) {
    let compilation = unsafe { self.0.as_mut() };

    compilation
      .missing_dependencies
      .extend(deps.into_iter().map(Into::into))
  }

  #[napi]
  pub fn add_build_dependencies(&mut self, deps: Vec<String>) {
    let compilation = unsafe { self.0.as_mut() };

    compilation
      .build_dependencies
      .extend(deps.into_iter().map(Into::into))
  }

  #[napi]
  pub fn rebuild_module(
    &mut self,
    env: Env,
    module_identifiers: Vec<String>,
    f: JsFunction,
  ) -> Result<()> {
    let compilation = unsafe { self.0.as_mut() };

    callbackify(env, f, async {
      let modules = compilation
        .rebuild_module(
          IdentifierSet::from_iter(module_identifiers.into_iter().map(ModuleIdentifier::from)),
          |modules| {
            modules
              .into_iter()
              .map(|module| JsModuleWrapper::new(module.as_ref(), None))
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
    &self,
    env: Env,
    request: String,
    layer: Option<String>,
    public_path: Option<JsFilename>,
    base_uri: Option<String>,
    original_module: Option<String>,
    original_module_context: Option<String>,
    callback: JsFunction,
  ) -> Result<()> {
    let compilation = unsafe { self.0.as_ref() };

    callbackify(env, callback, async {
      let module_executor = compilation
        .module_executor
        .as_ref()
        .expect("should have module executor");
      let result = module_executor
        .import_module(
          request,
          layer,
          public_path.map(|p| p.into()),
          base_uri,
          original_module_context.map(rspack_core::Context::from),
          original_module.map(ModuleIdentifier::from),
        )
        .await;
      match result {
        Ok(res) => {
          let js_result = JsExecuteModuleResult {
            cacheable: res.cacheable,
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
            assets: res.assets.into_iter().collect(),
            id: res.id,
          };
          Ok(js_result)
        }
        Err(e) => Err(Error::new(napi::Status::GenericFailure, format!("{e}"))),
      }
    })
  }

  #[napi(getter)]
  pub fn entries(&mut self) -> JsEntries {
    let compilation = unsafe { self.0.as_mut() };

    JsEntries::new(compilation)
  }

  #[napi]
  pub fn add_runtime_module(
    &mut self,
    chunk_ukey: u32,
    runtime_module: JsAddingRuntimeModule,
  ) -> napi::Result<()> {
    let compilation = unsafe { self.0.as_mut() };

    compilation
      .add_runtime_module(
        &ChunkUkey::from(chunk_ukey),
        Box::new(RuntimeModuleFromJs::from(runtime_module)),
      )
      .map_err(|e| Error::new(napi::Status::GenericFailure, format!("{e}")))
  }
}

thread_local! {
  static COMPILATION_INSTANCE_REFS: RefCell<HashMap<CompilationId, OneShotRef<ClassInstance<JsCompilation>>>> = Default::default();
}

// The difference between JsCompilationWrapper and JsCompilation is:
// JsCompilationWrapper maintains a cache to ensure that the corresponding instance of the same Compilation is unique on the JS side.
//
// This means that when transferring a JsCompilation from Rust to JS, you must use JsCompilationWrapper instead.
pub struct JsCompilationWrapper(NonNull<rspack_core::Compilation>);

unsafe impl Send for JsCompilationWrapper {}

impl JsCompilationWrapper {
  pub fn new(compilation: *const Compilation) -> Self {
    #[allow(clippy::unwrap_used)]
    Self(NonNull::new(compilation as *mut Compilation).unwrap())
  }

  pub fn cleanup_last_compilation(compilation_id: CompilationId) {
    COMPILATION_INSTANCE_REFS.with(|ref_cell| {
      let mut refs = ref_cell.borrow_mut();
      refs.remove(&compilation_id);
    });
    JsModuleWrapper::cleanup_last_compilation(compilation_id);
  }
}

impl ToNapiValue for JsCompilationWrapper {
  unsafe fn to_napi_value(env: sys::napi_env, val: Self) -> Result<sys::napi_value> {
    COMPILATION_INSTANCE_REFS.with(|ref_cell| {
      let mut refs = ref_cell.borrow_mut();
      let compilation = unsafe { val.0.as_ref() };

      let compilation_id = compilation.id();
      match refs.entry(compilation_id) {
        std::collections::hash_map::Entry::Occupied(entry) => {
          let r = entry.get();
          ToNapiValue::to_napi_value(env, r)
        }
        std::collections::hash_map::Entry::Vacant(entry) => {
          let env_wrapper = Env::from_raw(env);
          let instance = JsCompilation(val.0).into_instance(env_wrapper)?;
          let r = OneShotRef::new(env, instance)?;
          let r = entry.insert(r);
          ToNapiValue::to_napi_value(env, r)
        }
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
  pub cacheable: bool,
  pub assets: Vec<String>,
  pub id: u32,
}

#[napi(object)]
#[derive(Clone, Debug)]
pub struct JsBuildTimeExecutionOption {
  pub public_path: Option<String>,
  pub base_uri: Option<String>,
}
