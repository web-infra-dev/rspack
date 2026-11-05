import { defineConfig } from '@rspack/cli';
import type { Externals } from '@rspack/core';

export default defineConfig(
  ['constructor', 'toString', '__proto__'].flatMap((layer) =>
    ['base', 'default', 'explicit'].map((mode) => {
      const externals: Externals =
        mode === 'base'
          ? { bridge: 'path', byLayer: {} }
          : {
              byLayer:
                mode === 'default'
                  ? { default: { bridge: 'path' } }
                  : { [layer]: { bridge: 'path' } },
            };
      return {
        output: { library: { name: 'TestLib', type: 'umd' } },
        externalsType: 'umd',
        externals,
        module: { rules: [{ test: /module\.js$/, layer }] },
        lazyCompilation: { entries: false },
      };
    }),
  ),
);
