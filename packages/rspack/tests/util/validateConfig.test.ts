import { describe, expect, it } from 'rstack/test';
import { validateRspackConfig } from '../../src/util/validateConfig';

describe('validateRspackConfig', () => {
  it('should allow string[] external item values when output.library.type is umd', () => {
    expect(() =>
      validateRspackConfig({
        output: {
          library: {
            type: 'umd',
          },
        },
        externals: {
          subtract: ['./math', 'subtract'],
        },
      }),
    ).not.toThrow();

    expect(() =>
      validateRspackConfig({
        output: {
          library: {
            type: 'umd',
          },
        },
        externalsType: 'umd',
        externals: {
          byLayer: {
            ssr: {
              subtract: ['./math', 'subtract'],
            },
          },
        },
      }),
    ).not.toThrow();
  });

  it('should still reject incomplete UMD external objects', () => {
    expect(() =>
      validateRspackConfig({
        output: {
          library: {
            type: 'umd',
          },
        },
        externals: {
          lodash: {
            root: '_',
            commonjs: 'lodash',
            commonjs2: 'lodash',
          } as any,
        },
      }),
    ).toThrow(
      /External object must have "root", "commonjs", "commonjs2", "amd" properties/,
    );
  });
});
