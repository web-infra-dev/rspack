import { threadId, type MessagePort } from 'node:worker_threads';
import {
  beginJsTaskImport,
  endJsTaskImport,
  getJsTaskSubscription,
  registerJsTaskRunner,
  type JsLoaderContext,
  type JsResolver,
  type RawResolveOptionsWithDependencyType,
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

async function executeLoader(
  context: JsLoaderContext,
  compilerId: number,
  getResolver?: (options: RawResolveOptionsWithDependencyType) => JsResolver,
): Promise<JsLoaderContext> {
  const usesObjectBridge = !getResolver;
  if (usesObjectBridge) activeBridgeTasks++;
  let module: ReturnType<typeof createWorkerModule> | undefined;
  try {
    const main = usesObjectBridge
      ? await getWorkerLoaderContext(compilerId, context)
      : undefined;
    const workerModule = createWorkerModule(context._module!, main);
    module = workerModule;
    return await runLoaders(
      main?.compiler ?? createStrictWorkerCompiler(compilerId),
      context,
      workerModule.module,
      getResolver
        ? {
            getResolver(options = {}) {
              const { dependencyType, resolveToContext, ...resolve } = options;
              return new Resolver(
                getResolver({
                  ...getRawResolve(resolve),
                  dependencyType,
                  resolveToContext,
                }),
              );
            },
          }
        : undefined,
    ).finally(workerModule.commitBuildInfo);
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
    registerJsTaskRunner((task) => {
      switch (task.type) {
        case 'loader':
          return executeLoader(task.context, task.compilerId, task.getResolver);
      }
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
          getJsTaskSubscription();
          control.postMessage({ id });
        } catch (error) {
          control.postMessage({ id, error: String(error) });
        }
      }
    });
    control.postMessage({ threadId });
  });
}
