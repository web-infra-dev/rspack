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
  readResource: liteTapable.HookMap<
    liteTapable.AsyncSeriesBailHook<[LoaderContext], string | Buffer>
  >;
}

Object.defineProperty(binding.NormalModule, 'getCompilationHooks', {
  enumerable: true,
  configurable: true,
  value(compilation: Compilation): NormalModuleCompilationHooks {
    return getOrCreateCompilationHooks(compilation, compilation, () => ({
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
