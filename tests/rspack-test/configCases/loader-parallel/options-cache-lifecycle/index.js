const { serialize } = require('node:v8');
const { Worker } = require('node:worker_threads');
const path = require('node:path');
const { getWorkerOptionsCache, releaseWorkerOptions, reuseWorkerOptions, registerCompilerWorkerOptions, releaseCompilerWorkerOptions } = require('./helper');

it('invalidates changed and removed snapshots and separates owners', () => {
  const cache = getWorkerOptionsCache(91);
  const first = reuseWorkerOptions(cache, 'reference', { nested: { value: 1 } });
  expect(reuseWorkerOptions(cache, 'reference', { nested: { value: 1 } })).toBe(first);
  const changed = reuseWorkerOptions(cache, 'reference', { nested: { value: 2 } });
  expect(changed).not.toBe(first);
  expect(changed.nested.value).toBe(2);
  expect(reuseWorkerOptions(getWorkerOptionsCache(92), 'reference', { nested: { value: 2 } })).not.toBe(changed);
  cache.delete('reference');
  expect(reuseWorkerOptions(cache, 'reference', { nested: { value: 2 } })).not.toBe(changed);
  const shared = { data: new SharedArrayBuffer(4) };
  expect(reuseWorkerOptions(cache, 'reference', shared)).toBe(shared);
  let calls = 0;
  const compiler = cache.get('compiler', serialize({ mode: 'development' }), () => { calls++; return {}; });
  expect(cache.get('compiler', serialize({ mode: 'development' }), () => { calls++; return {}; })).toBe(compiler);
  expect(calls).toBe(1);
  expect(cache.get('compiler', serialize({ mode: 'production' }), () => ({}))).not.toBe(compiler);
  releaseWorkerOptions(91);
  expect(getWorkerOptionsCache(91)).not.toBe(cache);
  releaseWorkerOptions(91);
  releaseWorkerOptions(92);
});

it('releases the cached owner in other workers', async () => {
  const worker = new Worker(`
    const { parentPort, workerData } = require('node:worker_threads');
    const { getWorkerOptionsCache } = require(workerData.helper);
    const cache = getWorkerOptionsCache(93);
    let attempts = 0;
    const timer = setInterval(() => {
      if (getWorkerOptionsCache(93) !== cache) {
        clearInterval(timer);
        parentPort.postMessage('released');
      } else if (++attempts > 500) {
        clearInterval(timer);
        throw new Error('Worker options were not released');
      }
    }, 10);
    parentPort.postMessage('ready');
  `, { eval: true, workerData: { helper: path.join(__dirname, 'helper.js') } });
  try {
    await new Promise((resolve, reject) => {
      worker.on('error', reject);
      worker.on('message', message => {
        if (message === 'ready') releaseWorkerOptions(93);
        else if (message === 'released') resolve();
      });
    });
  } finally {
    await worker.terminate();
  }
});

it('cleans child owners with the root while keeping other compilers isolated', () => {
  const root = {}; root.root = root;
  const child = { root };
  const other = {}; other.root = other;
  registerCompilerWorkerOptions(root, 94);
  registerCompilerWorkerOptions(child, 95);
  registerCompilerWorkerOptions(other, 96);
  const rootCache = getWorkerOptionsCache(94);
  const childCache = getWorkerOptionsCache(95);
  const otherCache = getWorkerOptionsCache(96);
  releaseCompilerWorkerOptions(child);
  expect(getWorkerOptionsCache(94)).toBe(rootCache);
  expect(getWorkerOptionsCache(95)).not.toBe(childCache);
  registerCompilerWorkerOptions(child, 95);
  const nextChildCache = getWorkerOptionsCache(95);
  releaseCompilerWorkerOptions(root);
  expect(getWorkerOptionsCache(94)).not.toBe(rootCache);
  expect(getWorkerOptionsCache(95)).not.toBe(nextChildCache);
  expect(getWorkerOptionsCache(96)).toBe(otherCache);
  releaseCompilerWorkerOptions(child);
  releaseCompilerWorkerOptions(other);
  releaseWorkerOptions(94);
  releaseWorkerOptions(95);
});
