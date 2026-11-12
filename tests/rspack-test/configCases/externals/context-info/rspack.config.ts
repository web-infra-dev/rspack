import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: './index.js',
  module: {
    rules: [
      {
        test: /other-layer\.js$/,
        layer: 'other-layer',
      },
    ],
  },
  externals: [
    function ({ request, contextInfo }, callback) {
      if (request === 'external-pkg') {
        if (contextInfo?.issuerLayer === 'other-layer') {
          return callback(undefined, 'var 2');
        }
        return callback(undefined, 'var 1');
      }
      return callback();
    },
  ],
});
