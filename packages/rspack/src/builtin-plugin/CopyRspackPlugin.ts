import {
  BuiltinPluginName,
  type RawCopyPattern,
  type RawCopyRspackPluginOptions,
} from '@rspack/binding';

import { create } from './base';

export type CopyRspackPluginOptions = {
  /** An array of objects that describe the copy operations to be performed. */
  patterns: (
    | string
    | (Pick<RawCopyPattern, 'from'> & Partial<Omit<RawCopyPattern, 'from'>>)
  )[];
};

export const CopyRspackPlugin = create(
  BuiltinPluginName.CopyRspackPlugin,
  (copy: CopyRspackPluginOptions): RawCopyRspackPluginOptions => {
    const ret: RawCopyRspackPluginOptions = {
      patterns: [],
    };

    ret.patterns = (copy.patterns || []).map((pattern) => {
      if (typeof pattern === 'string') {
        pattern = { from: pattern };
      }

      pattern.force ??= false;
      pattern.noErrorOnMissing ??= false;
      pattern.priority ??= 0;
      pattern.globOptions ??= {};
      pattern.copyPermissions ??= false;

      const originalTransform = pattern.transform;
      if (originalTransform) {
        if (typeof originalTransform === 'object') {
          pattern.transform = (input, absoluteFilename) =>
            Promise.resolve(
              originalTransform.transformer(input, absoluteFilename),
            ) as Promise<string> | Promise<Buffer>;
        } else {
          pattern.transform = (input, absoluteFilename) =>
            Promise.resolve(originalTransform(input, absoluteFilename)) as
              | Promise<string>
              | Promise<Buffer>;
        }
      }

      return pattern as RawCopyPattern;
    });

    return ret;
  },
);
