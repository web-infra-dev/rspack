import { BuiltinPluginName, type RawRslibPluginOptions } from '@rspack/binding';

import { create } from './base';

export type RslibPluginArgument =
  | Partial<Omit<RawRslibPluginOptions, 'handler'>>
  | ((percentage: number, msg: string, ...args: string[]) => void)
  | undefined;

export const RslibPlugin = create(
  BuiltinPluginName.RslibPlugin,
  (rslib: RawRslibPluginOptions): RawRslibPluginOptions => {
    return rslib;
  },
);
