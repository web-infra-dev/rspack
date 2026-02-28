import {
  BuiltinPluginName,
  type RawContextReplacementPluginOptions,
} from '@rspack/binding';

import { create } from './base';

export const ContextReplacementPlugin = create(
  BuiltinPluginName.ContextReplacementPlugin,
  (
    resourceRegExp: RegExp,
    newContentResource?: any,
    newContentRecursive?: any,
    newContentRegExp?: any,
  ) => {
    const rawOptions: RawContextReplacementPluginOptions = {
      resourceRegExp,
    };
    if (typeof newContentResource === 'function') {
      // rawOptions.newContentCallback = newContentResource;
    } else if (
      typeof newContentResource === 'string' &&
      typeof newContentRecursive === 'object'
    ) {
      rawOptions.newContentResource = newContentResource;
      rawOptions.newContentCreateContextMap = newContentRecursive;
    } else if (
      typeof newContentResource === 'string' &&
      typeof newContentRecursive === 'function'
    ) {
      rawOptions.newContentResource = newContentResource;
      // rawOptions.newContentCreateContextMap = newContentRecursive;
    } else {
      if (typeof newContentResource !== 'string') {
        newContentRegExp = newContentRecursive;
        newContentRecursive = newContentResource;
        newContentResource = undefined;
      }
      if (typeof newContentRecursive !== 'boolean') {
        newContentRegExp = newContentRecursive;
        newContentRecursive = undefined;
      }
      rawOptions.newContentResource = newContentResource;
      rawOptions.newContentRecursive = newContentRecursive;
      rawOptions.newContentRegExp = newContentRegExp;
    }
    return rawOptions;
  },
);
