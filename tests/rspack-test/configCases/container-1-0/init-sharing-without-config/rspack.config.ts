import { defineConfig } from '@rspack/cli';
import { container } from '@rspack/core';

const { ModuleFederationPluginV1: ModuleFederationPlugin } = container;

export default defineConfig({
  plugins: [new ModuleFederationPlugin({})],
});
