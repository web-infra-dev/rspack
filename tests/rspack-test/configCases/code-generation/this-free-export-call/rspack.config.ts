import { defineConfig } from '@rspack/cli';

export default [false, true].map((concatenateModules, index) =>
  defineConfig({
    mode: 'production',
    node: {
      __dirname: false,
      __filename: false,
    },
    output: {
      filename: `bundle${index}.js`,
    },
    optimization: {
      concatenateModules,
      minimize: false,
      mangleExports: false,
      inlineExports: false,
    },
  }),
);
