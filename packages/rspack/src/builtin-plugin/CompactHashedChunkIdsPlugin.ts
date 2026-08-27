import { type BuiltinPlugin, BuiltinPluginName } from '@rspack/binding';
import { createBuiltinPlugin, RspackBuiltinPlugin } from './base';

export interface CompactHashedChunkIdsPluginOptions {
  minLength?: number;
}

export class CompactHashedChunkIdsPlugin extends RspackBuiltinPlugin {
  name = BuiltinPluginName.CompactHashedChunkIdsPlugin;
  affectedHooks = 'compilation' as const;

  constructor(private options: CompactHashedChunkIdsPluginOptions = {}) {
    super();
  }

  raw(): BuiltinPlugin {
    return createBuiltinPlugin(this.name, { ...this.options });
  }
}

/** @deprecated Use `CompactHashedChunkIdsPluginOptions` instead. */
export type CompatHashedChunkIdsPluginOptions =
  CompactHashedChunkIdsPluginOptions;

/** @deprecated Use `CompactHashedChunkIdsPlugin` instead. */
export const CompatHashedChunkIdsPlugin = CompactHashedChunkIdsPlugin;

/** @deprecated Use `CompactHashedChunkIdsPlugin` instead. */
export type CompatHashedChunkIdsPlugin = CompactHashedChunkIdsPlugin;
