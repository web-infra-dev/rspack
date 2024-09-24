mod dependencies;
mod entries;

use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;
use std::ops::DerefMut;

use dependencies::DependenciesDTO;
use entries::JsEntries;
use napi_derive::napi;
use rspack_collections::IdentifierSet;
use rspack_core::get_chunk_from_ukey;
use rspack_core::get_chunk_group_from_ukey;
use rspack_core::rspack_sources::BoxSource;
use rspack_core::AssetInfo;
use rspack_core::ChunkUkey;
use rspack_core::CompilationId;
use rspack_core::ModuleIdentifier;
use rspack_error::Diagnostic;
use rspack_napi::napi::bindgen_prelude::*;
use rspack_napi::NapiResultExt;
use rspack_napi::Ref;
use rspack_plugin_runtime::RuntimeModuleFromJs;
use sys::napi_env;

use super::module::ToJsModule;
use super::{JsFilename, PathWithInfo};
use crate::utils::callbackify;
use crate::JsAddingRuntimeModule;
use crate::JsStatsOptimizationBailout;
use crate::LocalJsFilename;
use crate::ModuleDTOWrapper;
use crate::{
  chunk::JsChunk, JsAsset, JsAssetInfo, JsChunkGroup, JsCompatSource, JsPathData, JsStats,
  ToJsCompatSource,
};
use crate::{JsRspackDiagnostic, JsRspackError};

#[napi]
pub struct JsCompilation(pub(crate) &'static mut rspack_core::Compilation);

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

  #[napi(getter, ts_return_type = "Array<ModuleDTO>")]
  pub fn modules(&'static self) -> Vec<ModuleDTOWrapper> {
    self
      .0
      .get_module_graph()
      .modules()
      .keys()
      .cloned()
      .map(|module_id| ModuleDTOWrapper::new(module_id, self.0))
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
  pub fn get_named_chunk_keys(&self) -> Vec<String> {
    self.0.named_chunks.keys().cloned().collect::<Vec<_>>()
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
  pub fn get_named_chunk_group_keys(&self) -> Vec<String> {
    self
      .0
      .named_chunk_groups
      .keys()
      .cloned()
      .collect::<Vec<_>>()
  }

  #[napi]
  pub fn get_named_chunk_group(&self, name: String) -> Option<JsChunkGroup> {
    self.0.named_chunk_groups.get(&name).and_then(|c| {
      get_chunk_group_from_ukey(c, &self.0.chunk_group_by_ukey)
        .map(|cg| JsChunkGroup::from_chunk_group(cg, self.0))
    })
  }

  #[napi]
  pub fn set_asset_source(&mut self, name: String, source: JsCompatSource) {
    let source: BoxSource = source.into();
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
  pub fn get_asset_filenames(&self) -> Vec<String> {
    self
      .0
      .assets()
      .iter()
      .filter(|(_, asset)| asset.get_source().is_some())
      .map(|(filename, _)| filename)
      .cloned()
      .collect()
  }

  #[napi]
  pub fn has_asset(&self, name: String) -> bool {
    self.0.assets().contains_key(&name)
  }

  #[napi]
  pub fn emit_asset_from_loader(
    &mut self,
    filename: String,
    source: JsCompatSource,
    asset_info: JsAssetInfo,
    module: String,
  ) {
    self.emit_asset(filename.clone(), source, asset_info);
    self
      .0
      .module_assets
      .entry(ModuleIdentifier::from(module))
      .or_default()
      .insert(filename);
  }

  #[napi]
  pub fn emit_asset(&mut self, filename: String, source: JsCompatSource, asset_info: JsAssetInfo) {
    self.0.emit_asset(
      filename,
      rspack_core::CompilationAsset::new(Some(source.into()), asset_info.into()),
    );
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
  pub fn chunk_groups(&self) -> Vec<JsChunkGroup> {
    self
      .0
      .chunk_group_by_ukey
      .values()
      .map(|cg| JsChunkGroup::from_chunk_group(cg, self.0))
      .collect::<Vec<JsChunkGroup>>()
  }

  #[napi(getter)]
  pub fn hash(&self) -> Option<String> {
    self.0.get_hash().map(|hash| hash.to_owned())
  }

  #[napi]
  pub fn dependencies(&'static self) -> DependenciesDTO {
    DependenciesDTO::new(self.0)
  }

  #[napi]
  pub fn push_diagnostic(&mut self, diagnostic: JsRspackDiagnostic) {
    self.0.push_diagnostic(diagnostic.into());
  }

  #[napi]
  pub fn splice_diagnostic(
    &mut self,
    start: u32,
    end: u32,
    replace_with: Vec<crate::JsRspackDiagnostic>,
  ) {
    let diagnostics = replace_with.into_iter().map(Into::into).collect();
    self
      .0
      .splice_diagnostic(start as usize, end as usize, diagnostics);
  }

  #[napi(ts_args_type = r#"diagnostic: ExternalObject<'Diagnostic'>"#)]
  pub fn push_native_diagnostic(&mut self, diagnostic: External<Diagnostic>) {
    self.0.push_diagnostic(diagnostic.clone());
  }

  #[napi(ts_args_type = r#"diagnostics: ExternalObject<'Diagnostic[]'>"#)]
  pub fn push_native_diagnostics(&mut self, mut diagnostics: External<Vec<Diagnostic>>) {
    while let Some(diagnostic) = diagnostics.pop() {
      self.0.push_diagnostic(diagnostic);
    }
  }

  #[napi]
  pub fn get_errors(&self) -> Vec<JsRspackError> {
    let colored = self.0.options.stats.colors;
    self
      .0
      .get_errors_sorted()
      .map(|d| {
        JsRspackError::try_from_diagnostic(d, colored)
          .expect("should convert diagnostic to `JsRspackError`")
      })
      .collect()
  }

  #[napi]
  pub fn get_warnings(&self) -> Vec<JsRspackError> {
    let colored = self.0.options.stats.colors;
    self
      .0
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
      Ok(compilation.0.get_stats())
    })?))
  }

  #[napi]
  pub fn get_asset_path(
    &self,
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
    filename: LocalJsFilename,
    data: JsPathData,
  ) -> napi::Result<PathWithInfo> {
    let path_and_asset_info = self
      .0
      .get_asset_path_with_info(&filename.into(), data.as_core_path_data())?;
    Ok(path_and_asset_info.into())
  }

  #[napi]
  pub fn get_path(&self, filename: LocalJsFilename, data: JsPathData) -> napi::Result<String> {
    self.0.get_path(&filename.into(), data.as_core_path_data())
  }

  #[napi]
  pub fn get_path_with_info(
    &self,
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
      .extend(deps.into_iter().map(Into::into))
  }

  #[napi]
  pub fn add_context_dependencies(&mut self, deps: Vec<String>) {
    self
      .0
      .context_dependencies
      .extend(deps.into_iter().map(Into::into))
  }

  #[napi]
  pub fn add_missing_dependencies(&mut self, deps: Vec<String>) {
    self
      .0
      .missing_dependencies
      .extend(deps.into_iter().map(Into::into))
  }

  #[napi]
  pub fn add_build_dependencies(&mut self, deps: Vec<String>) {
    self
      .0
      .build_dependencies
      .extend(deps.into_iter().map(Into::into))
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
          IdentifierSet::from_iter(module_identifiers.into_iter().map(ModuleIdentifier::from)),
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
    &'static self,
    env: Env,
    request: String,
    layer: Option<String>,
    public_path: Option<JsFilename>,
    base_uri: Option<String>,
    original_module: Option<String>,
    original_module_context: Option<String>,
    callback: JsFunction,
  ) -> Result<()> {
    callbackify(env, callback, async {
      let module_executor = self
        .0
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
  pub fn entries(&'static mut self) -> JsEntries {
    JsEntries::new(self.0)
  }

  #[napi]
  pub fn add_runtime_module(
    &'static mut self,
    chunk_ukey: u32,
    runtime_module: JsAddingRuntimeModule,
  ) -> napi::Result<()> {
    self
      .0
      .add_runtime_module(
        &ChunkUkey::from(chunk_ukey),
        Box::new(RuntimeModuleFromJs::from(runtime_module)),
      )
      .map_err(|e| Error::new(napi::Status::GenericFailure, format!("{e}")))
  }
}

#[derive(Default)]
struct CompilationInstanceRefs(RefCell<HashMap<CompilationId, (Ref, napi_env)>>);

impl Drop for CompilationInstanceRefs {
  fn drop(&mut self) {
    // cleanup references to be executed in cases of panic or unexpected termination
    let mut refs = self.0.borrow_mut();
    for (_, (mut r, env)) in refs.drain() {
      let _ = r.unref(env);
    }
  }
}

thread_local! {
  static COMPILATION_INSTANCE_REFS: CompilationInstanceRefs = Default::default();
}

// The difference between JsCompilationWrapper and JsCompilation is:
// JsCompilationWrapper maintains a cache to ensure that the corresponding instance of the same Compilation is unique on the JS side.
//
// This means that when transferring a JsCompilation from Rust to JS, you must use JsCompilationWrapper instead.
pub struct JsCompilationWrapper(pub(crate) &'static mut rspack_core::Compilation);

impl JsCompilationWrapper {
  pub fn new(compilation: &mut rspack_core::Compilation) -> Self {
    // SAFETY:
    // 1. `Compiler` is stored on the heap and pinned in binding crate.
    // 2. `Compilation` outlives `JsCompilation` and `Compiler` outlives `Compilation`.
    // 3. `JsCompilation` was replaced everytime a new `Compilation` was created before getting accessed.
    Self(unsafe {
      std::mem::transmute::<&'_ mut rspack_core::Compilation, &'static mut rspack_core::Compilation>(
        compilation,
      )
    })
  }

  pub fn cleanup(compilation_id: CompilationId) {
    COMPILATION_INSTANCE_REFS.with(|ref_cell| {
      let mut refs = ref_cell.0.borrow_mut();
      if let Some((mut r, env)) = refs.remove(&compilation_id) {
        let _ = r.unref(env);
      }
    });
    ModuleDTOWrapper::cleanup(compilation_id);
  }
}

impl ToNapiValue for JsCompilationWrapper {
  unsafe fn to_napi_value(env: sys::napi_env, val: Self) -> Result<sys::napi_value> {
    COMPILATION_INSTANCE_REFS.with(|ref_cell| {
      let mut env_wrapper = Env::from_raw(env);
      let mut refs = ref_cell.0.borrow_mut();
      let compilation_id = val.0.id();
      let mut vacant = false;
      let napi_value = match refs.entry(compilation_id) {
        std::collections::hash_map::Entry::Occupied(entry) => {
          let r = entry.get();
          ToNapiValue::to_napi_value(env, &r.0)
        }
        std::collections::hash_map::Entry::Vacant(entry) => {
          vacant = true;
          let instance = JsCompilation(val.0).into_instance(env_wrapper)?;
          let napi_value = ToNapiValue::to_napi_value(env, instance)?;
          let r = Ref::new(env, napi_value, 1)?;
          let r = entry.insert((r, env));
          ToNapiValue::to_napi_value(env, &r.0)
        }
      };
      if vacant {
        // cleanup references to be executed when the JS thread exits normally
        let _ = env_wrapper
          .add_env_cleanup_hook((), move |_| JsCompilationWrapper::cleanup(compilation_id));
      }
      napi_value
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
