import {
  BuiltinPluginName,
  type SourceMapDevToolPluginOptions,
} from '@rspack/binding';

import { create } from './base';

export type EvalSourceMapDevToolPluginOptions = Pick<
  SourceMapDevToolPluginOptions,
  | 'columns'
  | 'debugIds'
  | 'exclude'
  | 'ignoreList'
  | 'include'
  | 'module'
  | 'moduleFilenameTemplate'
  | 'namespace'
  | 'noSources'
  | 'sourceRoot'
  | 'test'
>;

export const EvalSourceMapDevToolPlugin = create(
  BuiltinPluginName.EvalSourceMapDevToolPlugin,
  (options: EvalSourceMapDevToolPluginOptions): SourceMapDevToolPluginOptions =>
    options,
  'compilation',
);
