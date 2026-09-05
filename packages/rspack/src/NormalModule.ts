import binding from '@rspack/binding';
import * as liteTapable from '@rspack/lite-tapable';
import type { Source } from 'webpack-sources';
import { type Compilation, getOrCreateCompilationHooks } from './Compilation';
import type { LoaderContext } from './config';
import type { Module } from './Module';
import { SourceAdapter } from './util/source';

Object.defineProperty(binding.NormalModule.prototype, 'identifier', {
  enumerable: true,
  configurable: true,
  value(this: binding.NormalModule): string {
    // @ts-expect-error
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

/**
 * One entry of a module's loader list, in the shape webpack passes to
 * `beforeLoaders` taps.
 */
export interface LoaderItem {
  /** Absolute path of the loader, without the options query. */
  loader: string;
  /**
   * Loader options. An object when the loader was configured with one, the raw
   * query string when it was configured with a query, `undefined` otherwise.
   */
  options?: string | (object & { ident?: string }) | null;
  /** Key the options object is registered under, `null` for inline options. */
  ident: string | null;
  /** Module type of the loader itself, derived from its file extension. */
  type: string | null;
}

export interface NormalModuleCompilationHooks {
  beforeLoaders: liteTapable.SyncHook<[LoaderItem[], binding.NormalModule]>;
  loader: liteTapable.SyncHook<[LoaderContext, Module]>;
  readResource: liteTapable.HookMap<
    liteTapable.AsyncSeriesBailHook<[LoaderContext], string | Buffer>
  >;
}

Object.defineProperty(binding.NormalModule, 'getCompilationHooks', {
  enumerable: true,
  configurable: true,
  value(compilation: Compilation): NormalModuleCompilationHooks {
    if (!(binding.COMPILATION_HOOKS_MAP_SYMBOL in compilation)) {
      throw new TypeError(
        "The 'compilation' argument must be an instance of Compilation",
      );
    }

    return getOrCreateCompilationHooks(compilation, compilation, () => ({
      beforeLoaders: new liteTapable.SyncHook(['loaders', 'module']),
      loader: new liteTapable.SyncHook(['loaderContext', 'module']),
      readResource: new liteTapable.HookMap(
        () => new liteTapable.AsyncSeriesBailHook(['loaderContext']),
      ),
    }));
  },
});

declare module '@rspack/binding' {
  interface NormalModuleConstructor {
    getCompilationHooks(compilation: Compilation): NormalModuleCompilationHooks;
  }
}

export { NormalModule } from '@rspack/binding';
