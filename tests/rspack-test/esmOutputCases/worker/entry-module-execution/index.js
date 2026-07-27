export function createWorker() {
  return new Worker(new URL('./worker.js', import.meta.url));
}

export function createCommonJsWorker() {
  return new Worker(new URL('./worker-cjs.js', import.meta.url));
}

export function createAsyncWorker() {
  return new Worker(new URL('./worker-async.js', import.meta.url));
}
