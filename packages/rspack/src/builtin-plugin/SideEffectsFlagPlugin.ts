import { type BuiltinPlugin, BuiltinPluginName } from '@rspack/binding';

import { createBuiltinPlugin, RspackBuiltinPlugin } from './base';

export class SideEffectsFlagPlugin extends RspackBuiltinPlugin {
  name = BuiltinPluginName.SideEffectsFlagPlugin;
  affectedHooks = 'compilation' as const;

  constructor(private analyzeSideEffectsFree = true) {
    super();
  }

  raw(): BuiltinPlugin {
    return createBuiltinPlugin(this.name, this.analyzeSideEffectsFree);
  }
}
