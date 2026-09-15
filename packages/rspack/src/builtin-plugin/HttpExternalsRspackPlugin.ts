import {
  BuiltinPluginName,
  type RawHttpExternalsRspackPluginOptions,
} from '@rspack/binding';

import { create } from './base';

export const HttpExternalsRspackPlugin = create(
  BuiltinPluginName.HttpExternalsRspackPlugin,
  (webAsync: boolean): RawHttpExternalsRspackPluginOptions => {
    return {
      webAsync,
    };
  },
);
