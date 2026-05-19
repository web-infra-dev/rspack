import {
  BuiltinPluginName,
  type RawProgressPluginHandlerInfo,
  type RawProgressPluginOptions,
} from '@rspack/binding';

import { create } from './base';

export type ProgressPluginOptions =
  | Partial<Omit<RawProgressPluginOptions, 'handler'>>
  | ((
      percentage: number,
      msg: string,
      info: RawProgressPluginHandlerInfo,
    ) => void)
  | undefined;

export type ProgressPluginHandlerInfo = RawProgressPluginHandlerInfo;

export const ProgressPlugin = create(
  BuiltinPluginName.ProgressPlugin,
  (progress: ProgressPluginOptions = {}): RawProgressPluginOptions => {
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
