import { BuiltinPluginName } from '@rspack/binding';

import { create } from './base';

export const URLPlugin = create(
  BuiltinPluginName.URLPlugin,
  () => {},
  'compilation',
);
