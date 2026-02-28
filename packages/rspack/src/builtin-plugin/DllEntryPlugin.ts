import {
  BuiltinPluginName,
  type RawDllEntryPluginOptions,
} from '@rspack/binding';
import { create } from './base';

export type DllEntryPluginOptions = {
  name: string;
};

export const DllEntryPlugin = create(
  BuiltinPluginName.DllEntryPlugin,
  (
    context: string,
    entries: string[],
    options: DllEntryPluginOptions,
  ): RawDllEntryPluginOptions => {
    return {
      context,
      entries,
      name: options.name,
    };
  },
);
