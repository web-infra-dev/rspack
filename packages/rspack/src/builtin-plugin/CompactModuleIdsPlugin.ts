import { type BuiltinPlugin, BuiltinPluginName } from '@rspack/binding';
import { createBuiltinPlugin, RspackBuiltinPlugin } from './base';

export interface CompactModuleIdsPluginOptions {
  minLength?: number;
}

export class CompactModuleIdsPlugin extends RspackBuiltinPlugin {
  name = BuiltinPluginName.CompactModuleIdsPlugin;
  affectedHooks = 'compilation' as const;

  constructor(private options: CompactModuleIdsPluginOptions = {}) {
    super();
  }

  raw(): BuiltinPlugin {
    return createBuiltinPlugin(this.name, { ...this.options });
  }
}
