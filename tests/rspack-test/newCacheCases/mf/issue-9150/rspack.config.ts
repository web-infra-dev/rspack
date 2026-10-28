import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default defineConfig({
  context: import.meta.dirname,
  optimization: {
    minimize: false,
  },
  cache: {
    type: 'persistent',
  },
  plugins: [
    new rspack.container.ModuleFederationPlugin({
      shared: {
        react: {
          requiredVersion: '^19.0.0',
          version: '19.0.0',
          singleton: true,
          eager: true,
        },
      },
    }),
  ],
});
