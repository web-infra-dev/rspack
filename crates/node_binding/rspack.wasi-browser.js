import {
  createOnMessage as __wasmCreateOnMessageForFsProxy,
  getDefaultContext as __emnapiGetDefaultContext,
  instantiateNapiModule as __emnapiInstantiateNapiModule,
  WASI as __WASI,
} from '@napi-rs/wasm-runtime'
import { memfs, Buffer } from '@napi-rs/wasm-runtime/fs'


export const { fs: __fs, vol: __volume } = memfs()

const __wasi = new __WASI({
  version: 'preview1',
  fs: __fs,
  preopens: {
    '/': '/',
  },
})

const __wasmUrl = new URL('./rspack.wasm32-wasi.wasm', import.meta.url).href
const __emnapiContext = __emnapiGetDefaultContext()
__emnapiContext.feature.Buffer = Buffer

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
} = await __emnapiInstantiateNapiModule(__wasmFile, {
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
    for (const name of Object.keys(instance.exports)) {
      if (name.startsWith('__napi_register__')) {
        instance.exports[name]()
      }
    }
  },
})
export default __napiModule.exports
export const Assets = __napiModule.exports.Assets
export const AsyncDependenciesBlock = __napiModule.exports.AsyncDependenciesBlock
export const Chunk = __napiModule.exports.Chunk
export const ChunkGraph = __napiModule.exports.ChunkGraph
export const ChunkGroup = __napiModule.exports.ChunkGroup
export const Chunks = __napiModule.exports.Chunks
export const CodeGenerationResult = __napiModule.exports.CodeGenerationResult
export const CodeGenerationResults = __napiModule.exports.CodeGenerationResults
export const ConcatenatedModule = __napiModule.exports.ConcatenatedModule
export const ContextModule = __napiModule.exports.ContextModule
export const Dependency = __napiModule.exports.Dependency
export const Diagnostics = __napiModule.exports.Diagnostics
export const EntryDataDto = __napiModule.exports.EntryDataDto
export const EntryDataDTO = __napiModule.exports.EntryDataDTO
export const EntryDependency = __napiModule.exports.EntryDependency
export const EntryOptionsDto = __napiModule.exports.EntryOptionsDto
export const EntryOptionsDTO = __napiModule.exports.EntryOptionsDTO
export const ExternalModule = __napiModule.exports.ExternalModule
export const JsCompilation = __napiModule.exports.JsCompilation
export const JsCompiler = __napiModule.exports.JsCompiler
export const JsContextModuleFactoryAfterResolveData = __napiModule.exports.JsContextModuleFactoryAfterResolveData
export const JsContextModuleFactoryBeforeResolveData = __napiModule.exports.JsContextModuleFactoryBeforeResolveData
export const JsCoordinator = __napiModule.exports.JsCoordinator
export const JsDependencies = __napiModule.exports.JsDependencies
export const JsEntries = __napiModule.exports.JsEntries
export const JsExportsInfo = __napiModule.exports.JsExportsInfo
export const JsModuleGraph = __napiModule.exports.JsModuleGraph
export const JsResolver = __napiModule.exports.JsResolver
export const JsResolverFactory = __napiModule.exports.JsResolverFactory
export const JsStats = __napiModule.exports.JsStats
export const KnownBuildInfo = __napiModule.exports.KnownBuildInfo
export const Module = __napiModule.exports.Module
export const ModuleGraphConnection = __napiModule.exports.ModuleGraphConnection
export const NativeWatcher = __napiModule.exports.NativeWatcher
export const NativeWatchResult = __napiModule.exports.NativeWatchResult
export const NormalModule = __napiModule.exports.NormalModule
export const RawExternalItemFnCtx = __napiModule.exports.RawExternalItemFnCtx
export const ReadonlyResourceData = __napiModule.exports.ReadonlyResourceData
export const ResolverFactory = __napiModule.exports.ResolverFactory
export const Sources = __napiModule.exports.Sources
export const VirtualFileStore = __napiModule.exports.VirtualFileStore
export const JsVirtualFileStore = __napiModule.exports.JsVirtualFileStore
export const async = __napiModule.exports.async
export const BuiltinPluginName = __napiModule.exports.BuiltinPluginName
export const cleanupGlobalTrace = __napiModule.exports.cleanupGlobalTrace
export const EnforceExtension = __napiModule.exports.EnforceExtension
export const EXPECTED_RSPACK_CORE_VERSION = __napiModule.exports.EXPECTED_RSPACK_CORE_VERSION
export const formatDiagnostic = __napiModule.exports.formatDiagnostic
export const JsLoaderState = __napiModule.exports.JsLoaderState
export const JsRspackSeverity = __napiModule.exports.JsRspackSeverity
export const loadBrowserslist = __napiModule.exports.loadBrowserslist
export const minify = __napiModule.exports.minify
export const minifySync = __napiModule.exports.minifySync
export const RawJavascriptParserCommonjsExports = __napiModule.exports.RawJavascriptParserCommonjsExports
export const RawRuleSetConditionType = __napiModule.exports.RawRuleSetConditionType
export const registerGlobalTrace = __napiModule.exports.registerGlobalTrace
export const RegisterJsTapKind = __napiModule.exports.RegisterJsTapKind
export const sync = __napiModule.exports.sync
export const syncTraceEvent = __napiModule.exports.syncTraceEvent
export const transform = __napiModule.exports.transform
export const transformSync = __napiModule.exports.transformSync
