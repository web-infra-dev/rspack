import {
  BuiltinPluginName,
  type RawRemoveDuplicateModulesPluginOptions,
} from '@rspack/binding';

import { create } from './base';

export interface RemoveDuplicateModulesPluginOptions {
  /**
   * The minimum total size of a duplicated module group before extracting it.
   * @default 0
   */
  minSize?: number;
  /**
   * The minimum estimated size reduction before extracting a duplicated
   * module group.
   * @default 0
   */
  minSizeReduction?: number;
}

export const RemoveDuplicateModulesPlugin = create(
  BuiltinPluginName.RemoveDuplicateModulesPlugin,
  (
    options: RemoveDuplicateModulesPluginOptions = {},
  ): RawRemoveDuplicateModulesPluginOptions => {
    return {
      minSize: options.minSize,
      minSizeReduction: options.minSizeReduction,
    };
  },
);
