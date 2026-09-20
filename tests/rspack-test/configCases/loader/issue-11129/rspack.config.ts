import { defineConfig } from '@rspack/cli';
import path from 'node:path';

export default defineConfig({
  module: {
    rules: [
      {
        test: path.resolve(import.meta.dirname, 'lib.js'),
        resourceQuery: /inline/,
        use: 'exports-loader?type=commonjs&exports=single|lamejs',
      },
      {
        test: path.resolve(import.meta.dirname, 'lib.js'),
        resourceQuery: /object/,
        use: {
          loader: 'exports-loader',
          options: {
            type: 'commonjs',
            exports: 'single|lamejs',
          },
        },
      },
    ],
  },
});
