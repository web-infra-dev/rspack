import { defineConfig } from '@rspack/cli';

export default defineConfig(
  (['source-map', 'cheap-module-source-map'] as const).map((devtool) =>
    defineConfig({
      mode: 'development',
      devtool,
      externals: ['source-map'],
      externalsType: 'commonjs',
      optimization: { concatenateModules: false },
    }),
  ),
);
