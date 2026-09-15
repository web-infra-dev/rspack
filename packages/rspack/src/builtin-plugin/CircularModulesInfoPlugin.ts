import { type BuiltinPlugin, BuiltinPluginName } from '@rspack/binding';
import type { Compiler } from '../Compiler';
import { createBuiltinPlugin, RspackBuiltinPlugin } from './base';

export class CircularModulesInfoPlugin extends RspackBuiltinPlugin {
  name = BuiltinPluginName.CircularModulesInfoPlugin;

  raw(_compiler: Compiler): BuiltinPlugin {
    return createBuiltinPlugin(this.name, undefined);
  }
}
