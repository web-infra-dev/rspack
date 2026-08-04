module.exports = 'worker-cjs';

if (!import.meta.main) {
  throw new Error('wrapped CommonJS worker entry should be the main module');
}

globalThis.__workerCommonJsEntryExecuted = true;
