import { isMainThread } from 'node:worker_threads';
export default async (value, options) => {
  if (isMainThread) throw new Error('Nested workerFunction used main-isolate RPC');
  return value + options.suffix;
};
