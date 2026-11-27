import type { Fn, SyncHook } from '@rspack/lite-tapable';

import type { Compilation } from '../Compilation';

const defaultStatsTaps = new WeakSet<Function>();

export function tapDefaultStatsHook<T>(
  hook: SyncHook<T>,
  name: string,
  fn: Fn<T, void>,
): void {
  defaultStatsTaps.add(fn);
  hook.tap(name, fn);
}

function onlyDefaultTaps(hook: {
  taps: readonly { fn: Function }[];
  interceptors: readonly unknown[];
}): boolean {
  return (
    hook.interceptors.length === 0 &&
    hook.taps.every(({ fn }) => defaultStatsTaps.has(fn))
  );
}

export function canCreateDefaultStatsJson(compilation: Compilation): boolean {
  return (
    compilation.hooks.statsFactory.taps.length === 1 &&
    hasOnlyDefaultStatsHooks(compilation)
  );
}

function hasOnlyDefaultStatsHooks(compilation: Compilation): boolean {
  const {
    statsFactory,
    statsNormalize,
    statsPreset,
    statsPrinter,
    processWarnings,
  } = compilation.hooks;
  if (
    !onlyDefaultTaps(statsFactory) ||
    !onlyDefaultTaps(statsNormalize) ||
    !onlyDefaultTaps(statsPrinter) ||
    processWarnings.isUsed() ||
    statsPreset._interceptors.length !== 0
  ) {
    return false;
  }
  for (const hook of statsPreset._map.values()) {
    if (!onlyDefaultTaps(hook)) return false;
  }
  return compilation.children.every(hasOnlyDefaultStatsHooks);
}
