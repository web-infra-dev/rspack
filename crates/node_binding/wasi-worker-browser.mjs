import { MessageHandler, WASI, createFsProxy } from '@napi-rs/wasm-runtime'
import {
  createNapiModule as __emnapiCreateNapiModule,
  loadNapiModuleSync as __emnapiLoadNapiModuleSync,
} from '@emnapi/core'
import { memfsExported as __memfsExported } from '@napi-rs/wasm-runtime/fs'

const fs = createFsProxy(__memfsExported)

const errorOutputs = []

const __EMNAPI_MEMORY_LIMIT = 0x80000000

function __assertEmnapiMemoryRange(ptr, size) {
  if (ptr === 0) {
    return ptr
  }

  const start = Number(ptr)
  const allocationSize = Number(size) >>> 0
  const end = start + allocationSize

  if (!(start < __EMNAPI_MEMORY_LIMIT && end <= __EMNAPI_MEMORY_LIMIT)) {
    throw new Error(
      `emnapi malloc exceeded the 2 GiB memory limit: start=0x${start.toString(16)}, size=0x${allocationSize.toString(16)}, end=0x${end.toString(16)}, limit=0x${__EMNAPI_MEMORY_LIMIT.toString(16)}`
    )
  }

  return ptr
}

function __wrapEmnapiInstance(instance) {
  const wrappedExports = Object.create(instance.exports)
  for (const name of ['malloc', '_malloc']) {
    const original = instance.exports[name]
    if (typeof original !== 'function') {
      continue
    }

    Object.defineProperty(wrappedExports, name, {
      value: function(size) {
        return __assertEmnapiMemoryRange(original(size), size)
      },
      enumerable: true,
      configurable: true,
      writable: true,
    })
    wrappedExports[name].__rspackEmnapiMallocGuard = true
  }

  return {
    exports: wrappedExports,
  }
}

function __createPatchedNapiModule(options) {
  const napiModule = __emnapiCreateNapiModule(options)
  const originalInit = napiModule.init
  napiModule.init = function(initOptions) {
    return originalInit.call(this, {
      ...initOptions,
      instance: __wrapEmnapiInstance(initOptions.instance),
    })
  }
  return napiModule
}

const handler = new MessageHandler({
  onLoad({ wasmModule, wasmMemory }) {
    const wasi = new WASI({
      fs,
      preopens: {
        '/': '/',
      },
      print: function () {
        // eslint-disable-next-line no-console
        console.log.apply(console, arguments)
      },
      printErr: function() {
        // eslint-disable-next-line no-console
        console.error.apply(console, arguments)
        
        errorOutputs.push([...arguments])
      },
    })
    const __emnapiOptions = {
      childThread: true,
      wasi,
      overwriteImports(importObject) {
        importObject.env = {
          ...importObject.env,
          ...importObject.napi,
          ...importObject.emnapi,
          memory: wasmMemory,
        }
      },
    }
    return __emnapiLoadNapiModuleSync(
      __createPatchedNapiModule(__emnapiOptions),
      wasmModule,
      __emnapiOptions
    )
  },
  onError(error) {
    postMessage({ type: 'error', error, errorOutputs })
    errorOutputs.length = 0
  }
})

globalThis.onmessage = function (e) {
  handler.handle(e)
}
