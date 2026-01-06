import { BuiltinPluginName } from '@rspack/binding';

import { create } from './base';

export const InnerGraphPlugin = create(
  BuiltinPluginName.InnerGraphPlugin,
  () => {},
  'compilation',
);
