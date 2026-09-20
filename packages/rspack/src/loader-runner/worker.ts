import { threadId, type MessagePort } from 'node:worker_threads';
import {
  cancelWorkerReceive,
  recvWorkerTask,
  type JsLoaderTask,
} from '@rspack/binding';
import { runLoaders } from '.';
import {
  type WorkerLoaderContext,
  prepareWorkerTask,
  clearWorkerLoaderContext,
  createWorkerCompiler,
  getMainObject,
  setLoaderObjectBridge,
  setWorkerImportScheduler,
} from './service';

let nextReceive = 1;
let activeTasks = 0;

async function execute(task: JsLoaderTask): Promise<void> {
  activeTasks++;
  try {
    await prepareWorkerTask(task.mainObjectHandle, task.id);
    const context = task.takeContext();
    const main = getMainObject<WorkerLoaderContext>(context.bridgeHandle!);
    const nativeModule = context._module;
    context._module = main.module;
    const result = await runLoaders(
      createWorkerCompiler(main),
      context,
      main.extensions,
    );
    result._module = nativeModule;
    task.complete(result);
  } catch (error) {
    task.fail(
      error instanceof Error ? (error.stack ?? error.message) : String(error),
    );
  } finally {
    if (--activeTasks === 0) clearWorkerLoaderContext();
  }
}

// An import can recursively build another parallel module. While its loader is
// awaiting main, the same isolate must be able to service that module even with
// maxWorkers: 1. Cancel the extra receive when the import callback settles.
function beginImport(): () => void {
  let stopped = false;
  let receive = 0;
  const loop = async () => {
    while (!stopped) {
      receive = nextReceive++;
      const task = await recvWorkerTask(threadId, receive);
      if (task) await execute(task);
    }
  };
  void loop().catch((error) => {
    throw error;
  });
  return () => {
    stopped = true;
    cancelWorkerReceive(threadId, receive);
  };
}

async function run(): Promise<void> {
  while (true) {
    const task = await recvWorkerTask(threadId, nextReceive++);
    if (task) await execute(task);
  }
}

let initialized = false;
export default async function initialize({
  port,
}: {
  port: MessagePort;
}): Promise<void> {
  port.postMessage({ id: threadId, initialized });
  await new Promise<void>((resolve) => port.once('message', () => resolve()));
  if (initialized) {
    port.close();
    return;
  }
  setLoaderObjectBridge(port, threadId);
  setWorkerImportScheduler(beginImport);
  initialized = true;
  void run().catch((error) => {
    throw error;
  });
}
