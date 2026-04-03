import { Buffer } from 'node:buffer';
import binding from '@rspack/binding';
import type * as liteTapable from '@rspack/lite-tapable';
import type { Compiler } from './Compiler';

const HOOK_USAGE_TRACKED = Symbol('rspack.hookUsageTracked');
const HOOK_MAP_USAGE_TRACKED = Symbol('rspack.hookMapUsageTracked');
const INTERNAL_HOOK_INTERCEPTOR = Symbol('rspack.internalHookInterceptor');
export const COMPILER_HOOK_USAGE_TRACKERS = new WeakMap<
  Compiler,
  JsHookUsageTracker
>();

const COMPILER_SCOPED_HOOK_KINDS = new Set<binding.RegisterJsTapKind>([
  binding.RegisterJsTapKind.CompilerThisCompilation,
  binding.RegisterJsTapKind.CompilerCompilation,
  binding.RegisterJsTapKind.CompilerMake,
  binding.RegisterJsTapKind.CompilerFinishMake,
  binding.RegisterJsTapKind.CompilerShouldEmit,
  binding.RegisterJsTapKind.CompilerEmit,
  binding.RegisterJsTapKind.CompilerAfterEmit,
  binding.RegisterJsTapKind.CompilerAssetEmitted,
]);

const HOOK_USAGE_BUFFER_BYTE_LENGTH = 8;
const COMPILER_SCOPED_BYTE_MASKS = Buffer.alloc(HOOK_USAGE_BUFFER_BYTE_LENGTH);

for (const kind of COMPILER_SCOPED_HOOK_KINDS) {
  COMPILER_SCOPED_BYTE_MASKS[kind >> 3] |= 1 << (kind & 7);
}

type InterceptableHook = liteTapable.Hook<any, any, any> & {
  [HOOK_USAGE_TRACKED]?: boolean;
  _tap?: (type: string, options: liteTapable.Options, fn: Function) => void;
  intercept?: (interceptor: object) => void;
};

type InterceptableHookMap<H extends liteTapable.Hook<any, any, any>> =
  liteTapable.HookMap<H> & {
    [HOOK_MAP_USAGE_TRACKED]?: boolean;
    _map?: Map<unknown, H>;
  };

export class JsHookUsageTracker {
  #buffer: Buffer;

  constructor() {
    const storage =
      typeof SharedArrayBuffer === 'function'
        ? new SharedArrayBuffer(HOOK_USAGE_BUFFER_BYTE_LENGTH)
        : new ArrayBuffer(HOOK_USAGE_BUFFER_BYTE_LENGTH);
    this.#buffer = Buffer.from(storage);
  }

  getBuffer() {
    return this.#buffer;
  }

  mark(kind: binding.RegisterJsTapKind) {
    this.#buffer[kind >> 3] |= 1 << (kind & 7);
  }

  resetCompilationScopedBits() {
    for (let i = 0; i < this.#buffer.length; i++) {
      this.#buffer[i] &= COMPILER_SCOPED_BYTE_MASKS[i];
    }
  }

  resetAllBits() {
    for (let i = 0; i < this.#buffer.length; i++) {
      this.#buffer[i] = 0;
    }
  }
}

const isInternalHookInterceptor = (interceptor: unknown) =>
  typeof interceptor === 'object' &&
  interceptor !== null &&
  INTERNAL_HOOK_INTERCEPTOR in interceptor;

export const markHookInterceptorAsInternal = <T extends object>(
  interceptor: T,
): T => {
  Reflect.set(interceptor, INTERNAL_HOOK_INTERCEPTOR, true);
  return interceptor;
};

export const trackHookUsage = <T extends liteTapable.Hook<any, any, any>>(
  hook: T,
  tracker: JsHookUsageTracker,
  kind: binding.RegisterJsTapKind,
): T => {
  const trackableHook = hook as InterceptableHook;
  if (trackableHook[HOOK_USAGE_TRACKED]) {
    return hook;
  }
  trackableHook[HOOK_USAGE_TRACKED] = true;

  if (trackableHook._tap) {
    const tap = trackableHook._tap.bind(hook);
    trackableHook._tap = ((type, options, fn) => {
      tracker.mark(kind);
      return tap(type, options, fn);
    }) as typeof trackableHook._tap;
  }

  if (trackableHook.intercept) {
    const intercept = trackableHook.intercept.bind(hook);
    trackableHook.intercept = ((interceptor: object) => {
      if (!isInternalHookInterceptor(interceptor)) {
        tracker.mark(kind);
      }
      return intercept(interceptor);
    }) as typeof trackableHook.intercept;
  }

  return hook;
};

export const trackHookMapUsage = <H extends liteTapable.Hook<any, any, any>>(
  hookMap: liteTapable.HookMap<H>,
  tracker: JsHookUsageTracker,
  kind: binding.RegisterJsTapKind,
): liteTapable.HookMap<H> => {
  const trackableHookMap = hookMap as InterceptableHookMap<H>;
  if (trackableHookMap[HOOK_MAP_USAGE_TRACKED]) {
    return hookMap;
  }
  trackableHookMap[HOOK_MAP_USAGE_TRACKED] = true;

  hookMap.intercept({
    factory: (_key, hook) => {
      return trackHookUsage<H>(hook!, tracker, kind);
    },
  });
  for (const hook of trackableHookMap._map?.values() ?? []) {
    trackHookUsage(hook, tracker, kind);
  }

  return hookMap;
};
