import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default defineConfig({
  output: {
    assetModuleFilename: 'assets/[name][ext]',
  },
  module: {
    rules: [
      {
        test: /\.woff2$/,
        type: 'asset/resource',
      },
    ],
  },
  plugins: [
    new rspack.container.ModuleFederationPluginV1({
      shared: {
        'pkg/': {
          requiredVersion: false,
        },
      },
    }),
  ],
});
