import binding from '@rspack/binding';
import * as liteTapable from '@rspack/lite-tapable';
import type { Source } from 'webpack-sources';
import type { Chunk } from './Chunk';
import { type Compilation, checkCompilation } from './Compilation';
import { SourceAdapter } from './util/source';

Object.defineProperty(binding.ExternalModule.prototype, 'identifier', {
  enumerable: true,
  configurable: true,
  value(this: binding.ExternalModule): string {
    return this[binding.MODULE_IDENTIFIER_SYMBOL];
  },
});
Object.defineProperty(binding.ExternalModule.prototype, 'originalSource', {
  enumerable: true,
  configurable: true,
  value(this: binding.ExternalModule) {
    const originalSource = this._originalSource();
    if (originalSource) {
      return SourceAdapter.fromBinding(originalSource);
    }
    return null;
  },
});
Object.defineProperty(binding.ExternalModule.prototype, 'emitFile', {
  enumerable: true,
  configurable: true,
  value(
    this: binding.ExternalModule,
    filename: string,
    source: Source,
    assetInfo?: binding.AssetInfo,
  ) {
    return this._emitFile(filename, SourceAdapter.toBinding(source), assetInfo);
  },
});

export type ExternalModule = binding.ExternalModule;
export type ExternalModuleCompilationHooks = {
  chunkCondition: liteTapable.SyncBailHook<
    [Chunk, Compilation],
    boolean | undefined
  >;
};

const ExternalModule =
  binding.ExternalModule as typeof binding.ExternalModule & {
    getCompilationHooks(
      compilation: Compilation,
    ): ExternalModuleCompilationHooks;
  };

ExternalModule.getCompilationHooks = (compilation: Compilation) => {
  checkCompilation(compilation);

  const compilationHooksMap = compilation[
    binding.COMPILATION_HOOKS_MAP_SYMBOL
  ] as unknown as WeakMap<
    typeof ExternalModule,
    ExternalModuleCompilationHooks
  >;
  let hooks = compilationHooksMap.get(ExternalModule);
  if (hooks === undefined) {
    hooks = {
      chunkCondition: new liteTapable.SyncBailHook(['chunk', 'compilation']),
    };
    compilationHooksMap.set(ExternalModule, hooks);
  }
  return hooks;
};

export { ExternalModule };
