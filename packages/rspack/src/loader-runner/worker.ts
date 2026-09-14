import { isMainThread, parentPort, workerData } from 'node:worker_threads';

import { recvWorkerTask } from '@rspack/binding';
import { createWorkerLoaderCompiler, runLoaders } from '.';
import {
  clearWorkerLoaderBridgeData,
  setLoaderFunctionBridge,
  deserializeLoaderOptions,
  prepareWorkerFunctionValue,
} from './service';

async function runWorkerLoop(): Promise<never> {
  setLoaderFunctionBridge(workerData.workerFunctionPort);
  parentPort?.postMessage({ type: 'rspack-loader-worker-ready' });
  while (true) {
    const task = await recvWorkerTask();
    try {
      if (task.kind === 'function') {
        const { functions, data } = task.takeFunction();
        let result: false | undefined;
        for (const item of functions) {
          if (
            item.version !== 1 ||
            item.hook !== 'NormalModuleFactory.beforeResolve'
          ) {
            throw new Error('Unsupported workerFunction hook codec');
          }
          const fn = await prepareWorkerFunctionValue(
            deserializeLoaderOptions(item.value),
          );
          const value = await fn(data);
          if (value !== undefined && value !== false) {
            throw new TypeError(
              'workerFunction beforeResolve must return false or undefined',
            );
          }
          if (value === false) {
            result = false;
            break;
          }
        }
        task.completeFunction(data, result);
      } else if (task.kind === 'splitChunkName') {
        const { function: item, data } = task.takeSplitChunkName();
        if (
          item.version !== 1 ||
          item.hook !== 'optimization.splitChunks.name'
        ) {
          throw new Error('Unsupported workerFunction splitChunks.name codec');
        }
        const fn = await prepareWorkerFunctionValue(
          deserializeLoaderOptions(item.value),
        );
        const snapshot = JSON.parse(data) as {
          module: {
            identifier: string;
            nameForCondition: string | null;
            type: string;
            layer: string | null;
          };
          chunks: { name: string | null }[];
          cacheGroupKey: string;
        };
        const module = Object.freeze({
          identifier: () => snapshot.module.identifier,
          nameForCondition: () => snapshot.module.nameForCondition ?? undefined,
          type: snapshot.module.type,
          layer: snapshot.module.layer ?? undefined,
        });
        const chunks = Object.freeze(
          snapshot.chunks.map((chunk) =>
            Object.freeze({ name: chunk.name ?? undefined }),
          ),
        );
        const result = await fn(module, chunks, snapshot.cacheGroupKey);
        if (result !== undefined && typeof result !== 'string') {
          throw new TypeError(
            'workerFunction splitChunks.name must return a string or undefined',
          );
        }
        task.completeSplitChunkName(result);
      } else {
        const context = task.takeContext();
        const result = await runLoaders(
          createWorkerLoaderCompiler(context, task),
          context,
          true,
        );
        task.complete(result);
      }
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
