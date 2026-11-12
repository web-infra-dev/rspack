import { defineConfig } from '@rspack/cli';

export default defineConfig(
  ['javascript/auto', 'javascript/esm'].map((type) => ({
    amd: {},
    module: {
      rules: [{ test: /\.js$/, type }],
    },
    optimization: {
      concatenateModules: false,
      usedExports: true,
    },
  })),
);
