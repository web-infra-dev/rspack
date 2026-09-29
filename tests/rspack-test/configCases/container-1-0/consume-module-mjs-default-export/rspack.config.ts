import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default defineConfig({
  mode: 'development',
  plugins: [
    new rspack.container.ModuleFederationPluginV1({
      shared: ['./shared-esm-pkg/index.js'],
    }),
  ],
});
