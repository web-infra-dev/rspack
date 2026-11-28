import { defineConfig } from '@rspack/cli';

export default defineConfig(
  [false, true].map((dynamic) => ({
    externalsPresets: { node: false },
    externals: {
      base: 'var 42',
      value: 'var 42',
      './local': 'var 42',
      typed: { var: '42', commonjs: 'unused' },
      byLayer: dynamic
        ? (layer) =>
            layer === 'app'
              ? {
                  byLayer: () => ({
                    value: 'var 43',
                    './local': false,
                    typed: { commonjs: 'unused-layer' },
                  }),
                }
              : undefined
        : {
            app: {
              value: 'var 43',
              './local': false,
              typed: { commonjs: 'unused-layer' },
            },
          },
    },
    module: {
      rules: [
        { test: /app\.js$/, layer: 'app' },
        { test: /other\.js$/, layer: 'other' },
      ],
    },
  })),
);
