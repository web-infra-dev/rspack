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
    const rawOptions = toRawSplitChunksOptions(this.options, compiler, true);
    if (rawOptions === undefined) {
      throw new Error('rawOptions should not be undefined');
    }
    return createBuiltinPlugin(this.name, rawOptions);
  }
}

export function toRawSplitChunksOptions(
  sc: false | OptimizationSplitChunksOptions,
  compiler: Compiler,
  enableNameBatch = false,
): RawSplitChunksOptions | undefined {
  if (!sc) {
    return;
  }

  function getName(name: any) {
    interface Context {
      module: Module;
      chunks: Chunk[];
      cacheGroupKey: string;
    }

    if (typeof name === 'function') {
      if (!enableNameBatch) {
        return {
          name: (ctx: Context) => {
            if (typeof ctx.module === 'undefined') {
              return name(undefined);
            }
            return name(ctx.module, getChunks(ctx.chunks), ctx.cacheGroupKey);
          },
        };
      }

      return {
        nameBatch: getNameBatch(name),
      };
    }
    return { name };
  }

  function getNameBatch(name: any) {
    interface Batch {
      modules: Module[];
      chunks: Chunk[];
      chunkData: Uint32Array;
      cacheGroupKey: string;
    }

    return (batch: Batch) => {
      const { modules, chunks, chunkData, cacheGroupKey } = batch;
      const results = new Array(modules.length);
      // The offset table is followed by indices into the deduplicated chunk array.
      const chunkIndexStart = modules.length + 1;

      for (let i = 0; i < modules.length; i++) {
        const module = modules[i];
        const start = chunkData[i];
        const end = chunkData[i + 1];
        const contextChunks = new Array<Chunk>(end - start);
        for (let j = start; j < end; j++) {
          contextChunks[j - start] = chunks[chunkData[chunkIndexStart + j]];
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

  function getTest(test: OptimizationSplitChunksCacheGroup['test']) {
    if (typeof test === 'function') {
      return (ctx: JsCacheGroupTestCtx) => {
        // chunk graph and module graph should all exist in the optimizeChunks stage
        const info = {
          moduleGraph: compiler._lastCompilation!.moduleGraph,
          chunkGraph: compiler._lastCompilation!.chunkGraph,
        };
        return test(ctx.module, info);
      };
    }
    return test;
  }

  function getChunks(chunks: any) {
    if (typeof chunks === 'function') {
      return (chunk: Chunk) => chunks(chunk);
    }
    return chunks;
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
    ...getName(name),
    chunks: getChunks(chunks),
    defaultSizeTypes: defaultSizeTypes || ['javascript', 'unknown'],
    cacheGroups: Object.entries(cacheGroups)
      .filter(([_key, group]) => group !== false)
      .map(([key, group]) => {
        const {
          test,
          name,
          chunks,
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
          ...getName(name),
          chunks: getChunks(chunks),
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
