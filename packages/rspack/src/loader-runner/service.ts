import { cpus } from 'node:os';
import path from 'node:path';
import { deserialize, serialize } from 'node:v8';
import { MessageChannel, type MessagePort, Worker } from 'node:worker_threads';

import { dispatchJsLoaderTask } from '@rspack/binding';

interface WorkerSlot {
  worker: Worker;
  mainPort: MessagePort;
  mainSyncPort: MessagePort;
}

interface ActiveTask {
  handleIncomingRequest: HandleIncomingRequest;
  pendingRequests: Map<number, Promise<any>>;
}

interface RunOptions {
  handleIncomingRequest: HandleIncomingRequest;
  transferList?: readonly Transferable[];
}

interface WorkerResult {
  ok: boolean;
  data?: WorkerArgs;
  error?: WorkerError;
}

let workerPool: Promise<void> | undefined;
let nextTaskId = 1;
let shuttingDown = false;
const workerSlots = new Map<number, WorkerSlot>();
const activeTasks = new Map<number, ActiveTask>();

const handleRequest = (port: MessagePort, message: WorkerRequestMessage) => {
  const task = activeTasks.get(message.taskId);
  if (!task) {
    port.postMessage({
      type: 'response-error',
      id: message.id,
      error: serializeError(
        new Error(`No active loader task found for id ${message.taskId}`),
      ),
    } satisfies WorkerResponseErrorMessage);
    return;
  }

  task.pendingRequests.set(
    message.id,
    Promise.resolve()
      .then(() =>
        task.handleIncomingRequest(message.requestType, ...message.data),
      )
      .then((result) => {
        port.postMessage({
          type: 'response',
          id: message.id,
          data: result,
        } satisfies WorkerResponseMessage);
        return result;
      })
      .catch((error) => {
        port.postMessage({
          type: 'response-error',
          id: message.id,
          error: serializeError(error),
        } satisfies WorkerResponseErrorMessage);
      }),
  );
};

const handleSyncRequest = async (
  port: MessagePort,
  message: WorkerRequestSyncMessage,
) => {
  const sharedBufferView = new Int32Array(message.sharedBuffer);

  try {
    const task = activeTasks.get(message.taskId);
    if (!task) {
      throw new Error(`No active loader task found for id ${message.taskId}`);
    }

    let result: any;
    switch (message.requestType) {
      case RequestSyncType.WaitForPendingRequest: {
        const pendingRequestId = message.data[0];
        const isArray = Array.isArray(pendingRequestId);
        const ids = isArray ? pendingRequestId : [pendingRequestId];
        result = await Promise.all(
          ids.map((id) => task.pendingRequests.get(id)),
        );
        if (!isArray) result = result[0];
        break;
      }
      default:
        throw new Error(`Unknown request type: ${message.requestType}`);
    }

    port.postMessage({
      type: 'response',
      id: message.id,
      data: result,
    } satisfies WorkerResponseMessage);
  } catch (error) {
    port.postMessage({
      type: 'response-error',
      id: message.id,
      error: serializeError(error),
    } satisfies WorkerResponseErrorMessage);
  } finally {
    Atomics.add(sharedBufferView, 0, 1);
    Atomics.notify(sharedBufferView, 0, Number.POSITIVE_INFINITY);
  }
};

const spawnWorker = (slotId: number): Promise<void> => {
  const { port1: mainPort, port2: workerPort } = new MessageChannel();
  const { port1: mainSyncPort, port2: workerSyncPort } = new MessageChannel();
  const worker = new Worker(path.resolve(import.meta.dirname, 'worker.js'), {
    workerData: {
      rspackNativeLoaderWorker: true,
      workerPort,
      workerSyncPort,
    },
    transferList: [workerPort, workerSyncPort],
  });
  const slot = { worker, mainPort, mainSyncPort };
  workerSlots.set(slotId, slot);

  worker.unref();
  mainPort.unref();
  mainSyncPort.unref();

  return new Promise<void>((resolve, reject) => {
    let ready = false;
    const closePorts = () => {
      mainPort.close();
      mainSyncPort.close();
    };

    mainPort.on('message', (message: WorkerMessage) => {
      if (message.type === 'ready') {
        ready = true;
        resolve();
      } else if (message.type === 'init-error') {
        reject(message.error);
      } else if (message.type === 'request') {
        handleRequest(mainPort, message);
      }
    });
    mainPort.on('messageerror', (error) => {
      if (!ready) reject(error);
    });
    // eslint-disable-next-line @typescript-eslint/no-misused-promises
    mainSyncPort.on('message', (message: WorkerRequestSyncMessage) =>
      handleSyncRequest(mainSyncPort, message),
    );
    mainSyncPort.on('messageerror', (error) => {
      if (!ready) reject(error);
    });
    worker.on('error', (error) => {
      if (!ready) reject(error);
    });
    worker.on('exit', (code) => {
      closePorts();
      if (workerSlots.get(slotId) === slot) workerSlots.delete(slotId);
      if (!ready) {
        reject(
          new Error(`Loader worker exited during startup with code ${code}`),
        );
      }
      if (!shuttingDown) {
        void spawnWorker(slotId).catch(() => {
          // The next loader dispatch reports that no consumer is available.
        });
      }
    });
  });
};

const ensureLoaderWorkers = (workerOptions?: {
  maxWorkers?: number;
}): Promise<void> => {
  if (workerPool) return workerPool;

  const availableThreads = Math.max(cpus().length - 1, 1);
  const maxWorkers = workerOptions?.maxWorkers
    ? Math.max(workerOptions.maxWorkers, 1)
    : undefined;
  const maxWorkersFromEnv = parseInt(
    process.env.RSPACK_LOADER_WORKER_THREADS || '',
    10,
  );
  const workerCount = maxWorkers || maxWorkersFromEnv || availableThreads;
  shuttingDown = false;
  workerPool = Promise.all(
    Array.from({ length: workerCount }, (_, slotId) => spawnWorker(slotId)),
  ).then(() => undefined);
  return workerPool;
};

export interface WorkerResponseMessage {
  type: 'response';
  id: number;
  data: any;
}

export interface WorkerResponseErrorMessage {
  type: 'response-error';
  id: number;
  error: WorkerError;
}

interface WorkerReadyMessage {
  type: 'ready';
}

interface WorkerInitErrorMessage {
  type: 'init-error';
  error: WorkerError;
}

export interface WorkerRequestMessage {
  type: 'request';
  taskId: number;
  id: number;
  requestType: RequestType;
  data: WorkerArgs;
}

export interface WorkerRequestSyncMessage {
  type: 'request-sync';
  taskId: number;
  id: number;
  requestType: RequestSyncType;
  data: WorkerArgs;
  sharedBuffer: SharedArrayBuffer;
}

export type WorkerMessage =
  | WorkerResponseMessage
  | WorkerRequestMessage
  | WorkerResponseErrorMessage
  | WorkerRequestSyncMessage
  | WorkerReadyMessage
  | WorkerInitErrorMessage;

export function isWorkerResponseMessage(
  message: WorkerMessage,
): message is WorkerResponseMessage {
  return message.type === 'response';
}

export function isWorkerResponseErrorMessage(
  message: WorkerMessage,
): message is WorkerResponseErrorMessage {
  return message.type === 'response-error';
}

export enum RequestType {
  AddDependency = 'AddDependency',
  AddContextDependency = 'AddContextDependency',
  AddMissingDependency = 'AddMissingDependency',
  AddBuildDependency = 'AddBuildDependency',
  GetDependencies = 'GetDependencies',
  GetContextDependencies = 'GetContextDependencies',
  GetMissingDependencies = 'GetMissingDependencies',
  ClearDependencies = 'ClearDependencies',
  Resolve = 'Resolve',
  GetResolve = 'GetResolve',
  GetLogger = 'GetLogger',
  EmitError = 'EmitError',
  EmitWarning = 'EmitWarning',
  EmitFile = 'EmitFile',
  EmitDiagnostic = 'EmitDiagnostic',
  SetCacheable = 'SetCacheable',
  ImportModule = 'ImportModule',
  UpdateLoaderObjects = 'UpdateLoaderObjects',
  LoaderCacheGet = 'LoaderCacheGet',
  LoaderCacheStore = 'LoaderCacheStore',
  CompilationGetPath = 'CompilationGetPath',
  CompilationGetPathWithInfo = 'CompilationGetPathWithInfo',
  CompilationGetAssetPath = 'CompilationGetAssetPath',
  CompilationGetAssetPathWithInfo = 'CompilationGetAssetPathWithInfo',
}

export enum RequestSyncType {
  WaitForPendingRequest = 'WaitForPendingRequest',
}

export type HandleIncomingRequest = (
  requestType: RequestType,
  ...args: any[]
) => any;

type WorkerArgs = any[];
export type WorkerError = Error;

export function serializeError(error: unknown): WorkerError {
  if (
    error instanceof Error ||
    (error && typeof error === 'object' && 'message' in error)
  ) {
    return {
      ...error,
      name: (error as Error).name,
      stack: (error as Error).stack,
      message: (error as Error).message,
    };
  }
  if (typeof error === 'string') {
    return { name: 'Error', message: error };
  }
  throw new Error(
    'Failed to serialize error, only string, Error instances and objects with a message property are supported',
  );
}

function checkCloneableProps(obj: any, loaderName: string) {
  const errors = [];
  for (const key of Object.keys(obj)) {
    try {
      structuredClone(obj[key]);
    } catch (error: any) {
      errors.push({ key, type: typeof obj[key], reason: error.message });
    }
  }
  if (errors.length > 0) {
    const errorMsg = errors
      .map(
        (error) =>
          `option "${error.key}" (type: ${error.type}) is not cloneable: ${error.reason}`,
      )
      .join('\n');
    throw new Error(
      `The options for ${loaderName} are not cloneable, which is not supported by parallelLoader. Consider disabling parallel for this loader or removing the non-cloneable properties from the options:\n${errorMsg}`,
    );
  }
}

export const run = async (
  loaderName: string,
  task: any,
  options: RunOptions,
  workerOptions?: { maxWorkers?: number },
): Promise<WorkerArgs> => {
  checkCloneableProps(task, loaderName);
  await ensureLoaderWorkers(workerOptions);

  const taskId = nextTaskId++;
  const pendingRequests = new Map<number, Promise<any>>();
  activeTasks.set(taskId, {
    handleIncomingRequest: options.handleIncomingRequest,
    pendingRequests,
  });

  try {
    const payload = await dispatchJsLoaderTask(serialize({ taskId, task }));
    const result = deserialize(payload) as WorkerResult;
    await Promise.allSettled(pendingRequests.values());
    if (!result.ok) throw result.error;
    return result.data || [];
  } finally {
    activeTasks.delete(taskId);
  }
};
