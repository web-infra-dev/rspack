import { HookBase, HookMap } from '@rspack/lite-tapable';
import { getWorkerFunctionDescriptor } from '../workerFunction';

/** Validate at registration, including taps registered through withOptions or HookMap.for. */
export function guardWorkerFunctionHooks(
  hooks: object,
  supported?: string,
): void {
  for (const [name, value] of Object.entries(hooks)) {
    if (value instanceof HookMap) {
      value.intercept({
        factory: (_key, hook) => {
          guardWorkerFunctionHooks({ hook });
          return hook!;
        },
      });
    } else if (value instanceof HookBase) {
      const tap = value._tap;
      value._tap = function (type, options, fn) {
        if (getWorkerFunctionDescriptor(fn)) {
          if (name !== supported || type !== 'promise') {
            throw new Error(
              'workerFunction hook scheduling only supports NormalModuleFactory.beforeResolve.tapPromise',
            );
          }
          if (this.interceptors.length) {
            throw new Error(
              'workerFunction beforeResolve does not support JavaScript hook interceptors',
            );
          }
        }
        return tap.call(this, type, options, fn);
      };
      if (name === supported) {
        const intercept = value.intercept;
        value.intercept = function (interceptor) {
          if (this.taps.some((tap) => getWorkerFunctionDescriptor(tap.fn))) {
            throw new Error(
              'workerFunction beforeResolve does not support JavaScript hook interceptors',
            );
          }
          return intercept.call(this, interceptor);
        };
      }
    }
  }
}
