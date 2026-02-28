import {
  BuiltinPluginName,
  type RawNormalModuleReplacementPluginOptions,
} from '@rspack/binding';
import type { ResolveData } from '../Module';
import { create } from './base';

export const NormalModuleReplacementPlugin = create(
  BuiltinPluginName.NormalModuleReplacementPlugin,
  (
    resourceRegExp: RegExp,
    newResource: string | ((data: ResolveData) => void),
  ): RawNormalModuleReplacementPluginOptions => {
    return {
      resourceRegExp,
      newResource:
        typeof newResource === 'function'
          ? (data) => {
              newResource(data);
              return data;
            }
          : newResource,
    };
  },
);
