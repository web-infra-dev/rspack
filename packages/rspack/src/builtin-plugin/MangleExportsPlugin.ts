import { type BuiltinPlugin, BuiltinPluginName } from '@rspack/binding';
import { createBuiltinPlugin, RspackBuiltinPlugin } from './base';

export class MangleExportsPlugin extends RspackBuiltinPlugin {
  name = BuiltinPluginName.MangleExportsPlugin;
  affectedHooks = 'compilation' as const;

  constructor(private deterministic: boolean) {
    super();
  }

  raw(): BuiltinPlugin {
    return createBuiltinPlugin(this.name, this.deterministic);
  }
}
