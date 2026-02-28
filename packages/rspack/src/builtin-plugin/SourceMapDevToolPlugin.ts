import {
  BuiltinPluginName,
  type SourceMapDevToolPluginOptions,
} from '@rspack/binding';

import { create } from './base';

export type { SourceMapDevToolPluginOptions };

export const SourceMapDevToolPlugin = create(
  BuiltinPluginName.SourceMapDevToolPlugin,
  (options: SourceMapDevToolPluginOptions) => options,
  'compilation',
);
