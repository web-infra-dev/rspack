import { type BuiltinPlugin, BuiltinPluginName } from '@rspack/binding';
import { createBuiltinPlugin, RspackBuiltinPlugin } from './base';

export interface CompactHashedModuleIdsPluginOptions {
  minLength?: number;
}

export class CompactHashedModuleIdsPlugin extends RspackBuiltinPlugin {
  name = BuiltinPluginName.CompactHashedModuleIdsPlugin;
  affectedHooks = 'compilation' as const;

  constructor(private options: CompactHashedModuleIdsPluginOptions = {}) {
    super();
  }

  raw(): BuiltinPlugin {
    return createBuiltinPlugin(this.name, { ...this.options });
  }
}

/** @deprecated Use `CompactHashedModuleIdsPluginOptions` instead. */
export type CompatHashedModuleIdsPluginOptions =
  CompactHashedModuleIdsPluginOptions;

/** @deprecated Use `CompactHashedModuleIdsPlugin` instead. */
export const CompatHashedModuleIdsPlugin = CompactHashedModuleIdsPlugin;

/** @deprecated Use `CompactHashedModuleIdsPlugin` instead. */
export type CompatHashedModuleIdsPlugin = CompactHashedModuleIdsPlugin;
