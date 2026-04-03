import { Buffer } from 'node:buffer';
import binding from '@rspack/binding';
import * as liteTapable from '@rspack/lite-tapable';
import type { Compiler } from './Compiler';

const HOOK_USAGE_TRACKED = Symbol('rspack.hookUsageTracked');
const HOOK_MAP_USAGE_TRACKED = Symbol('rspack.hookMapUsageTracked');
export const COMPILER_HOOK_USAGE_TRACKERS = new WeakMap<
  Compiler,
  JsHookUsageTracker
>();

const { compiler: COMPILER_HOOKS, compilation: COMPILATION_HOOKS } =
  binding.getRegisterJsTapScopeKinds();

const HOOK_USAGE_BUFFER_BYTE_LENGTH =
  (Math.max(...COMPILER_HOOKS, ...COMPILATION_HOOKS) >> 3) + 1;

export class JsHookUsageTracker {
  #buffer: Buffer;

  constructor() {
    this.#buffer = Buffer.alloc(HOOK_USAGE_BUFFER_BYTE_LENGTH);
  }

  getBuffer() {
    return this.#buffer;
  }

  mark(kind: binding.RegisterJsTapKind) {
    const byteIndex = kind >> 3;
    const bitMask = 1 << (kind & 7);
    if (this.#buffer[byteIndex] & bitMask) {
      return;
    }
    this.#buffer[byteIndex] |= bitMask;
  }

  resetCompilationScopedBits() {
    for (const kind of COMPILATION_HOOKS) {
      this.#buffer[kind >> 3] &= ~(1 << (kind & 7));
    }
  }

  resetAllBits() {
    this.#buffer.fill(0);
  }
}

type InterceptableHook = liteTapable.Hook<any, any, any> & {
  [HOOK_USAGE_TRACKED]?: boolean;
};

type InterceptableHookMap<H extends liteTapable.Hook<any, any, any>> =
  liteTapable.HookMap<H> & {
    [HOOK_MAP_USAGE_TRACKED]?: boolean;
  };

export const trackHookUsage = <T extends liteTapable.Hook<any, any, any>>(
  hook: T,
  tracker: JsHookUsageTracker,
  kind: binding.RegisterJsTapKind,
) => {
  const trackableHook = hook as InterceptableHook;
  if (trackableHook[HOOK_USAGE_TRACKED]) {
    return hook;
  }
  trackableHook[HOOK_USAGE_TRACKED] = true;

  let marked = false;

  hook.intercept({
    register: (tapInfo) => {
      if (!marked) {
        marked = true;
        tracker.mark(kind);
      }
      return tapInfo;
    },
  });

  return hook;
};

export const trackHookMapUsage = <H extends liteTapable.Hook<any, any, any>>(
  hookMap: liteTapable.HookMap<H>,
  tracker: JsHookUsageTracker,
  kind: binding.RegisterJsTapKind,
) => {
  const trackableHookMap = hookMap as InterceptableHookMap<H>;
  if (trackableHookMap[HOOK_MAP_USAGE_TRACKED]) {
    return hookMap;
  }
  trackableHookMap[HOOK_MAP_USAGE_TRACKED] = true;

  hookMap.intercept({
    factory: (_key, hook) => {
      if (hook === undefined) {
        throw new Error('HookMap factory interceptor must receive a hook');
      }
      return trackHookUsage<H>(hook, tracker, kind);
    },
  });

  return hookMap;
};
