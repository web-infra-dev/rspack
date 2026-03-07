import { BuiltinPluginName } from '@rspack/binding';

import { create } from '../builtin-plugin/base';

export interface ModuleFederationRuntimeExperimentsOptions {
  asyncStartup?: boolean;
  rsc?: boolean;
}

export interface ModuleFederationRuntimeOptions {
  entryRuntime?: string;
  experiments?: ModuleFederationRuntimeExperimentsOptions;
}

export const ModuleFederationRuntimePlugin = create(
  BuiltinPluginName.ModuleFederationRuntimePlugin,
  (options: ModuleFederationRuntimeOptions = {}) => options,
);
