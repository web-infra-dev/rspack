import { parentPort } from 'node:worker_threads';
import run from './lifecycle.mjs';

// Native build callbacks do not keep the worker event loop alive.
parentPort.ref();
await run();
parentPort.postMessage('ok');
parentPort.close();
