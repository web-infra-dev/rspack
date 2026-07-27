import { type BuiltinPlugin, BuiltinPluginName } from '@rspack/binding';
import { createBuiltinPlugin, RspackBuiltinPlugin } from './base';

export interface CompactChunkIdsPluginOptions {
  minLength?: number;
}

export class CompactChunkIdsPlugin extends RspackBuiltinPlugin {
  name = BuiltinPluginName.CompactChunkIdsPlugin;
  affectedHooks = 'compilation' as const;

  constructor(private options: CompactChunkIdsPluginOptions = {}) {
    super();
  }

  raw(): BuiltinPlugin {
    return createBuiltinPlugin(this.name, { ...this.options });
  }
}
