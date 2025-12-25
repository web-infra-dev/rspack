import { BuiltinPluginName } from '@rspack/binding';

import { create } from './base';

export const InlineExportsPlugin = create(
  BuiltinPluginName.InlineExportsPlugin,
  () => {},
  'compilation',
);
