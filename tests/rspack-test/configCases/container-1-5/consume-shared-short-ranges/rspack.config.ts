import { defineConfig } from '@rspack/cli';
import { container } from '@rspack/core';

const strict = (requiredVersion: string) => ({
  requiredVersion,
  singleton: true,
  strictVersion: true,
});

export default defineConfig({
  plugins: [
    new container.ModuleFederationPluginV1({
      name: 'container',
      filename: 'container.js',
      library: { type: 'commonjs-module' },
      shared: {
        'tilde-lib': strict('~1'),
        'caret-zero-lib': strict('^0'),
        'caret-zero-zero-lib': strict('^0.0'),
      },
    }),
  ],
});
