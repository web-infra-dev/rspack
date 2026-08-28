import { isMainThread, parentPort, workerData } from 'node:worker_threads';

import { recvWorkerTask } from '@rspack/binding';
import { createWorkerLoaderCompiler, runLoaders } from '.';
import {
  clearWorkerLoaderBridgeData,
  setLoaderFunctionBridge,
} from './service';

async function runWorkerLoop(): Promise<never> {
  setLoaderFunctionBridge(workerData.workerFunctionPort);
  parentPort?.postMessage({ type: 'rspack-loader-worker-ready' });
  while (true) {
    const task = await recvWorkerTask();
    const context = task.takeContext();
    try {
      const result = await runLoaders(
        createWorkerLoaderCompiler(context, task),
        context,
        true,
      );
      task.complete(result);
    } catch (error) {
      task.fail(
        error instanceof Error ? (error.stack ?? error.message) : String(error),
      );
    } finally {
      clearWorkerLoaderBridgeData();
    }
  }
}

if (!isMainThread && workerData?.rspackNativeLoaderWorker) {
  void runWorkerLoop().catch((error) => {
    queueMicrotask(() => {
      throw error;
    });
  });
}
