import {
  BuiltinPluginName,
  type RawDllReferenceAgencyPluginOptions,
} from '@rspack/binding';
import { create } from './base';

export type DllReferenceAgencyPluginOptions =
  RawDllReferenceAgencyPluginOptions;

export const DllReferenceAgencyPlugin = create(
  BuiltinPluginName.DllReferenceAgencyPlugin,
  (
    options: DllReferenceAgencyPluginOptions,
  ): RawDllReferenceAgencyPluginOptions => options,
);
