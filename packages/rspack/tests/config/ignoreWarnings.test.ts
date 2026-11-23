import {
  config,
  WebpackError,
  type Compilation,
  type Module,
} from '@rspack/core';
import { describe, expect, it } from 'rstack/test';

describe.each(['shorthand', 'message', 'module', 'file'])(
  'ignoreWarnings %s',
  (field) => {
    it.each(['', 'g', 'y', 'gy'])(
      'preserves lastIndex for /warning/%s across repeated matches',
      (flags) => {
        const pattern = new RegExp('warning', flags);
        pattern.lastIndex = 3;
        const rule = field === 'shorthand' ? pattern : { [field]: pattern };
        const [filter] = config.getNormalizedRspackOptions({
          ignoreWarnings: [rule],
        }).ignoreWarnings!;
        const compilation = {} as Compilation;
        const warning = Object.assign(new WebpackError('warning'), {
          file: 'warning',
          module: { readableIdentifier: () => 'warning' } as Module,
        });

        for (let i = 0; i < 3; i++) {
          expect(filter(warning, compilation)).toBe(true);
        }
        expect(pattern.lastIndex).toBe(3);
        expect(filter(new WebpackError('different'), compilation)).toBe(false);
      },
    );
  },
);
