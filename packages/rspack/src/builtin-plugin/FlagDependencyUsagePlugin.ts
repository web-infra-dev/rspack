import { type BuiltinPlugin, BuiltinPluginName } from '@rspack/binding';
import { createBuiltinPlugin, RspackBuiltinPlugin } from './base';

export class FlagDependencyUsagePlugin extends RspackBuiltinPlugin {
  name = BuiltinPluginName.FlagDependencyUsagePlugin;
  affectedHooks = 'compilation' as const;

  constructor(private global: boolean) {
    super();
  }

  raw(): BuiltinPlugin {
    return createBuiltinPlugin(this.name, this.global);
  }
}
