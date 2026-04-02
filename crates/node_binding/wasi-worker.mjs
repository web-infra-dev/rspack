import fs from "node:fs";
import { createRequire } from "node:module";
import { parse } from "node:path";
import { WASI } from "node:wasi";
import { parentPort, Worker } from "node:worker_threads";

const require = createRequire(import.meta.url);

const { MessageHandler, getDefaultContext } = require("@napi-rs/wasm-runtime");
const { createNapiModule: __emnapiCreateNapiModule, loadNapiModuleSync: __emnapiLoadNapiModuleSync } = require("@emnapi/core");

if (parentPort) {
  parentPort.on("message", (data) => {
    globalThis.onmessage({ data });
  });
}

Object.assign(globalThis, {
  self: globalThis,
  require,
  Worker,
  importScripts: function (f) {
    ;(0, eval)(fs.readFileSync(f, "utf8") + "//# sourceURL=" + f);
  },
  postMessage: function (msg) {
    if (parentPort) {
      parentPort.postMessage(msg);
    }
  },
});

const emnapiContext = getDefaultContext();

const __rootDir = parse(process.cwd()).root;

const __EMNAPI_MEMORY_LIMIT = 0x80000000;

function __assertEmnapiMemoryRange(ptr, size) {
  if (ptr === 0) {
    return ptr;
  }

  const start = Number(ptr);
  const allocationSize = Number(size) >>> 0;
  const end = start + allocationSize;

  if (!(start < __EMNAPI_MEMORY_LIMIT && end <= __EMNAPI_MEMORY_LIMIT)) {
    throw new Error(
      `emnapi malloc exceeded the 2 GiB memory limit: start=0x${start.toString(16)}, size=0x${allocationSize.toString(16)}, end=0x${end.toString(16)}, limit=0x${__EMNAPI_MEMORY_LIMIT.toString(16)}`
    );
  }

  return ptr;
}

function __wrapEmnapiInstance(instance) {
  const wrappedExports = Object.create(instance.exports);
  for (const name of ["malloc", "_malloc"]) {
    const original = instance.exports[name];
    if (typeof original !== "function") {
      continue;
    }

    Object.defineProperty(wrappedExports, name, {
      value: function (size) {
        return __assertEmnapiMemoryRange(original(size), size);
      },
      enumerable: true,
      configurable: true,
      writable: true,
    });
    wrappedExports[name].__rspackEmnapiMallocGuard = true;
  }

  return {
    exports: wrappedExports,
  };
}

function __createPatchedNapiModule(options) {
  const napiModule = __emnapiCreateNapiModule(options);
  const originalInit = napiModule.init;
  napiModule.init = function (initOptions) {
    return originalInit.call(this, {
      ...initOptions,
      instance: __wrapEmnapiInstance(initOptions.instance),
    });
  };
  return napiModule;
}

const handler = new MessageHandler({
  onLoad({ wasmModule, wasmMemory }) {
    const wasi = new WASI({
      version: 'preview1',
      env: process.env,
      preopens: {
        [__rootDir]: __rootDir,
      },
    });

    const __emnapiOptions = {
      childThread: true,
      wasi,
      context: emnapiContext,
      overwriteImports(importObject) {
        importObject.env = {
          ...importObject.env,
          ...importObject.napi,
          ...importObject.emnapi,
          memory: wasmMemory
        };
      },
    };
    return __emnapiLoadNapiModuleSync(
      __createPatchedNapiModule(__emnapiOptions),
      wasmModule,
      __emnapiOptions
    );
  },
});

globalThis.onmessage = function (e) {
  handler.handle(e);
};
