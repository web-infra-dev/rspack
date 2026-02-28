/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/blob/3919c84/lib/javascript/EnableChunkLoadingPlugin.js
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */

import { BuiltinPluginName } from '@rspack/binding';

import type { ChunkLoadingType, Compiler } from '../exports';
import { create } from './base';

const EnableChunkLoadingPluginInner = create(
  BuiltinPluginName.EnableChunkLoadingPlugin,
  (type: string): string => type,
);

const enabledTypes: WeakMap<Compiler, Set<ChunkLoadingType>> = new WeakMap();

const getEnabledTypes = (compiler: Compiler): Set<ChunkLoadingType> => {
  let set = enabledTypes.get(compiler);
  if (set === undefined) {
    set = new Set();
    enabledTypes.set(compiler, set);
  }
  return set;
};

export class EnableChunkLoadingPlugin extends EnableChunkLoadingPluginInner {
  static setEnabled(compiler: Compiler, type: ChunkLoadingType) {
    getEnabledTypes(compiler).add(type);
  }

  static checkEnabled(compiler: Compiler, type: ChunkLoadingType) {
    if (!getEnabledTypes(compiler).has(type)) {
      throw new Error(
        [
          `Chunk loading type "${type}" is not enabled.`,
          'EnableChunkLoadingPlugin need to be used to enable this type of chunk loading.',
          'This usually happens through the "output.enabledChunkLoadingTypes" option.',
          'If you are using a function as entry which sets "chunkLoading", you need to add all potential chunk loading types to "output.enabledChunkLoadingTypes".',
          `These types are enabled: ${Array.from(
            getEnabledTypes(compiler),
          ).join(', ')}`,
        ].join(' '),
      );
    }
  }

  override apply(compiler: Compiler): void {
    const [type] = this._args;
    // Only enable once
    const enabled = getEnabledTypes(compiler);
    if (enabled.has(type)) return;
    enabled.add(type);

    switch (type) {
      // builtin chunk loading types
      case 'jsonp':
      case 'import-scripts':
      case 'require':
      case 'async-node':
      case 'import': {
        super.apply(compiler);
        return;
      }
      default:
        throw new Error(`Unsupported chunk loading type ${type}.
Plugins which provide custom chunk loading types must call EnableChunkLoadingPlugin.setEnabled(compiler, type) to disable this error.`);
    }
  }
}
