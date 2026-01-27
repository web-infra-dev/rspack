import querystring from 'node:querystring';
import { promisify } from 'node:util';
import { type MessagePort, receiveMessageOnPort } from 'node:worker_threads';

import { JsLoaderState, type NormalModule } from '@rspack/binding';
import type { LoaderContext } from '../config';

import { createHash } from '../util/createHash';
import { absolutify, contextify } from '../util/identifier';
import { memoize } from '../util/memoize';
import loadLoader from './loadLoader';
import {
  isWorkerResponseErrorMessage,
  isWorkerResponseMessage,
  RequestSyncType,
  RequestType,
  serializeError,
  type WorkerError,
  type WorkerMessage,
  type WorkerRequestMessage,
  type WorkerRequestSyncMessage,
  type WorkerResponseErrorMessage,
  type WorkerResponseMessage,
} from './service';
import { convertArgs, runSyncOrAsync } from './utils';

const BUILTIN_LOADER_PREFIX = 'builtin:';

interface WorkerOptions {
  loaderContext: LoaderContext;
  loaderState: JsLoaderState;
  args: any[];

  workerData?: {
    workerPort: MessagePort;
    workerSyncPort: MessagePort;
  };
}

const loadLoaderAsync: (
  loaderObject: any,
  loaderContext: any,
) => Promise<void> = promisify(loadLoader);

function dirname(path: string) {
  if (path === '/') return '/';
  const i = path.lastIndexOf('/');
  const j = path.lastIndexOf('\\');
  const i2 = path.indexOf('/');
  const j2 = path.indexOf('\\');
  const idx = i > j ? i : j;
  const idx2 = i > j ? i2 : j2;
  if (idx < 0) return path;
  if (idx === idx2) return path.slice(0, idx + 1);
  return path.slice(0, idx);
}

async function loaderImpl(
  { args, loaderContext, loaderState }: WorkerOptions,
  sendRequest: SendRequestFunction,
  waitForPendingRequest: WaitForPendingRequestFunction,
) {
  //
  const resourcePath = loaderContext.resourcePath;
  const contextDirectory = resourcePath ? dirname(resourcePath) : null;

  const pendingDependencyRequest: SendRequestResult[] = [];

  // @ts-expect-error `loaderContext.parallel` only works with loaders in worker
  loaderContext.parallel = true;
  loaderContext.dependency = loaderContext.addDependency =
    function addDependency(file) {
      pendingDependencyRequest.push(
        sendRequest(RequestType.AddDependency, file),
      );
    };
  loaderContext.addContextDependency = function addContextDependency(context) {
    pendingDependencyRequest.push(
      sendRequest(RequestType.AddContextDependency, context),
    );
  };
  loaderContext.addBuildDependency = function addBuildDependency(file) {
    pendingDependencyRequest.push(
      sendRequest(RequestType.AddBuildDependency, file),
    );
  };
  loaderContext.getDependencies = function getDependencies() {
    waitForPendingRequest(pendingDependencyRequest);
    return sendRequest(RequestType.GetDependencies).wait();
  };
  loaderContext.getContextDependencies = function getContextDependencies() {
    waitForPendingRequest(pendingDependencyRequest);
    return sendRequest(RequestType.GetContextDependencies).wait();
  };
  loaderContext.getMissingDependencies = function getMissingDependencies() {
    waitForPendingRequest(pendingDependencyRequest);
    return sendRequest(RequestType.GetMissingDependencies).wait();
  };
  loaderContext.clearDependencies = function clearDependencies() {
    pendingDependencyRequest.push(sendRequest(RequestType.ClearDependencies));
  };
  loaderContext.resolve = function resolve(context, request, callback) {
    sendRequest(RequestType.Resolve, context, request).then(
      (result) => {
        callback(null, result);
      },
      (err) => {
        callback(err);
      },
    );
  };
  loaderContext.getResolve = function getResolve(options) {
    return (context, request, callback) => {
      if (!callback) {
        return new Promise((resolve, reject) => {
          sendRequest(RequestType.GetResolve, options, context, request).then(
            (result) => {
              resolve(result);
            },
            (err) => {
              reject(err);
            },
          );
        });
      }
      sendRequest(RequestType.GetResolve, options, context, request).then(
        (result) => {
          callback(null, result);
        },
        (err) => {
          callback(err);
        },
      );
    };
  };
  loaderContext.getLogger = function getLogger(name) {
    return {
      error(...args) {
        sendRequest(RequestType.GetLogger, 'error', name, args);
      },
      warn(...args) {
        sendRequest(RequestType.GetLogger, 'warn', name, args);
      },
      info(...args) {
        sendRequest(RequestType.GetLogger, 'info', name, args);
      },
      log(...args) {
        sendRequest(RequestType.GetLogger, 'log', name, args);
      },
      debug(...args) {
        sendRequest(RequestType.GetLogger, 'debug', name, args);
      },
      assert(assertion, ...args) {
        if (!assertion) {
          sendRequest(RequestType.GetLogger, 'error', name, args);
        }
      },
      trace() {
        sendRequest(RequestType.GetLogger, 'trace', name, ['Trace']);
      },
      clear() {
        sendRequest(RequestType.GetLogger, 'clear', name);
      },
      status(...args) {
        sendRequest(RequestType.GetLogger, 'status', name, args);
      },
      group(...args) {
        sendRequest(RequestType.GetLogger, 'group', name, args);
      },
      groupCollapsed(...args) {
        sendRequest(RequestType.GetLogger, 'groupCollapsed', name, args);
      },
      groupEnd(...args) {
        sendRequest(RequestType.GetLogger, 'groupEnd', name, args);
      },
      profile(label) {
        sendRequest(RequestType.GetLogger, 'profile', name, [label]);
      },
      profileEnd(label) {
        sendRequest(RequestType.GetLogger, 'profileEnd', name, [label]);
      },
      time(label) {
        sendRequest(RequestType.GetLogger, 'time', name, [label]);
      },
      timeEnd(label) {
        sendRequest(RequestType.GetLogger, 'timeEnd', name, [label]);
      },
      timeLog(label, ...args) {
        sendRequest(RequestType.GetLogger, 'timeLog', name, [label, ...args]);
      },
      timeAggregate(label) {
        sendRequest(RequestType.GetLogger, 'timeAggregate', name, [label]);
      },
      timeAggregateEnd(label) {
        sendRequest(RequestType.GetLogger, 'timeAggregateEnd', name, [label]);
      },
    };
  } as LoaderContext['getLogger'];

  loaderContext.emitError = function emitError(err) {
    sendRequest(RequestType.EmitError, serializeError(err));
  };
  loaderContext.emitWarning = function emitWarning(warning) {
    sendRequest(RequestType.EmitWarning, serializeError(warning));
  };
  loaderContext.emitFile = function emitFile(
    name,
    content,
    sourceMap,
    assetInfo,
  ) {
    sendRequest(RequestType.EmitFile, name, content, sourceMap, assetInfo);
  };
  loaderContext.experiments = {
    emitDiagnostic(diagnostic) {
      sendRequest(RequestType.EmitDiagnostic, diagnostic);
    },
  };

  const getAbsolutify = memoize(() => absolutify.bindCache({}));
  const getAbsolutifyInContext = memoize(() =>
    absolutify.bindContextCache(contextDirectory!, {}),
  );
  const getContextify = memoize(() => contextify.bindCache({}));
  const getContextifyInContext = memoize(() =>
    contextify.bindContextCache(contextDirectory!, {}),
  );

  loaderContext.utils = {
    absolutify: (context, request) => {
      return context === contextDirectory
        ? getAbsolutifyInContext()(request)
        : getAbsolutify()(context, request);
    },
    contextify: (context, request) => {
      return context === contextDirectory
        ? getContextifyInContext()(request)
        : getContextify()(context, request);
    },
    createHash: (type) => {
      return createHash(
        type || loaderContext._compilation.outputOptions.hashFunction!,
      );
    },
  };

  loaderContext._compiler = {
    ...loaderContext._compiler,
    rspack: {
      // @ts-expect-error: some properties are missing.
      experiments: {
        swc: require('../swc'),
      },
    },
    // @ts-expect-error: some properties are missing.
    webpack: {
      util: {
        createHash: require('../util/createHash').createHash,
        cleverMerge: require('../util/cleverMerge').cleverMerge,
      },
    },
  };

  loaderContext._compilation = {
    ...loaderContext._compilation,
    getPath(filename, data) {
      return sendRequest(RequestType.CompilationGetPath, filename, data).wait();
    },
    getPathWithInfo(filename, data) {
      return sendRequest(
        RequestType.CompilationGetPathWithInfo,
        filename,
        data,
      ).wait();
    },
    getAssetPath(filename, data) {
      return sendRequest(
        RequestType.CompilationGetAssetPath,
        filename,
        data,
      ).wait();
    },
    getAssetPathWithInfo(filename, data) {
      return sendRequest(
        RequestType.CompilationGetAssetPathWithInfo,
        filename,
        data,
      ).wait();
    },
  } as LoaderContext['_compilation'];

  const _module = loaderContext._module as any;
  loaderContext._module = {
    type: _module.type,
    identifier() {
      return _module.identifier;
    },
    matchResource: _module.matchResource,
    request: _module.request,
    userRequest: _module.userRequest,
    rawRequest: _module.rawRequest,
  } as NormalModule;

  // @ts-expect-error
  loaderContext.importModule = function importModule(
    request,
    options,
    callback,
  ) {
    if (!callback) {
      return new Promise((resolve, reject) => {
        sendRequest(RequestType.ImportModule, request, options).then(
          (result) => {
            resolve(result);
          },
          (err) => {
            reject(err);
          },
        );
      });
    }
    sendRequest(RequestType.ImportModule, request, options).then(
      (result) => {
        callback(null, result);
      },
      (err) => {
        callback(err);
      },
    );
  };

  loaderContext.fs = require('node:fs');

  Object.defineProperty(loaderContext, 'request', {
    enumerable: true,
    get: () =>
      loaderContext.loaders
        .map((o) => o.request)
        .concat(loaderContext.resource || '')
        .join('!'),
  });
  Object.defineProperty(loaderContext, 'remainingRequest', {
    enumerable: true,
    get: () => {
      if (
        loaderContext.loaderIndex >= loaderContext.loaders.length - 1 &&
        !loaderContext.resource
      )
        return '';
      return loaderContext.loaders
        .slice(loaderContext.loaderIndex + 1)
        .map((o) => o.request)
        .concat(loaderContext.resource || '')
        .join('!');
    },
  });
  Object.defineProperty(loaderContext, 'currentRequest', {
    enumerable: true,
    get: () =>
      loaderContext.loaders
        .slice(loaderContext.loaderIndex)
        .map((o) => o.request)
        .concat(loaderContext.resource || '')
        .join('!'),
  });
  Object.defineProperty(loaderContext, 'previousRequest', {
    enumerable: true,
    get: () =>
      loaderContext.loaders
        .slice(0, loaderContext.loaderIndex)
        .map((o) => o.request)
        .join('!'),
  });
  Object.defineProperty(loaderContext, 'query', {
    enumerable: true,
    get: () => {
      const entry = loaderContext.loaders[loaderContext.loaderIndex];
      return entry.options && typeof entry.options === 'object'
        ? entry.options
        : entry.query;
    },
  });

  loaderContext.getOptions = function getOptions() {
    const loader = getCurrentLoader(loaderContext);
    let options = loader?.options;

    if (typeof options === 'string') {
      if (options.startsWith('{') && options.endsWith('}')) {
        try {
          options = JSON.parse(options);
        } catch (e: any) {
          throw new Error(
            `JSON parsing failed for loader's string options: ${e.message}`,
          );
        }
      } else {
        options = querystring.parse(options);
      }
    }

    if (options === null || options === undefined) {
      options = {};
    }

    return options;
  };

  loaderContext.cacheable = function cacheable(flag?: boolean) {
    if (flag === false) {
      sendRequest(RequestType.SetCacheable, false);
    }
  };

  Object.defineProperty(loaderContext, 'data', {
    enumerable: true,
    get: () => loaderContext.loaders[loaderContext.loaderIndex].loaderItem.data,
    set: (value) => {
      loaderContext.loaders[loaderContext.loaderIndex].loaderItem.data = value;
    },
  });

  const shouldYieldToMainThread = (currentLoaderObject: any) => {
    if (!currentLoaderObject?.parallel) {
      return true;
    }
    if (currentLoaderObject?.request.startsWith(BUILTIN_LOADER_PREFIX)) {
      return true;
    }
    return false;
  };

  // Execute loader list until the current loader object is to yield to the main
  // thread.  This happens if the loader is marked as non-parallel or if it is a
  // builtin loader which belongs to the rust side.
  switch (loaderState) {
    case JsLoaderState.Pitching: {
      while (loaderContext.loaderIndex < loaderContext.loaders.length) {
        const currentLoaderObject =
          loaderContext.loaders[loaderContext.loaderIndex];
        if (shouldYieldToMainThread(currentLoaderObject)) break;
        if (currentLoaderObject.pitchExecuted) {
          loaderContext.loaderIndex += 1;
          continue;
        }

        await loadLoaderAsync(currentLoaderObject, loaderContext._compiler);
        const fn = currentLoaderObject.pitch;
        currentLoaderObject.pitchExecuted = true;
        if (!fn) continue;

        args =
          (await runSyncOrAsync(fn, loaderContext, [
            loaderContext.remainingRequest,
            loaderContext.previousRequest,
            currentLoaderObject.loaderItem.data,
          ])) || [];

        const hasArg = args.some((value) => value !== undefined);
        if (hasArg) {
          break;
        }
      }
      break;
    }
    case JsLoaderState.Normal: {
      while (loaderContext.loaderIndex >= 0) {
        const currentLoaderObject =
          loaderContext.loaders[loaderContext.loaderIndex];

        if (shouldYieldToMainThread(currentLoaderObject)) break;
        if (currentLoaderObject.normalExecuted) {
          loaderContext.loaderIndex--;
          continue;
        }

        await loadLoaderAsync(currentLoaderObject, loaderContext._compiler);
        const fn = currentLoaderObject.normal;
        currentLoaderObject.normalExecuted = true;
        if (!fn) continue;
        convertArgs(args, !!currentLoaderObject.raw);
        args = (await runSyncOrAsync(fn, loaderContext, args)) || [];
      }
    }
  }

  sendRequest(
    RequestType.UpdateLoaderObjects,
    loaderContext.loaders.map((item) => {
      return {
        data: item.loaderItem.data,
        normalExecuted: item.normalExecuted,
        pitchExecuted: item.pitchExecuted,
      };
    }),
  );

  return args;
}

let nextId = 0;
const responseCallbacks: Record<
  number,
  (err: WorkerError | null, data: any) => void
> = {};

function handleIncomingResponses(workerMessage: WorkerMessage) {
  if (isWorkerResponseMessage(workerMessage)) {
    const { id, data } = workerMessage;
    const callback = responseCallbacks[id];
    if (callback) {
      delete responseCallbacks[id];
      callback(null, /* data */ data);
    } else {
      throw new Error(`No callback found for response with id ${id}`);
    }
  } else if (isWorkerResponseErrorMessage(workerMessage)) {
    const { id, error } = workerMessage;
    const callback = responseCallbacks[id];
    if (callback) {
      delete responseCallbacks[id];
      callback(error, undefined);
    } else {
      throw new Error(`No callback found for response with id ${id}`);
    }
  }
}

type SendRequestResult<T = any> = Promise<T> & {
  // Put current thread into sleep until the request is resolved.
  // Should not return error in `wait`
  //
  // Pending requests now are not returning errors.
  // To handle errors, you should not call `wait()` on send request
  // result;
  //
  // You should use `sendRequest` directly or use `sendRequest.sync`.
  wait: () => T;
  // The request Id
  id: number;
};

interface SendRequestFunction {
  <T = any>(requestType: RequestType, ...args: any[]): SendRequestResult<T>;
  sync<T = any>(requestType: RequestSyncType, ...args: any[]): T;
}

type WaitForPendingRequestFunction = (
  id: SendRequestResult[] | SendRequestResult,
) => any;

function createWaitForPendingRequest(
  sendRequest: SendRequestFunction,
): WaitForPendingRequestFunction {
  return (requests: SendRequestResult[] | SendRequestResult) => {
    return sendRequest.sync(
      RequestSyncType.WaitForPendingRequest,
      (Array.isArray(requests) ? requests : [requests]).map((request) => {
        return request.id;
      }),
    );
  };
}

function createSendRequest(
  workerPort: MessagePort,
  workerSyncPort: MessagePort,
): SendRequestFunction {
  const sendRequest = ((requestType, ...args) => {
    const id = nextId++;
    workerPort.postMessage({
      type: 'request',
      id,
      requestType,
      data: args,
    } satisfies WorkerRequestMessage);
    const result = new Promise((resolve, reject) => {
      responseCallbacks[id] = (err, data) => {
        if (err) {
          reject(err);
          return;
        }
        resolve(data);
      };
    }) as SendRequestResult;
    result.wait = () => {
      return sendRequest.sync(RequestSyncType.WaitForPendingRequest, id);
    };
    result.id = id;
    return result;
  }) as SendRequestFunction;
  sendRequest.sync = createSendRequestSync(workerSyncPort);
  return sendRequest;
}

function createSendRequestSync(workerSyncPort: MessagePort) {
  return (requestType: RequestSyncType, ...args: any[]) => {
    const id = nextId++;

    // Create `sharedArrayBuffer` for each request.
    // This is used to synchronize between the main thread and worker thread.
    const sharedBuffer = new SharedArrayBuffer(8);
    const sharedBufferView = new Int32Array(sharedBuffer);

    workerSyncPort.postMessage({
      type: 'request-sync',
      id,
      requestType,
      data: args,
      sharedBuffer,
    } satisfies WorkerRequestSyncMessage);

    // Atomics.wait returns immediately with the value 'not-equal'
    // Otherwise, the thread is blocked until another thread calls Atomics.notify
    // with the same memory location or the timeout is reached.
    //
    // See: https://v8.dev/features/atomics
    const status = Atomics.wait(sharedBufferView, 0, 0);
    if (status !== 'ok' && status !== 'not-equal')
      throw new Error(`Internal error: Atomics.wait() failed: ${status}`);

    const {
      message,
    }: { message: WorkerResponseMessage | WorkerResponseErrorMessage } =
      receiveMessageOnPort(workerSyncPort)!;

    if (id !== message.id) {
      throw new Error(`Unexpected response id: ${message.id}, expected: ${id}`);
    }

    if (isWorkerResponseMessage(message)) {
      return message.data;
    }

    throw message.error;
  };
}

function worker(workerOptions: WorkerOptions) {
  const workerData = workerOptions.workerData!;
  delete workerOptions.workerData;

  workerData.workerPort.on('message', handleIncomingResponses);
  const sendRequest = createSendRequest(
    workerData.workerPort,
    workerData.workerSyncPort,
  );
  const waitFor = createWaitForPendingRequest(sendRequest);

  loaderImpl(workerOptions, sendRequest, waitFor)
    .then((data) => {
      workerData.workerPort.postMessage({ type: 'done', data });
    })
    .catch((err) => {
      workerData.workerPort.postMessage({
        type: 'done-error',
        error: serializeError(err),
      });
    });
}

function getCurrentLoader(
  loaderContext: LoaderContext,
  index = loaderContext.loaderIndex,
) {
  if (
    loaderContext.loaders?.length &&
    index < loaderContext.loaders.length &&
    index >= 0 &&
    loaderContext.loaders[index]
  ) {
    return loaderContext.loaders[index];
  }
  return null;
}

export default worker;
