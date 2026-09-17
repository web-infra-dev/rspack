import type { Compilation } from '../Compilation';
import type { Compiler } from '../Compiler';
import type { LoaderContext } from '../config';
import { NormalModule } from '../NormalModule';
import { getWorkerFunctionDescriptor } from '../workerFunction';
import { prepareWorkerFunctionValue } from './service';

export function collectLoaderHookChanges(
  context: LoaderContext,
  before: Map<PropertyKey, PropertyDescriptor | undefined>,
): { properties: Record<string, any>; symbols: [string, any][] } {
  const properties: Record<string, any> = {};
  const symbols: [string, any][] = [];
  for (const key of Reflect.ownKeys(context)) {
    const previous = before.get(key);
    const current = Object.getOwnPropertyDescriptor(context, key);
    if (
      current &&
      'value' in current &&
      (!previous ||
        !('value' in previous) ||
        !Object.is(previous.value, current.value))
    ) {
      if (typeof key === 'string') {
        properties[key] = current.value;
      } else {
        const name = Symbol.keyFor(key);
        if (name !== undefined) symbols.push([name, current.value]);
      }
    }
  }
  return { properties, symbols };
}

/** Run sync hooks together, preserving tap order and interceptor semantics. */
export async function runLoaderHooks(
  compiler: Compiler,
  loaderContext: LoaderContext,
  hooksOnly: boolean,
): Promise<Function[]> {
  const hooks = [];
  let compilation: Compilation | undefined = compiler._lastCompilation;
  while (compilation) {
    hooks.push(NormalModule.getCompilationHooks(compilation).loader);
    compilation = compilation.compiler.parentCompilation;
    if (hooks.length > 1000) {
      throw new Error(
        'Too many nested child compiler, exceeded max limitation 1000',
      );
    }
  }

  // Only move the complete hook chain: an ordinary tap or interceptor may read
  // fields set by an earlier worker tap, including taps from child compilations.
  if (
    hooksOnly &&
    hooks.every(
      (hook) =>
        hook.interceptors.length === 0 &&
        hook.taps.every((tap) => getWorkerFunctionDescriptor(tap.fn)),
    )
  ) {
    return hooks.flatMap((hook) => hook.taps.map((tap) => tap.fn));
  }

  for (const hook of hooks) {
    if (!hook.taps.some((tap) => getWorkerFunctionDescriptor(tap.fn))) {
      hook.call(loaderContext, loaderContext._module);
      continue;
    }
    const queried = hook.queryStageRange([-Infinity, Infinity]);
    // Prepare a per-invocation view; concurrent modules must not mutate shared taps.
    queried.tapsInRange = await Promise.all(
      queried.tapsInRange.map(async (tap) =>
        getWorkerFunctionDescriptor(tap.fn)
          ? {
              ...tap,
              fn: await prepareWorkerFunctionValue(tap.fn, compiler),
            }
          : tap,
      ),
    );
    queried.call(loaderContext, loaderContext._module);
  }
  return [];
}
