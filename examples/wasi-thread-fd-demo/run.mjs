import { readFileSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { Worker } from 'node:worker_threads';
import { WASI } from 'node:wasi';

const rootDir = resolve('/').replace(/\\/g, '/');
const wasmPath = resolve('rust-demo/target/wasm32-wasip1-threads/ci/wasi_thread_fd_demo.wasm');
const demoFilePath = '/tmp/rspack-wasi-thread-fd-demo.txt';

writeFileSync(demoFilePath, 'rspack wasm wasi fd namespace demo\n');

const wasmBytes = readFileSync(wasmPath);
const module = await WebAssembly.compile(wasmBytes);

const sharedMemory = new WebAssembly.Memory({
  initial: 17,
  maximum: 32,
  shared: true,
});

function createWasi() {
  return new WASI({
    version: 'preview1',
    preopens: {
      [rootDir]: rootDir,
    },
  });
}

function instantiateOnThread() {
  const wasi = createWasi();
  const importObject = wasi.getImportObject();
  importObject.env = {
    ...(importObject.env ?? {}),
    memory: sharedMemory,
  };
  return WebAssembly.instantiate(module, importObject).then((instance) => {
    wasi.initialize(instance);
    return instance.exports.demo_open_keep();
  });
}

const mainFd = await instantiateOnThread();

const workerFd = await new Promise((resolvePromise, rejectPromise) => {
  const worker = new Worker(new URL('./worker.mjs', import.meta.url), {
    workerData: {
      wasmBytes,
      memory: sharedMemory,
      rootDir,
    },
  });
  worker.once('message', resolvePromise);
  worker.once('error', rejectPromise);
  worker.once('exit', (code) => {
    if (code !== 0) {
      rejectPromise(new Error(`worker exited with code ${code}`));
    }
  });
});

console.log(`main fd: ${mainFd}`);
console.log(`worker fd: ${workerFd}`);

if (mainFd === workerFd) {
  console.log('duplicate fd values across threads: per-thread WASI fd namespace confirmed');
} else {
  console.log('fd values differ; this runtime did not reproduce the mismatch');
  process.exitCode = 1;
}
