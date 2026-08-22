import { type BuiltinPlugin, BuiltinPluginName } from '@rspack/binding';
import { createBuiltinPlugin, RspackBuiltinPlugin } from './base';

export interface CompatHashedModuleIdsPluginOptions {
  minLength?: number;
}

export class CompatHashedModuleIdsPlugin extends RspackBuiltinPlugin {
  name = BuiltinPluginName.CompatHashedModuleIdsPlugin;
  affectedHooks = 'compilation' as const;

  constructor(private options: CompatHashedModuleIdsPluginOptions = {}) {
    super();
  }

  raw(): BuiltinPlugin {
    return createBuiltinPlugin(this.name, { ...this.options });
  }
}
