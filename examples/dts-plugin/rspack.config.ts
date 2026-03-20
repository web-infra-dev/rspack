import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import {
  rspack,
  type SwcLoaderOptions,
} from '../../packages/rspack/dist/index.js';

const root = dirname(fileURLToPath(import.meta.url));

export default {
  context: root,
  mode: 'development',
  entry: {
    main: './src/index.ts',
  },
  output: {
    clean: true,
    path: resolve(root, 'dist'),
  },
  resolve: {
    extensions: ['...', '.ts'],
  },
  module: {
    rules: [
      {
        test: /\.ts$/,
        use: [
          {
            loader: 'builtin:swc-loader',
            options: {
              jsc: {
                parser: {
                  syntax: 'typescript',
                },
              },
            } satisfies SwcLoaderOptions,
          },
        ],
      },
    ],
  },
  plugins: [
    new rspack.experiments.DtsPlugin({
      entries: {
        index: './types/index.d.ts',
      },
    }),
  ],
};
