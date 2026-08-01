import { createWorker as createOtherWorker } from './other.js';

export function callOtherWorker() {
  return createOtherWorker();
}

export function createWorker() {
  return new Worker(
    /* webpackChunkName: "worker" */ new URL('./worker.js', import.meta.url),
  );
}
