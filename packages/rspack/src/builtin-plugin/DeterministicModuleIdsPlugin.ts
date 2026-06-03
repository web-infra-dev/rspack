import { type BuiltinPlugin, BuiltinPluginName } from '@rspack/binding';
import { createBuiltinPlugin, RspackBuiltinPlugin } from './base';
import type { Module } from '../Module';

export interface DeterministicModuleIdsPluginOptions {
  context?: string;
  test?: (module: Module) => boolean;
  maxLength?: number;
  salt?: number;
  fixedLength?: boolean;
  failOnConflict?: boolean;
}

export class DeterministicModuleIdsPlugin extends RspackBuiltinPlugin {
  name = BuiltinPluginName.DeterministicModuleIdsPlugin;
  affectedHooks = 'compilation' as const;

  constructor(private options: DeterministicModuleIdsPluginOptions = {}) {
    super();
  }

  raw(): BuiltinPlugin {
    return createBuiltinPlugin(this.name, { ...this.options });
  }
}
