import { defineConfig } from '@rspack/cli';

import path from 'node:path';

export default defineConfig({
  externals: {
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
  },
  target: 'web',
  node: false,
  module: {
    rules: [
      {
        test: /\.s[ac]ss$/i,
        use: [{ loader: 'sass-loader' }],
        type: 'css',
        generator: {
          exportsOnly: false,
        },
      },
    ],
  },
  resolve: {
    alias: {
      'path-to-alias': path.resolve(import.meta.dirname, 'a', `alias.scss`),
      '@scss': path.resolve(
        import.meta.dirname,
        'b',
        'directory-6',
        `_index.scss`,
      ),
      '@path-to-scss-dir': path.resolve(import.meta.dirname, 'b'),
      '@/path-to-scss-dir': path.resolve(import.meta.dirname, 'b'),
    },
  },
});
