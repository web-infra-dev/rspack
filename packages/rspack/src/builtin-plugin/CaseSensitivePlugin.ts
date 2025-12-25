import { BuiltinPluginName } from '@rspack/binding';

import { create } from './base';

export const CaseSensitivePlugin = create(
  BuiltinPluginName.CaseSensitivePlugin,
  () => {},
  'compilation',
);
