if (import.meta.main) {
  globalThis.__workerIsMain = true;
}

await Promise.resolve();

export default 'worker-async';

globalThis.__workerAsyncEntryExecuted = true;
