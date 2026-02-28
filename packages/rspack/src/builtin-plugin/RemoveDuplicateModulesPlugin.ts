import { BuiltinPluginName } from '@rspack/binding';

import { create } from './base';

export const RemoveDuplicateModulesPlugin = create(
  BuiltinPluginName.RemoveDuplicateModulesPlugin,
  () => {
    return {};
  },
);
