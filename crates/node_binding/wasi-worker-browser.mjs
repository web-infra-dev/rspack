import { instantiateNapiModuleSync, MessageHandler, WASI, createFsProxy } from '@napi-rs/wasm-runtime'
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

function __patchEmnapiMalloc(instance) {
  const { exports } = instance
  for (const name of ['malloc', '_malloc']) {
    const original = exports[name]
    if (typeof original !== 'function' || original.__rspackEmnapiMallocGuard) {
      continue
    }

    const wrapped = function(size) {
      return __assertEmnapiMemoryRange(original(size), size)
    }
    wrapped.__rspackEmnapiMallocGuard = true
    exports[name] = wrapped
  }
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
    return instantiateNapiModuleSync(wasmModule, {
      childThread: true,
      wasi,
      beforeInit({ instance }) {
        __patchEmnapiMalloc(instance)
      },
      overwriteImports(importObject) {
        importObject.env = {
          ...importObject.env,
          ...importObject.napi,
          ...importObject.emnapi,
          memory: wasmMemory,
        }
      },
    })
  },
  onError(error) {
    postMessage({ type: 'error', error, errorOutputs })
    errorOutputs.length = 0
  }
})

globalThis.onmessage = function (e) {
  handler.handle(e)
}
