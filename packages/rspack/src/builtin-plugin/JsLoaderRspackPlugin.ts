import { BuiltinPluginName } from '@rspack/binding';

import { create } from './base';

export const JsLoaderRspackPlugin = create(
  BuiltinPluginName.JsLoaderRspackPlugin,
  () => undefined,
  /* Not Inheretable */
  'thisCompilation',
);
