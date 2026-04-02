import { parentPort, workerData } from 'node:worker_threads';
import { WASI } from 'node:wasi';

const { wasmBytes, memory, rootDir } = workerData;

const wasi = new WASI({
  version: 'preview1',
  preopens: {
    [rootDir]: rootDir,
  },
});

const importObject = wasi.getImportObject();
importObject.env = {
  ...(importObject.env ?? {}),
  memory,
};

const module = await WebAssembly.compile(wasmBytes);
const instance = await WebAssembly.instantiate(module, importObject);
wasi.initialize(instance);
parentPort.postMessage(instance.exports.demo_open_keep());
