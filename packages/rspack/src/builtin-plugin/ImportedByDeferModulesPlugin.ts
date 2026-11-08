import { BuiltinPluginName } from '@rspack/binding';

import { create } from './base';

export const ImportedByDeferModulesPlugin = create(
  BuiltinPluginName.ImportedByDeferModulesPlugin,
  () => {},
  'compilation',
);
