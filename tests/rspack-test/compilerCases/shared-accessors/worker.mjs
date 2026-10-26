import { parentPort } from 'node:worker_threads';
import { createRequire } from 'node:module';
import run from './lifecycle.mjs';

// Native build callbacks do not keep the worker event loop alive.
parentPort.ref();
await run();
parentPort.postMessage({
  status: 'ok',
  bindings: Object.keys(createRequire(import.meta.url).cache).filter(file =>
    /[/\\]rspack\.[^/\\]+\.node$/.test(file),
  ),
});
parentPort.close();
