if (import.meta.main) {
  globalThis.__workerIsMain = true;
}

await Promise.resolve();

if (!globalThis.__workerIsMain) {
  throw new Error('worker entry should be the main module');
}

export default 'worker-async';

globalThis.__workerAsyncEntryExecuted = true;
