import { defineConfig } from '@rspack/cli';
import { container } from '@rspack/core';

export default defineConfig({
  externals: {
    './federation.js': 'commonjs ./federation.js',
  },
  plugins: [
    new container.ModuleFederationPluginV1({
      name: 'federation',
      filename: 'federation.js',
      library: { type: 'commonjs-module' },
      shareScope: ['custom'],
      exposes: { './scope': './scope.js' },
    }),
  ],
});
