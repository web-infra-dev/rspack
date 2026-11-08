import {
  BuiltinPluginName,
  type RawRstestPluginOptions,
} from '@rspack/binding';

import { create } from './base';

export const RstestPlugin = create(
  BuiltinPluginName.RstestPlugin,
  (rstest: RawRstestPluginOptions): RawRstestPluginOptions => {
    return rstest;
  },
);
