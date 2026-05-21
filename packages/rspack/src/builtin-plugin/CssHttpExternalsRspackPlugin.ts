import { BuiltinPluginName } from '@rspack/binding';

import { create } from './base';

export const CssHttpExternalsRspackPlugin = create(
  BuiltinPluginName.CssHttpExternalsRspackPlugin,
  () => undefined,
);
