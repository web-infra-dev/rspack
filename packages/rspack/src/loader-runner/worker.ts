import { threadId, type MessagePort } from 'node:worker_threads';
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
  clearWorkerLoaderReferences,
  createStrictWorkerCompiler,
  getWorkerLoaderContext,
  setLoaderObjectBridge,
  setWorkerImportScheduler,
} from './service';

let activeBridgeTasks = 0;

async function execute(task: JsLoaderTask): Promise<void> {
  let usesObjectBridge = false;
  let module: ReturnType<typeof createWorkerModule> | undefined;
  try {
    const strict = task.strict;
    const context = task.takeContext();
    if (!strict) {
      usesObjectBridge = true;
      activeBridgeTasks++;
    }
    const main = strict
      ? undefined
      : await getWorkerLoaderContext(task.mainObjectHandle, context);
    const workerModule = createWorkerModule(context._module!, main);
    module = workerModule;
    const result = await runLoaders(
      main?.compiler ?? createStrictWorkerCompiler(task.mainObjectHandle),
      context,
      workerModule.module,
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
    ).finally(workerModule.commitBuildInfo);
    task.complete(result);
  } catch (error) {
    task.fail(
      error instanceof Error ? (error.stack ?? error.message) : String(error),
    );
  } finally {
    module?.dispose();
    if (usesObjectBridge && --activeBridgeTasks === 0)
      clearWorkerLoaderReferences();
  }
}

// An import may recursively build another parallel module on this worker.
function beginImport(): () => void {
  const permit = beginJsTaskImport();
  return () => endJsTaskImport(permit);
}

let initialized = false;

// Tinypool runs only this startup handshake. Loader tasks arrive through MPMC.
export default async function initialize({
  port,
  control,
}: {
  port: MessagePort;
  control: MessagePort;
}): Promise<void> {
  const reused = initialized;
  if (!reused) {
    setLoaderObjectBridge(port, threadId);
    setWorkerImportScheduler(beginImport);
    registerJsTaskRunner((_compiler, task) => {
      void execute(task as JsLoaderTask);
    }, threadId);
    initialized = true;
  } else {
    port.close();
  }
  await new Promise<void>((resolve) => {
    control.on('message', ({ type, id }: { type: string; id?: number }) => {
      if (type === 'initialized') {
        if (reused) control.close();
        else control.unref();
        resolve();
        return;
      }
      if (type === 'subscribe') {
        try {
          getJsTaskSubscription(true);
          control.postMessage({ id });
        } catch (error) {
          control.postMessage({ id, error: String(error) });
        }
      }
    });
    control.postMessage({ threadId });
  });
}
