import { type BuiltinPlugin, BuiltinPluginName } from '@rspack/binding';
import { createBuiltinPlugin, RspackBuiltinPlugin } from './base';

export interface DeterministicChunkIdsPluginOptions {
  salt?: number;
}

export class DeterministicChunkIdsPlugin extends RspackBuiltinPlugin {
  name = BuiltinPluginName.DeterministicChunkIdsPlugin;
  affectedHooks = 'compilation' as const;

  constructor(private options: DeterministicChunkIdsPluginOptions = {}) {
    super();
  }

  raw(): BuiltinPlugin {
    return createBuiltinPlugin(this.name, { ...this.options });
  }
}
