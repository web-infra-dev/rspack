import util from 'node:util';
import binding from '@rspack/binding';
import * as liteTapable from '@rspack/lite-tapable';
import type { Source } from 'webpack-sources';
import type { Compilation } from './Compilation';
import type { LoaderContext } from './config';
import type { Module } from './Module';
import { SourceAdapter } from './util/source';

Object.defineProperty(binding.NormalModule.prototype, 'identifier', {
  enumerable: true,
  configurable: true,
  value(this: binding.NormalModule): string {
    return this[binding.MODULE_IDENTIFIER_SYMBOL];
  },
});
Object.defineProperty(binding.NormalModule.prototype, 'originalSource', {
  enumerable: true,
  configurable: true,
  value(this: binding.NormalModule) {
    const originalSource = this._originalSource();
    if (originalSource) {
      return SourceAdapter.fromBinding(originalSource);
    }
    return null;
  },
});
Object.defineProperty(binding.NormalModule.prototype, 'emitFile', {
  enumerable: true,
  configurable: true,
  value(
    this: binding.NormalModule,
    filename: string,
    source: Source,
    assetInfo?: binding.AssetInfo,
  ) {
    return this._emitFile(filename, SourceAdapter.toBinding(source), assetInfo);
  },
});

export interface NormalModuleCompilationHooks {
  loader: liteTapable.SyncHook<[LoaderContext, Module]>;
  readResourceForScheme: any;
  readResource: liteTapable.HookMap<
    liteTapable.AsyncSeriesBailHook<[LoaderContext], string | Buffer>
  >;
}

const createFakeHook = <T extends Record<string, any>>(
  fakeHook: T,
  message?: string,
  code?: string,
): FakeHook<T> => {
  return Object.freeze(
    Object.assign(
      message && code
        ? deprecateAllProperties(fakeHook, message, code)
        : fakeHook,
      { _fakeHook: true },
    ),
  );
};
type FakeHook<T> = {
  _fakeHook: true;
} & T;
const deprecateAllProperties = <O extends object>(
  obj: O,
  message: string,
  code: string,
) => {
  const newObj: any = {};
  const descriptors = Object.getOwnPropertyDescriptors(obj);
  for (const name of Object.keys(descriptors)) {
    const descriptor = descriptors[name];
    if (typeof descriptor.value === 'function') {
      Object.defineProperty(newObj, name, {
        ...descriptor,
        value: util.deprecate(descriptor.value, message, code),
      });
    } else if (descriptor.get || descriptor.set) {
      Object.defineProperty(newObj, name, {
        ...descriptor,
        get: descriptor.get && util.deprecate(descriptor.get, message, code),
        set: descriptor.set && util.deprecate(descriptor.set, message, code),
      });
    } else {
      let value = descriptor.value;
      Object.defineProperty(newObj, name, {
        configurable: descriptor.configurable,
        enumerable: descriptor.enumerable,
        get: util.deprecate(() => value, message, code),
        set: descriptor.writable
          ? util.deprecate((v: any) => (value = v), message, code)
          : undefined,
      });
    }
  }
  return newObj;
};

Object.defineProperty(binding.NormalModule, 'getCompilationHooks', {
  enumerable: true,
  configurable: true,
  value(compilation: Compilation): NormalModuleCompilationHooks {
    if (!(binding.COMPILATION_HOOKS_MAP_SYMBOL in compilation)) {
      throw new TypeError(
        "The 'compilation' argument must be an instance of Compilation",
      );
    }

    const compilationHooksMap =
      compilation[binding.COMPILATION_HOOKS_MAP_SYMBOL];
    let hooks = compilationHooksMap.get(compilation);
    if (hooks === undefined) {
      hooks = {
        loader: new liteTapable.SyncHook(['loaderContext', 'module']),
        // TODO webpack 6 deprecate
        readResourceForScheme: new liteTapable.HookMap((scheme) => {
          const hook = hooks!.readResource.for(scheme);
          return createFakeHook({
            tap: (options: string, fn: any) =>
              hook.tap(options, (loaderContext: LoaderContext) =>
                fn(loaderContext.resource),
              ),
            tapAsync: (options: string, fn: any) =>
              hook.tapAsync(
                options,
                (loaderContext: LoaderContext, callback: any) =>
                  fn(loaderContext.resource, callback),
              ),
            tapPromise: (options: string, fn: any) =>
              hook.tapPromise(options, (loaderContext: LoaderContext) =>
                fn(loaderContext.resource),
              ),
          }) as any;
        }),
        readResource: new liteTapable.HookMap(
          () => new liteTapable.AsyncSeriesBailHook(['loaderContext']),
        ),
      };
      compilationHooksMap.set(compilation, hooks);
    }
    return hooks;
  },
});

declare module '@rspack/binding' {
  interface NormalModuleConstructor {
    getCompilationHooks(compilation: Compilation): NormalModuleCompilationHooks;
  }
}

export { NormalModule } from '@rspack/binding';
