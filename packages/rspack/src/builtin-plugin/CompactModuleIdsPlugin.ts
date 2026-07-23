import { type BuiltinPlugin, BuiltinPluginName } from '@rspack/binding';
import { createBuiltinPlugin, RspackBuiltinPlugin } from './base';
import type { Module } from '../Module';

export interface CompactModuleIdsPluginOptions {
  context?: string;
  test?: (module: Module) => boolean;
  maxLength?: number;
  salt?: number;
  fixedLength?: boolean;
  failOnConflict?: boolean;
}

export class CompactModuleIdsPlugin extends RspackBuiltinPlugin {
  name = BuiltinPluginName.CompactModuleIdsPlugin;
  affectedHooks = 'compilation' as const;

  constructor(private options: CompactModuleIdsPluginOptions = {}) {
    super();
  }

  raw(): BuiltinPlugin {
    return createBuiltinPlugin(this.name, { ...this.options });
  }
}
