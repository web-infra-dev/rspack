import { BuiltinPluginName } from '@rspack/binding';

import { create } from '../builtin-plugin/base';

export interface ModuleFederationRuntimeOptions {
  entryRuntime?: string;
}

export const ModuleFederationRuntimePlugin = create(
  BuiltinPluginName.ModuleFederationRuntimePlugin,
  (options: ModuleFederationRuntimeOptions = {}) => options,
);
