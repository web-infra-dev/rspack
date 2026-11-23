import { BuiltinPluginName } from '@rspack/binding';

import { create } from '../builtin-plugin/base';

export interface ModuleFederationRuntimeExperimentsOptions {
  asyncStartup?: boolean;
}

export interface ModuleFederationRuntimeOptions {
  entryRuntime?: string;
  removeUnusedRuntime?: boolean | RegExp;
  experiments?: ModuleFederationRuntimeExperimentsOptions;
}

export const ModuleFederationRuntimePlugin = create(
  BuiltinPluginName.ModuleFederationRuntimePlugin,
  (options: ModuleFederationRuntimeOptions = {}) => options,
);
