import {
  BuiltinPluginName,
  type SourceMapDevToolPluginOptions,
} from '@rspack/binding';

import { create } from './base';

export const EvalSourceMapDevToolPlugin = create(
  BuiltinPluginName.EvalSourceMapDevToolPlugin,
  (options: SourceMapDevToolPluginOptions): SourceMapDevToolPluginOptions =>
    options,
  'compilation',
);
