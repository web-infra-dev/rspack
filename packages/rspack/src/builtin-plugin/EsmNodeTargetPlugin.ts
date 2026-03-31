import { BuiltinPluginName } from '@rspack/binding';

import { create } from './base';

export const EsmNodeTargetPlugin = create(
  BuiltinPluginName.EsmNodeTargetPlugin,
  () => undefined,
);
