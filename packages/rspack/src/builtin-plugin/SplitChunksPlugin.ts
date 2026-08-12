import {
  type BuiltinPlugin,
  BuiltinPluginName,
  type JsCacheGroupTestCtx,
  type RawCacheGroupOptions,
  type RawSplitChunksOptions,
} from '@rspack/binding';

import type { Chunk } from '../Chunk';
import type { Compiler } from '../Compiler';
import type {
  OptimizationSplitChunksCacheGroup,
  OptimizationSplitChunksOptions,
} from '../config';
import type { Module } from '../Module';
import { JsSplitChunkSizes } from '../util/SplitChunkSize';
import { createBuiltinPlugin, RspackBuiltinPlugin } from './base';

export class SplitChunksPlugin extends RspackBuiltinPlugin {
  name = BuiltinPluginName.SplitChunksPlugin;
  affectedHooks = 'thisCompilation' as const;

  constructor(private options: OptimizationSplitChunksOptions) {
    super();
  }

  raw(compiler: Compiler): BuiltinPlugin {
    const rawOptions = toRawSplitChunksOptions(this.options, compiler);
    if (rawOptions === undefined) {
      throw new Error('rawOptions should not be undefined');
    }
    return createBuiltinPlugin(this.name, rawOptions);
  }
}

export function toRawSplitChunksOptions(
  sc: false | OptimizationSplitChunksOptions,
  compiler: Compiler,
): RawSplitChunksOptions | undefined {
  if (!sc) {
    return;
  }

  function getName(name: any) {
    interface Batch {
      modules: Module[];
      chunks: Chunk[];
      moduleIndices: Uint32Array;
      chunkOffsets: Uint32Array;
      chunkIndices: Uint32Array;
      cacheGroupKey: string;
    }

    if (typeof name === 'function') {
      return (batch: Batch) => {
        const {
          modules,
          chunks,
          moduleIndices,
          chunkOffsets,
          chunkIndices,
          cacheGroupKey,
        } = batch;
        const results = new Array(moduleIndices.length);

        for (let i = 0; i < moduleIndices.length; i++) {
          const module = modules[moduleIndices[i]];
          const start = chunkOffsets[i];
          const end = chunkOffsets[i + 1];
          const contextChunks = new Array<Chunk>(end - start);
          for (let j = start; j < end; j++) {
            contextChunks[j - start] = chunks[chunkIndices[j]];
          }

          if (typeof module === 'undefined') {
            results[i] = name(undefined);
          } else {
            results[i] = name(module, contextChunks, cacheGroupKey);
          }
        }

        return results;
      };
    }
    return name;
  }

  function getTest(test: OptimizationSplitChunksCacheGroup['test']) {
    if (typeof test === 'function') {
      return (contexts: JsCacheGroupTestCtx[]) => {
        // chunk graph and module graph should all exist in the optimizeChunks stage
        const info = {
          moduleGraph: compiler._lastCompilation!.moduleGraph,
          chunkGraph: compiler._lastCompilation!.chunkGraph,
        };
        return contexts.map((ctx) => test(ctx.module, info));
      };
    }
    return test;
  }

  function getChunks(chunks: any) {
    if (typeof chunks === 'function') {
      return (items: Chunk[]) => items.map((chunk) => chunks(chunk));
    }
    return chunks;
  }

  function getLayer(layer: OptimizationSplitChunksCacheGroup['layer']) {
    if (typeof layer === 'function') {
      return (layers: (string | undefined)[]) => layers.map(layer);
    }
    return layer;
  }

  const {
    name,
    chunks,
    defaultSizeTypes,
    cacheGroups = {},
    fallbackCacheGroup,
    minSize,
    minSizeReduction,
    enforceSizeThreshold,
    maxSize,
    maxAsyncSize,
    maxInitialSize,
    ...passThrough
  } = sc;

  return {
    name: getName(name),
    chunks: getChunks(chunks),
    defaultSizeTypes: defaultSizeTypes || ['javascript', 'unknown'],
    cacheGroups: Object.entries(cacheGroups)
      .filter(([_key, group]) => group !== false)
      .map(([key, group]) => {
        const {
          test,
          name,
          chunks,
          layer,
          minSize,
          minSizeReduction,
          enforceSizeThreshold,
          maxSize,
          maxAsyncSize,
          maxInitialSize,
          ...passThrough
        } = group as Exclude<typeof group, false>;
        const rawGroup: RawCacheGroupOptions = {
          key,
          test: getTest(test),
          name: getName(name),
          chunks: getChunks(chunks),
          layer: getLayer(layer),
          minSize: JsSplitChunkSizes.__to_binding(minSize),
          minSizeReduction: JsSplitChunkSizes.__to_binding(minSizeReduction),
          enforceSizeThreshold:
            JsSplitChunkSizes.__to_binding(enforceSizeThreshold),
          maxSize: JsSplitChunkSizes.__to_binding(maxSize),
          maxAsyncSize: JsSplitChunkSizes.__to_binding(maxAsyncSize),
          maxInitialSize: JsSplitChunkSizes.__to_binding(maxInitialSize),
          ...passThrough,
        };
        return rawGroup;
      }),
    fallbackCacheGroup: {
      chunks: getChunks(chunks),
      ...fallbackCacheGroup,
    },
    minSize: JsSplitChunkSizes.__to_binding(minSize),
    minSizeReduction: JsSplitChunkSizes.__to_binding(minSizeReduction),
    enforceSizeThreshold: JsSplitChunkSizes.__to_binding(enforceSizeThreshold),
    maxSize: JsSplitChunkSizes.__to_binding(maxSize),
    maxAsyncSize: JsSplitChunkSizes.__to_binding(maxAsyncSize),
    maxInitialSize: JsSplitChunkSizes.__to_binding(maxInitialSize),
    ...passThrough,
  };
}
