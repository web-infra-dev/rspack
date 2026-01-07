import {
  BuiltinPluginName,
  type RawFlagAllModulesAsUsedPluginOptions,
} from '@rspack/binding';
import { create } from './base';

export const FlagAllModulesAsUsedPlugin = create(
  BuiltinPluginName.FlagAllModulesAsUsedPlugin,
  (explanation: string): RawFlagAllModulesAsUsedPluginOptions => {
    return {
      explanation,
    };
  },
);
