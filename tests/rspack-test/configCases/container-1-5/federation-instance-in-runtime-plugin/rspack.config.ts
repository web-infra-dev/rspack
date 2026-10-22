import { defineConfig } from '@rspack/cli';
import { container } from '@rspack/core';

const { ModuleFederationPlugin } = container;

export default defineConfig({
  optimization: {
    // concatenateModules: false,
    moduleIds: 'named',
  },
  output: {
    filename: 'someDir/[name].js',
    chunkFilename: 'someDir/[name].js',
  },
  plugins: [
    new ModuleFederationPlugin({
      filename: 'someDir/container.js',
      runtimePlugins: ['./plugin.js'],
    }),
  ],
});
