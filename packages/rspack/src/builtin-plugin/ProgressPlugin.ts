import {
  BuiltinPluginName,
  type RawProgressPluginOptions,
} from '@rspack/binding';

import { create } from './base';

export type ProgressPluginArgument =
  | Partial<Omit<RawProgressPluginOptions, 'handler'>>
  | ((
      percentage: number,
      msg: string,
      info: { builtModules: number; moduleIdentifier?: string },
    ) => void)
  | undefined;
export const ProgressPlugin = create(
  BuiltinPluginName.ProgressPlugin,
  (progress: ProgressPluginArgument = {}): RawProgressPluginOptions => {
    if (typeof progress === 'function') {
      return {
        handler: (percentage, msg, info) => {
          progress(percentage, msg, info);
        },
      };
    }
    return progress;
  },
);
