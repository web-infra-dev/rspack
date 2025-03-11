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
    for (const name of Object.keys(instance.exports)) {
      if (name.startsWith('__napi_register__')) {
        instance.exports[name]()
      }
    }
  },
})
export const Dependency = __napiModule.exports.Dependency
export const EntryDataDto = __napiModule.exports.EntryDataDto
export const EntryDataDTO = __napiModule.exports.EntryDataDTO
export const EntryDependency = __napiModule.exports.EntryDependency
export const EntryOptionsDto = __napiModule.exports.EntryOptionsDto
export const EntryOptionsDTO = __napiModule.exports.EntryOptionsDTO
export const JsChunk = __napiModule.exports.JsChunk
export const JsChunkGraph = __napiModule.exports.JsChunkGraph
export const JsChunkGroup = __napiModule.exports.JsChunkGroup
export const JsCompilation = __napiModule.exports.JsCompilation
export const JsCompiler = __napiModule.exports.JsCompiler
export const JsContextModuleFactoryAfterResolveData = __napiModule.exports.JsContextModuleFactoryAfterResolveData
export const JsContextModuleFactoryBeforeResolveData = __napiModule.exports.JsContextModuleFactoryBeforeResolveData
export const JsDependencies = __napiModule.exports.JsDependencies
export const JsDependenciesBlock = __napiModule.exports.JsDependenciesBlock
export const JsEntries = __napiModule.exports.JsEntries
export const JsExportsInfo = __napiModule.exports.JsExportsInfo
export const JsModule = __napiModule.exports.JsModule
export const JsModuleGraph = __napiModule.exports.JsModuleGraph
export const JsModuleGraphConnection = __napiModule.exports.JsModuleGraphConnection
export const JsResolver = __napiModule.exports.JsResolver
export const JsResolverFactory = __napiModule.exports.JsResolverFactory
export const JsStats = __napiModule.exports.JsStats
export const RawExternalItemFnCtx = __napiModule.exports.RawExternalItemFnCtx
export const BuiltinPluginName = __napiModule.exports.BuiltinPluginName
export const cleanupGlobalTrace = __napiModule.exports.cleanupGlobalTrace
export const formatDiagnostic = __napiModule.exports.formatDiagnostic
export const JsLoaderState = __napiModule.exports.JsLoaderState
export const JsRspackSeverity = __napiModule.exports.JsRspackSeverity
export const RawRuleSetConditionType = __napiModule.exports.RawRuleSetConditionType
export const registerGlobalTrace = __napiModule.exports.registerGlobalTrace
export const RegisterJsTapKind = __napiModule.exports.RegisterJsTapKind
