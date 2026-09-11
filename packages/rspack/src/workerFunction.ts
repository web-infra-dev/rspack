/** Internal wire format. Functions are prepared before entering synchronous user code. */
export const WORKER_FUNCTION_MARKER = '__rspack_worker_function__';

export type WorkerFunctionDescriptor = {
  version: 1;
  execution: 'worker';
  target: string;
  options: any;
  module?: { path: string; type?: string };
};

const descriptors = new WeakMap<Function, WorkerFunctionDescriptor>();

/**
 * Executes a module export with options appended to its arguments.
 * Native worker scheduling currently supports NormalModuleFactory.beforeResolve.tapPromise.
 */
export function workerFunction<Fn extends Function>(
  target: string,
  options: any,
): Fn {
  if (IS_BROWSER)
    throw new Error('workerFunction is not supported in browsers');
  if (typeof target !== 'string' || !target) {
    throw new TypeError('workerFunction target must be a non-empty string');
  }
  return createWorkerFunctionPlaceholder({
    version: 1,
    execution: 'worker',
    target,
    options,
  }) as unknown as Fn;
}

export function getWorkerFunctionDescriptor(
  value: unknown,
): WorkerFunctionDescriptor | undefined {
  return typeof value === 'function' ? descriptors.get(value) : undefined;
}

export function validateWorkerFunctionDescriptor(
  value: any,
): asserts value is WorkerFunctionDescriptor {
  if (
    !value ||
    value.version !== 1 ||
    value.execution !== 'worker' ||
    typeof value.target !== 'string' ||
    !value.target ||
    (value.module !== undefined &&
      (!value.module ||
        typeof value.module.path !== 'string' ||
        !value.module.path ||
        (value.module.type !== undefined &&
          value.module.type !== 'module' &&
          value.module.type !== 'commonjs')))
  ) {
    throw new TypeError(
      'Invalid workerFunction descriptor (expected version 1)',
    );
  }
}

export function createWorkerFunctionPlaceholder(
  descriptor: WorkerFunctionDescriptor,
): Function {
  validateWorkerFunctionDescriptor(descriptor);
  const placeholder = function () {
    throw new Error(
      'workerFunction must be prepared by Rspack before use; hook scheduling only supports NormalModuleFactory.beforeResolve.tapPromise',
    );
  };
  descriptors.set(placeholder, descriptor);
  return placeholder;
}

export function markResolvedWorkerFunction(
  fn: Function,
  descriptor: WorkerFunctionDescriptor,
): void {
  descriptors.set(fn, descriptor);
}
