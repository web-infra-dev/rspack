// Compact chunk ids have at most 13 hash characters, so 14 module-less
// facades with the same empty module-derived name reproduce the exhaustion.
const workerFactories = [
  () => new Worker(new URL('./worker.js?00', import.meta.url)),
  () => new Worker(new URL('./worker.js?01', import.meta.url)),
  () => new Worker(new URL('./worker.js?02', import.meta.url)),
  () => new Worker(new URL('./worker.js?03', import.meta.url)),
  () => new Worker(new URL('./worker.js?04', import.meta.url)),
  () => new Worker(new URL('./worker.js?05', import.meta.url)),
  () => new Worker(new URL('./worker.js?06', import.meta.url)),
  () => new Worker(new URL('./worker.js?07', import.meta.url)),
  () => new Worker(new URL('./worker.js?08', import.meta.url)),
  () => new Worker(new URL('./worker.js?09', import.meta.url)),
  () => new Worker(new URL('./worker.js?10', import.meta.url)),
  () => new Worker(new URL('./worker.js?11', import.meta.url)),
  () => new Worker(new URL('./worker.js?12', import.meta.url)),
  () => new Worker(new URL('./worker.js?13', import.meta.url)),
];
// Ordinary dynamic imports are async chunk groups, not async entrypoints. Even
// two blocks targeting the same module must not turn their emptied chunks into
// facades after splitChunks moves that module into `workers`.
const loadAsync = () => import('./async.js');
const loadAsyncAgain = () => import('./async.js');

it('preserves worker facades while removing ordinary empty chunks', () => {
  expect(workerFactories).toHaveLength(14);
  expect(loadAsync).toBeInstanceOf(Function);
  expect(loadAsyncAgain).toBeInstanceOf(Function);
});
