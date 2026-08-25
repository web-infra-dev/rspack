import { type BuiltinPlugin, BuiltinPluginName } from '@rspack/binding';
import { createBuiltinPlugin, RspackBuiltinPlugin } from './base';

export interface CompatHashedChunkIdsPluginOptions {
  minLength?: number;
}

export class CompatHashedChunkIdsPlugin extends RspackBuiltinPlugin {
  name = BuiltinPluginName.CompatHashedChunkIdsPlugin;
  affectedHooks = 'compilation' as const;

  constructor(private options: CompatHashedChunkIdsPluginOptions = {}) {
    super();
  }

  raw(): BuiltinPlugin {
    return createBuiltinPlugin(this.name, { ...this.options });
  }
}
