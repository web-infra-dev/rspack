import { childIsMain } from "./worker-child";

if (import.meta.main) {
  globalThis.__workerIsMain = true;
}

if (childIsMain) {
  throw new Error("an imported module must not be marked as the entry");
}

await Promise.resolve();

if (!globalThis.__workerIsMain) {
  throw new Error('worker entry should be the main module');
}

export default 'worker-async';

globalThis.__workerAsyncEntryExecuted = true;
