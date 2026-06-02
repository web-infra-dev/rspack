import {
  type BuiltinPlugin,
  BuiltinPluginName,
  type RawSyncModuleIdsPluginOptions,
} from '@rspack/binding';

import type { Module } from '../Module';
import { createBuiltinPlugin, RspackBuiltinPlugin } from './base';

export interface SyncModuleIdsPluginOptions {
  path: string;
  context?: string;
  test?: (module: Module) => boolean;
  mode?: 'read' | 'create' | 'merge' | 'update';
}

export class SyncModuleIdsPlugin extends RspackBuiltinPlugin {
  name = BuiltinPluginName.SyncModuleIdsPlugin;
  affectedHooks = 'compilation' as const;

  constructor(private options: SyncModuleIdsPluginOptions) {
    super();
  }

  raw(): BuiltinPlugin {
    const options: RawSyncModuleIdsPluginOptions = { ...this.options };
    return createBuiltinPlugin(this.name, options);
  }
}
