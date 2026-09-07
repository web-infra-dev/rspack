import { BuiltinPluginName, type RawRslibPluginOptions } from '@rspack/binding';

import { create } from './base';

export const RslibPlugin = create(
  BuiltinPluginName.RslibPlugin,
  (rslib: RawRslibPluginOptions): RawRslibPluginOptions => {
    return rslib;
  },
);
