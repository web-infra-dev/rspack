import { defineConfig } from '@rspack/cli';

const baseConfig = defineConfig({
  mode: 'production',
  entry: './index',
  stats: {
    assets: true,
    modules: true,
    modulesSpace: Infinity,
    optimizationBailout: true,
    nestedModules: true,
    usedExports: true,
    providedExports: true,
  },
  optimization: {
    minimize: true,
  },
});

export default defineConfig([
  baseConfig,
  {
    ...baseConfig,
    output: {
      filename: '[name].no-side.js',
    },
    optimization: {
      ...baseConfig.optimization,
      sideEffects: false,
    },
  },
]);
