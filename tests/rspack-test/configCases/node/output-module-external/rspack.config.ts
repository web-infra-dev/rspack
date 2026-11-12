import { defineConfig } from '@rspack/cli';
import { DefinePlugin } from '@rspack/core';

export default defineConfig({
  target: 'node',
  entry: {
    require: './require.js',
    import: './import.js',
  },
  output: {
    module: true,
    filename: '[name].mjs',
  },
  plugins: [
    new DefinePlugin({
      NODE_VERSION: JSON.stringify(
        process.versions.node.split('.').map(Number)[0],
      ),
    }),
  ],
});
