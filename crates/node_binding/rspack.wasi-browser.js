import {
  instantiateNapiModuleSync as __emnapiInstantiateNapiModuleSync,
  getDefaultContext as __emnapiGetDefaultContext,
  WASI as __WASI,
  createOnMessage as __wasmCreateOnMessageForFsProxy,
} from '@napi-rs/wasm-runtime'
import { memfs } from '@napi-rs/wasm-runtime/fs'
import __wasmUrl from './rspack.wasm32-wasi.wasm?url'

export const { fs: __fs, vol: __volume } = memfs()

const __wasi = new __WASI({
  version: 'preview1',
  fs: __fs,
  preopens: {
    '/': '/',
  },
})

const __emnapiContext = __emnapiGetDefaultContext()

const __sharedMemory = new WebAssembly.Memory({
  initial: 16384,
  maximum: 65536,
  shared: true,
})

const __wasmFile = await fetch(__wasmUrl).then((res) => res.arrayBuffer())

const {
  instance: __napiInstance,
  module: __wasiModule,
  napiModule: __napiModule,
} = __emnapiInstantiateNapiModuleSync(__wasmFile, {
  context: __emnapiContext,
  asyncWorkPoolSize: 4,
  wasi: __wasi,
  onCreateWorker() {
    const worker = new Worker(new URL('./wasi-worker-browser.mjs', import.meta.url), {
      type: 'module',
    })
    worker.addEventListener('message', __wasmCreateOnMessageForFsProxy(__fs))

    return worker
  },
  overwriteImports(importObject) {
    importObject.env = {
      ...importObject.env,
      ...importObject.napi,
      ...importObject.emnapi,
      memory: __sharedMemory,
    }
    return importObject
  },
  beforeInit({ instance }) {
    __napi_rs_initialize_modules(instance)
  },
})

function __napi_rs_initialize_modules(__napiInstance) {
  __napiInstance.exports['__napi_register__ThreadsafeNodeFS_struct_0']?.()
  __napiInstance.exports['__napi_register__NodeFsStats_struct_1']?.()
  __napiInstance.exports['__napi_register__JsAssetInfoRelated_struct_0']?.()
  __napiInstance.exports['__napi_register__JsAssetInfo_struct_1']?.()
  __napiInstance.exports['__napi_register__JsAsset_struct_2']?.()
  __napiInstance.exports['__napi_register__JsAssetEmittedArgs_struct_3']?.()
  __napiInstance.exports['__napi_register__JsChunk_struct_4']?.()
  __napiInstance.exports['__napi_register__JsChunk_impl_18']?.()
  __napiInstance.exports['__napi_register__JsChunk_impl_26']?.()
  __napiInstance.exports['__napi_register__JsChunkAssetArgs_struct_27']?.()
  __napiInstance.exports['__napi_register__JsChunkGraph_struct_28']?.()
  __napiInstance.exports['__napi_register__JsChunkGraph_impl_36']?.()
  __napiInstance.exports['__napi_register__JsChunkGroup_struct_37']?.()
  __napiInstance.exports['__napi_register__JsChunkGroup_impl_42']?.()
  __napiInstance.exports['__napi_register__JsChunkGroup_impl_49']?.()
  __napiInstance.exports['__napi_register__JsChunkGroupOrigin_struct_50']?.()
  __napiInstance.exports['__napi_register__JsCleanOptions_struct_51']?.()
  __napiInstance.exports['__napi_register__JsCodegenerationResults_struct_52']?.()
  __napiInstance.exports['__napi_register__JsCodegenerationResult_struct_53']?.()
  __napiInstance.exports['__napi_register__JsDependencies_struct_54']?.()
  __napiInstance.exports['__napi_register__JsDependencies_impl_67']?.()
  __napiInstance.exports['__napi_register__EntryOptionsDTO_struct_68']?.()
  __napiInstance.exports['__napi_register__EntryOptionsDTO_impl_85']?.()
  __napiInstance.exports['__napi_register__JsEntryData_struct_86']?.()
  __napiInstance.exports['__napi_register__EntryDataDTO_struct_87']?.()
  __napiInstance.exports['__napi_register__EntryDataDTO_impl_91']?.()
  __napiInstance.exports['__napi_register__JsEntries_struct_92']?.()
  __napiInstance.exports['__napi_register__JsEntries_impl_101']?.()
  __napiInstance.exports['__napi_register__JsCompilation_struct_102']?.()
  __napiInstance.exports['__napi_register__JsCompilation_impl_149']?.()
  __napiInstance.exports['__napi_register__JsExecuteModuleResult_struct_150']?.()
  __napiInstance.exports['__napi_register__JsBuildTimeExecutionOption_struct_151']?.()
  __napiInstance.exports['__napi_register__JsContextModuleFactoryBeforeResolveData_struct_152']?.()
  __napiInstance.exports['__napi_register__JsContextModuleFactoryBeforeResolveData_impl_161']?.()
  __napiInstance.exports['__napi_register__JsContextModuleFactoryAfterResolveData_struct_162']?.()
  __napiInstance.exports['__napi_register__JsContextModuleFactoryAfterResolveData_impl_174']?.()
  __napiInstance.exports['__napi_register__JsDependency_struct_175']?.()
  __napiInstance.exports['__napi_register__JsDependency_impl_181']?.()
  __napiInstance.exports['__napi_register__RawDependency_struct_182']?.()
  __napiInstance.exports['__napi_register__JsDependenciesBlock_struct_183']?.()
  __napiInstance.exports['__napi_register__JsDependenciesBlock_impl_186']?.()
  __napiInstance.exports['__napi_register__JsExportsInfo_struct_187']?.()
  __napiInstance.exports['__napi_register__JsExportsInfo_impl_192']?.()
  __napiInstance.exports['__napi_register__JsHtmlPluginTag_struct_193']?.()
  __napiInstance.exports['__napi_register__JsHtmlPluginAssets_struct_194']?.()
  __napiInstance.exports['__napi_register__JsBeforeAssetTagGenerationData_struct_195']?.()
  __napiInstance.exports['__napi_register__JsHtmlPluginAssetTags_struct_196']?.()
  __napiInstance.exports['__napi_register__JsAlterAssetTagsData_struct_197']?.()
  __napiInstance.exports['__napi_register__JsAlterAssetTagGroupsData_struct_198']?.()
  __napiInstance.exports['__napi_register__JsAfterTemplateExecutionData_struct_199']?.()
  __napiInstance.exports['__napi_register__JsBeforeEmitData_struct_200']?.()
  __napiInstance.exports['__napi_register__JsAfterEmitData_struct_201']?.()
  __napiInstance.exports['__napi_register__JsFactoryMeta_struct_202']?.()
  __napiInstance.exports['__napi_register__JsModule_struct_203']?.()
  __napiInstance.exports['__napi_register__JsModule_impl_221']?.()
  __napiInstance.exports['__napi_register__JsExecuteModuleArg_struct_222']?.()
  __napiInstance.exports['__napi_register__JsRuntimeModule_struct_223']?.()
  __napiInstance.exports['__napi_register__JsRuntimeModuleArg_struct_224']?.()
  __napiInstance.exports['__napi_register__JsAddingRuntimeModule_struct_225']?.()
  __napiInstance.exports['__napi_register__JsBuildMeta_struct_226']?.()
  __napiInstance.exports['__napi_register__JsBuildMetaDefaultObjectRedirectWarn_struct_227']?.()
  __napiInstance.exports['__napi_register__JsDefaultObjectRedirectWarnObject_struct_228']?.()
  __napiInstance.exports['__napi_register__JsModuleGraph_struct_229']?.()
  __napiInstance.exports['__napi_register__JsModuleGraph_impl_238']?.()
  __napiInstance.exports['__napi_register__JsModuleGraphConnection_struct_239']?.()
  __napiInstance.exports['__napi_register__JsModuleGraphConnection_impl_242']?.()
  __napiInstance.exports['__napi_register__JsResolveForSchemeArgs_struct_243']?.()
  __napiInstance.exports['__napi_register__JsBeforeResolveArgs_struct_244']?.()
  __napiInstance.exports['__napi_register__JsFactorizeArgs_struct_245']?.()
  __napiInstance.exports['__napi_register__JsResolveArgs_struct_246']?.()
  __napiInstance.exports['__napi_register__JsCreateData_struct_247']?.()
  __napiInstance.exports['__napi_register__JsAfterResolveData_struct_248']?.()
  __napiInstance.exports['__napi_register__JsNormalModuleFactoryCreateModuleArgs_struct_249']?.()
  __napiInstance.exports['__napi_register__JsEntryPluginOptions_struct_250']?.()
  __napiInstance.exports['__napi_register__JsEntryOptions_struct_251']?.()
  __napiInstance.exports['__napi_register__JsLibraryCustomUmdObject_struct_252']?.()
  __napiInstance.exports['__napi_register__JsLibraryName_struct_253']?.()
  __napiInstance.exports['__napi_register__JsLibraryAuxiliaryComment_struct_254']?.()
  __napiInstance.exports['__napi_register__JsLibraryOptions_struct_255']?.()
  __napiInstance.exports['__napi_register__RawAliasOptionItem_struct_256']?.()
  __napiInstance.exports['__napi_register__RawResolveTsconfigOptions_struct_257']?.()
  __napiInstance.exports['__napi_register__RawResolveOptions_struct_258']?.()
  __napiInstance.exports['__napi_register__RawResolveOptionsWithDependencyType_struct_259']?.()
  __napiInstance.exports['__napi_register__JsPathData_struct_260']?.()
  __napiInstance.exports['__napi_register__JsPathDataChunkLike_struct_261']?.()
  __napiInstance.exports['__napi_register__PathWithInfo_struct_262']?.()
  __napiInstance.exports['__napi_register__RawContextReplacementPluginOptions_struct_263']?.()
  __napiInstance.exports['__napi_register__JsLoaderItem_struct_264']?.()
  __napiInstance.exports['__napi_register__JsLoaderState_265']?.()
  __napiInstance.exports['__napi_register__JsLoaderContext_struct_266']?.()
  __napiInstance.exports['__napi_register__JsBannerContentFnCtx_struct_267']?.()
  __napiInstance.exports['__napi_register__RawBannerPluginOptions_struct_268']?.()
  __napiInstance.exports['__napi_register__RawBundlerInfoPluginOptions_struct_269']?.()
  __napiInstance.exports['__napi_register__RawToOptions_struct_270']?.()
  __napiInstance.exports['__napi_register__RawCopyPattern_struct_271']?.()
  __napiInstance.exports['__napi_register__RawInfo_struct_272']?.()
  __napiInstance.exports['__napi_register__RawRelated_struct_273']?.()
  __napiInstance.exports['__napi_register__RawCopyGlobOptions_struct_274']?.()
  __napiInstance.exports['__napi_register__RawCopyRspackPluginOptions_struct_275']?.()
  __napiInstance.exports['__napi_register__RawCssExtractPluginOption_struct_276']?.()
  __napiInstance.exports['__napi_register__RawDllEntryPluginOptions_struct_277']?.()
  __napiInstance.exports['__napi_register__RawLibManifestPluginOptions_struct_278']?.()
  __napiInstance.exports['__napi_register__RawDllReferenceAgencyPluginOptions_struct_279']?.()
  __napiInstance.exports['__napi_register__RawDllManifestContentItem_struct_280']?.()
  __napiInstance.exports['__napi_register__RawDllManifest_struct_281']?.()
  __napiInstance.exports['__napi_register__RawFlagAllModulesAsUsedPluginOptions_struct_282']?.()
  __napiInstance.exports['__napi_register__RawHtmlRspackPluginOptions_struct_283']?.()
  __napiInstance.exports['__napi_register__RawHtmlRspackPluginBaseOptions_struct_284']?.()
  __napiInstance.exports['__napi_register__RawOccurrenceChunkIdsPluginOptions_struct_285']?.()
  __napiInstance.exports['__napi_register__RawIgnorePluginOptions_struct_286']?.()
  __napiInstance.exports['__napi_register__RawModuleInfo_struct_287']?.()
  __napiInstance.exports['__napi_register__RawLazyCompilationOption_struct_288']?.()
  __napiInstance.exports['__napi_register__RawModuleArg_struct_289']?.()
  __napiInstance.exports['__napi_register__RawLightningCssMinimizerRspackPluginOptions_struct_290']?.()
  __napiInstance.exports['__napi_register__RawLightningCssMinimizerOptions_struct_291']?.()
  __napiInstance.exports['__napi_register__RawLightningCssBrowsers_struct_292']?.()
  __napiInstance.exports['__napi_register__RawDraft_struct_293']?.()
  __napiInstance.exports['__napi_register__RawNonStandard_struct_294']?.()
  __napiInstance.exports['__napi_register__RawLightningCssPseudoClasses_struct_295']?.()
  __napiInstance.exports['__napi_register__RawLimitChunkCountPluginOptions_struct_296']?.()
  __napiInstance.exports['__napi_register__RawContainerPluginOptions_struct_297']?.()
  __napiInstance.exports['__napi_register__RawExposeOptions_struct_298']?.()
  __napiInstance.exports['__napi_register__RawContainerReferencePluginOptions_struct_299']?.()
  __napiInstance.exports['__napi_register__RawRemoteOptions_struct_300']?.()
  __napiInstance.exports['__napi_register__RawProvideOptions_struct_301']?.()
  __napiInstance.exports['__napi_register__RawConsumeSharedPluginOptions_struct_302']?.()
  __napiInstance.exports['__napi_register__RawConsumeOptions_struct_303']?.()
  __napiInstance.exports['__napi_register__RawProgressPluginOptions_struct_304']?.()
  __napiInstance.exports['__napi_register__RawRuntimeChunkOptions_struct_305']?.()
  __napiInstance.exports['__napi_register__RawRuntimeChunkNameFnCtx_struct_306']?.()
  __napiInstance.exports['__napi_register__RawSizeLimitsPluginOptions_struct_307']?.()
  __napiInstance.exports['__napi_register__RawExtractComments_struct_308']?.()
  __napiInstance.exports['__napi_register__RawSwcJsMinimizerRspackPluginOptions_struct_309']?.()
  __napiInstance.exports['__napi_register__RawSwcJsMinimizerOptions_struct_310']?.()
  __napiInstance.exports['__napi_register__BuiltinPluginName_311']?.()
  __napiInstance.exports['__napi_register__BuiltinPlugin_struct_312']?.()
  __napiInstance.exports['__napi_register__RawCacheOptions_struct_313']?.()
  __napiInstance.exports['__napi_register__RawPathData_struct_314']?.()
  __napiInstance.exports['__napi_register__RawModuleFilenameTemplateFnCtx_struct_315']?.()
  __napiInstance.exports['__napi_register__RawSourceMapDevToolPluginOptions_struct_316']?.()
  __napiInstance.exports['__napi_register__RawEvalDevToolModulePluginOptions_struct_317']?.()
  __napiInstance.exports['__napi_register__RawEntryDynamicResult_struct_318']?.()
  __napiInstance.exports['__napi_register__RawDynamicEntryPluginOptions_struct_319']?.()
  __napiInstance.exports['__napi_register__RawExperimentSnapshotOptions_struct_320']?.()
  __napiInstance.exports['__napi_register__RawStorageOptions_struct_321']?.()
  __napiInstance.exports['__napi_register__RawExperimentCacheOptionsPersistent_struct_322']?.()
  __napiInstance.exports['__napi_register__RawIncremental_struct_323']?.()
  __napiInstance.exports['__napi_register__RawRspackFuture_struct_324']?.()
  __napiInstance.exports['__napi_register__RawExperiments_struct_325']?.()
  __napiInstance.exports['__napi_register__RawHttpExternalsRspackPluginOptions_struct_326']?.()
  __napiInstance.exports['__napi_register__RawExternalsPluginOptions_struct_327']?.()
  __napiInstance.exports['__napi_register__RawExternalItemFnResult_struct_328']?.()
  __napiInstance.exports['__napi_register__ContextInfo_struct_329']?.()
  __napiInstance.exports['__napi_register__RawExternalItemFnCtx_struct_330']?.()
  __napiInstance.exports['__napi_register__RawExternalItemFnCtxData_struct_331']?.()
  __napiInstance.exports['__napi_register__RawExternalItemFnCtx_impl_334']?.()
  __napiInstance.exports['__napi_register__RawExternalsPresets_struct_335']?.()
  __napiInstance.exports['__napi_register__RawModuleRuleUse_struct_336']?.()
  __napiInstance.exports['__napi_register____rspack_napi__RawRuleSetCondition_struct_337']?.()
  __napiInstance.exports['__napi_register__RawRuleSetConditionType_338']?.()
  __napiInstance.exports['__napi_register__RawRuleSetLogicalConditions_struct_339']?.()
  __napiInstance.exports['__napi_register__RawModuleRule_struct_340']?.()
  __napiInstance.exports['__napi_register__RawParserOptions_struct_341']?.()
  __napiInstance.exports['__napi_register__RawJavascriptParserOptions_struct_342']?.()
  __napiInstance.exports['__napi_register__RawAssetParserOptions_struct_343']?.()
  __napiInstance.exports['__napi_register__RawAssetParserDataUrl_struct_344']?.()
  __napiInstance.exports['__napi_register__RawAssetParserDataUrlOptions_struct_345']?.()
  __napiInstance.exports['__napi_register__RawCssParserOptions_struct_346']?.()
  __napiInstance.exports['__napi_register__RawCssAutoParserOptions_struct_347']?.()
  __napiInstance.exports['__napi_register__RawCssModuleParserOptions_struct_348']?.()
  __napiInstance.exports['__napi_register__RawJsonParserOptions_struct_349']?.()
  __napiInstance.exports['__napi_register__RawGeneratorOptions_struct_350']?.()
  __napiInstance.exports['__napi_register__RawAssetGeneratorOptions_struct_351']?.()
  __napiInstance.exports['__napi_register__RawAssetInlineGeneratorOptions_struct_352']?.()
  __napiInstance.exports['__napi_register__RawAssetResourceGeneratorOptions_struct_353']?.()
  __napiInstance.exports['__napi_register__RawAssetGeneratorDataUrlFnCtx_struct_354']?.()
  __napiInstance.exports['__napi_register__RawAssetGeneratorDataUrlOptions_struct_355']?.()
  __napiInstance.exports['__napi_register__RawCssGeneratorOptions_struct_356']?.()
  __napiInstance.exports['__napi_register__RawCssAutoGeneratorOptions_struct_357']?.()
  __napiInstance.exports['__napi_register__RawCssModuleGeneratorOptions_struct_358']?.()
  __napiInstance.exports['__napi_register__RawModuleOptions_struct_359']?.()
  __napiInstance.exports['__napi_register__RawFuncUseCtx_struct_360']?.()
  __napiInstance.exports['__napi_register__RawNodeOption_struct_361']?.()
  __napiInstance.exports['__napi_register__RawOptimizationOptions_struct_362']?.()
  __napiInstance.exports['__napi_register__RawTrustedTypes_struct_363']?.()
  __napiInstance.exports['__napi_register__RawEnvironment_struct_364']?.()
  __napiInstance.exports['__napi_register__RawOutputOptions_struct_365']?.()
  __napiInstance.exports['__napi_register__JsCacheGroupTestCtx_struct_366']?.()
  __napiInstance.exports['__napi_register__JsChunkOptionNameCtx_struct_367']?.()
  __napiInstance.exports['__napi_register__RawSplitChunkSizes_struct_368']?.()
  __napiInstance.exports['__napi_register__RawSplitChunksOptions_struct_369']?.()
  __napiInstance.exports['__napi_register__RawCacheGroupOptions_struct_370']?.()
  __napiInstance.exports['__napi_register__RawFallbackCacheGroupOptions_struct_371']?.()
  __napiInstance.exports['__napi_register__RawStatsOptions_struct_372']?.()
  __napiInstance.exports['__napi_register__RawOptions_struct_373']?.()
  __napiInstance.exports['__napi_register__JsResolver_struct_374']?.()
  __napiInstance.exports['__napi_register__JsResolver_impl_377']?.()
  __napiInstance.exports['__napi_register__JsResourceData_struct_378']?.()
  __napiInstance.exports['__napi_register__JsRspackDiagnostic_struct_379']?.()
  __napiInstance.exports['__napi_register__JsRspackSeverity_380']?.()
  __napiInstance.exports['__napi_register__JsRspackError_struct_381']?.()
  __napiInstance.exports['__napi_register__JsAdditionalTreeRuntimeRequirementsArg_struct_382']?.()
  __napiInstance.exports['__napi_register__JsRuntimeGlobals_struct_383']?.()
  __napiInstance.exports['__napi_register__JsAdditionalTreeRuntimeRequirementsResult_struct_384']?.()
  __napiInstance.exports['__napi_register__JsRuntimeRequirementInTreeArg_struct_385']?.()
  __napiInstance.exports['__napi_register__JsRuntimeRequirementInTreeResult_struct_386']?.()
  __napiInstance.exports['__napi_register__JsCreateScriptData_struct_387']?.()
  __napiInstance.exports['__napi_register__JsLinkPreloadData_struct_388']?.()
  __napiInstance.exports['__napi_register__JsLinkPrefetchData_struct_389']?.()
  __napiInstance.exports['__napi_register__JsCompatSource_struct_390']?.()
  __napiInstance.exports['__napi_register__JsCompatSourceOwned_struct_391']?.()
  __napiInstance.exports['__napi_register__JsModuleDescriptor_struct_392']?.()
  __napiInstance.exports['__napi_register__JsStatsError_struct_393']?.()
  __napiInstance.exports['__napi_register__JsStatsWarning_struct_394']?.()
  __napiInstance.exports['__napi_register__JsStatsModuleTrace_struct_395']?.()
  __napiInstance.exports['__napi_register__JsStatsModuleTraceModule_struct_396']?.()
  __napiInstance.exports['__napi_register__JsStatsModuleTraceDependency_struct_397']?.()
  __napiInstance.exports['__napi_register__JsStatsLogging_struct_398']?.()
  __napiInstance.exports['__napi_register__JsStatsAsset_struct_399']?.()
  __napiInstance.exports['__napi_register__JsStatsAssetInfo_struct_400']?.()
  __napiInstance.exports['__napi_register__JsStatsAssetInfoRelated_struct_401']?.()
  __napiInstance.exports['__napi_register__JsStatsModuleCommonAttributes_struct_402']?.()
  __napiInstance.exports['__napi_register__JsStatsModule_struct_403']?.()
  __napiInstance.exports['__napi_register__JsStatsModuleProfile_struct_404']?.()
  __napiInstance.exports['__napi_register__JsStatsMillisecond_struct_405']?.()
  __napiInstance.exports['__napi_register__JsStatsModuleIssuer_struct_406']?.()
  __napiInstance.exports['__napi_register__JsStatsModuleReason_struct_407']?.()
  __napiInstance.exports['__napi_register__JsOriginRecord_struct_408']?.()
  __napiInstance.exports['__napi_register__JsStatsSize_struct_409']?.()
  __napiInstance.exports['__napi_register__JsStatsChunk_struct_410']?.()
  __napiInstance.exports['__napi_register__JsStatsChunkGroupAsset_struct_411']?.()
  __napiInstance.exports['__napi_register__JsStatsChunkGroup_struct_412']?.()
  __napiInstance.exports['__napi_register__JsStatsChildGroupChildAssets_struct_413']?.()
  __napiInstance.exports['__napi_register__JsStatsChunkGroupChildren_struct_414']?.()
  __napiInstance.exports['__napi_register__JsStatsOptimizationBailout_struct_415']?.()
  __napiInstance.exports['__napi_register__JsStatsAssetsByChunkName_struct_416']?.()
  __napiInstance.exports['__napi_register__JsStatsOptions_struct_417']?.()
  __napiInstance.exports['__napi_register__JsStatsGetAssets_struct_418']?.()
  __napiInstance.exports['__napi_register__JsStatsCompilation_struct_419']?.()
  __napiInstance.exports['__napi_register__JsStats_struct_420']?.()
  __napiInstance.exports['__napi_register__JsStats_impl_425']?.()
  __napiInstance.exports['__napi_register__JsDiagnosticLocation_struct_0']?.()
  __napiInstance.exports['__napi_register__JsDiagnostic_struct_1']?.()
  __napiInstance.exports['__napi_register__format_diagnostic_2']?.()
  __napiInstance.exports['__napi_register__JsTap_struct_3']?.()
  __napiInstance.exports['__napi_register__RegisterJsTapKind_4']?.()
  __napiInstance.exports['__napi_register__RegisterJsTaps_struct_5']?.()
  __napiInstance.exports['__napi_register__JsResolverFactory_struct_6']?.()
  __napiInstance.exports['__napi_register__JsResolverFactory_impl_9']?.()
  __napiInstance.exports['__napi_register__Rspack_struct_10']?.()
  __napiInstance.exports['__napi_register__Rspack_impl_15']?.()
  __napiInstance.exports['__napi_register__register_global_trace_16']?.()
  __napiInstance.exports['__napi_register__cleanup_global_trace_17']?.()
}
export const EntryDataDto = __napiModule.exports.EntryDataDto
export const EntryDataDTO = __napiModule.exports.EntryDataDTO
export const EntryOptionsDto = __napiModule.exports.EntryOptionsDto
export const EntryOptionsDTO = __napiModule.exports.EntryOptionsDTO
export const JsChunk = __napiModule.exports.JsChunk
export const JsChunkGraph = __napiModule.exports.JsChunkGraph
export const JsChunkGroup = __napiModule.exports.JsChunkGroup
export const JsCompilation = __napiModule.exports.JsCompilation
export const JsContextModuleFactoryAfterResolveData = __napiModule.exports.JsContextModuleFactoryAfterResolveData
export const JsContextModuleFactoryBeforeResolveData = __napiModule.exports.JsContextModuleFactoryBeforeResolveData
export const JsDependencies = __napiModule.exports.JsDependencies
export const JsDependenciesBlock = __napiModule.exports.JsDependenciesBlock
export const JsDependency = __napiModule.exports.JsDependency
export const JsEntries = __napiModule.exports.JsEntries
export const JsExportsInfo = __napiModule.exports.JsExportsInfo
export const JsModule = __napiModule.exports.JsModule
export const JsModuleGraph = __napiModule.exports.JsModuleGraph
export const JsModuleGraphConnection = __napiModule.exports.JsModuleGraphConnection
export const JsResolver = __napiModule.exports.JsResolver
export const JsResolverFactory = __napiModule.exports.JsResolverFactory
export const JsStats = __napiModule.exports.JsStats
export const RawExternalItemFnCtx = __napiModule.exports.RawExternalItemFnCtx
export const Rspack = __napiModule.exports.Rspack
export const BuiltinPluginName = __napiModule.exports.BuiltinPluginName
export const cleanupGlobalTrace = __napiModule.exports.cleanupGlobalTrace
export const formatDiagnostic = __napiModule.exports.formatDiagnostic
export const JsLoaderState = __napiModule.exports.JsLoaderState
export const JsRspackSeverity = __napiModule.exports.JsRspackSeverity
export const RawRuleSetConditionType = __napiModule.exports.RawRuleSetConditionType
export const registerGlobalTrace = __napiModule.exports.registerGlobalTrace
export const RegisterJsTapKind = __napiModule.exports.RegisterJsTapKind
