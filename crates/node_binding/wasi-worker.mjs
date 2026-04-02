import fs from "node:fs";
import { createRequire } from "node:module";
import { parse } from "node:path";
import { WASI } from "node:wasi";
import { parentPort, Worker } from "node:worker_threads";

const require = createRequire(import.meta.url);

const { instantiateNapiModuleSync, MessageHandler, getDefaultContext } = require("@napi-rs/wasm-runtime");

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

function __patchEmnapiMalloc(instance) {
  const { exports } = instance;
  for (const name of ["malloc", "_malloc"]) {
    const original = exports[name];
    if (typeof original !== "function" || original.__rspackEmnapiMallocGuard) {
      continue;
    }

    const wrapped = function (size) {
      return __assertEmnapiMemoryRange(original(size), size);
    };
    wrapped.__rspackEmnapiMallocGuard = true;
    exports[name] = wrapped;
  }
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

    return instantiateNapiModuleSync(wasmModule, {
      childThread: true,
      wasi,
      context: emnapiContext,
      beforeInit({ instance }) {
        __patchEmnapiMalloc(instance);
      },
      overwriteImports(importObject) {
        importObject.env = {
          ...importObject.env,
          ...importObject.napi,
          ...importObject.emnapi,
          memory: wasmMemory
        };
      },
    });
  },
});

globalThis.onmessage = function (e) {
  handler.handle(e);
};
