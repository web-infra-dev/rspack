import { type BuiltinPlugin, BuiltinPluginName } from '@rspack/binding';
import { createBuiltinPlugin, RspackBuiltinPlugin } from './base';

export class DeterministicChunkIdsPlugin extends RspackBuiltinPlugin {
  name = BuiltinPluginName.DeterministicChunkIdsPlugin;
  affectedHooks = 'compilation' as const;

  raw(): BuiltinPlugin {
    return createBuiltinPlugin(this.name, undefined);
  }
}
