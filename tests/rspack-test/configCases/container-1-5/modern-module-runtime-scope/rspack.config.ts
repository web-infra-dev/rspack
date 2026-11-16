import { defineConfig } from '@rspack/cli';
import { container } from '@rspack/core';

export default defineConfig({
  entry: { bundle0: './index.js' },
  target: 'node',
  experiments: { runtimeMode: 'rspack' },
  output: {
    filename: '[name].mjs',
    module: true,
    library: { type: 'modern-module' },
    publicPath: '/mf/',
  },
  optimization: { runtimeChunk: false },
  plugins: [
    new container.ModuleFederationPlugin({
      name: 'scope_host',
      shared: {
        value: { import: './value', version: '1.0.0', requiredVersion: false },
      },
    }),
  ],
});
