import {
  threadId,
  parentPort,
  workerData,
  type MessagePort,
} from 'node:worker_threads';
import {
  beginJsTaskImport,
  endJsTaskImport,
  getJsTaskSubscription,
  registerJsTaskRunner,
  type JsLoaderTask,
} from '@rspack/binding';
import { runLoaders } from '.';
import { getRawResolve } from '../config/adapter';
import { Resolver } from '../Resolver';
import { createWorkerModule } from './workerModule';
import {
  prepareWorkerTask,
  clearWorkerLoaderContext,
  createWorkerCompiler,
  createStrictWorkerCompiler,
  setLoaderObjectBridge,
  setWorkerImportScheduler,
} from './service';

let activeTasks = 0;

async function execute(task: JsLoaderTask): Promise<void> {
  activeTasks++;
  let module: ReturnType<typeof createWorkerModule> | undefined;
  try {
    const strict = task.strict;
    const main = strict
      ? undefined
      : await prepareWorkerTask(task.mainObjectHandle, task.id);
    const context = task.takeContext();
    module = createWorkerModule(context._module!, main?.buildInfo);
    const result = await runLoaders(
      main
        ? createWorkerCompiler(main)
        : createStrictWorkerCompiler(task.mainObjectHandle),
      context,
      main?.extensions ?? {},
      module.module,
      strict
        ? {
            getResolver(options = {}) {
              const { dependencyType, resolveToContext, ...resolve } = options;
              return new Resolver(
                task.getResolver({
                  ...getRawResolve(resolve),
                  dependencyType,
                  resolveToContext,
                }),
              );
            },
          }
        : undefined,
    );
    task.complete(result);
  } catch (error) {
    task.fail(
      error instanceof Error ? (error.stack ?? error.message) : String(error),
    );
  } finally {
    module?.dispose();
    if (--activeTasks === 0) clearWorkerLoaderContext();
  }
}

// An import may recursively build another parallel module on this worker.
function beginImport(): () => void {
  const permit = beginJsTaskImport();
  return () => endJsTaskImport(permit);
}

const { port } = workerData as { port: MessagePort };
setLoaderObjectBridge(port, threadId);
setWorkerImportScheduler(beginImport);
registerJsTaskRunner((_compiler, task) => {
  void execute(task as JsLoaderTask);
}, threadId);
parentPort!.on('message', ({ id }: { id: number }) => {
  try {
    getJsTaskSubscription(true);
    parentPort!.postMessage({ id });
  } catch (error) {
    parentPort!.postMessage({ id, error: String(error) });
  }
});
parentPort!.postMessage({ loaded: true });
