import {
  BuiltinPluginName,
  type RawHashedModuleIdsPluginOptions,
} from '@rspack/binding';

import { create } from './base';

export const HashedModuleIdsPlugin = create(
  BuiltinPluginName.HashedModuleIdsPlugin,
  (options?: RawHashedModuleIdsPluginOptions) => ({ ...options }),
  'compilation',
);
