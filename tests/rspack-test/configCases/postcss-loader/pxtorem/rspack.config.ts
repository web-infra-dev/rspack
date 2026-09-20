import { defineConfig } from '@rspack/cli';
import { fileURLToPath } from 'node:url';

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
        test: /\.css$/,
        use: [
          {
            loader: 'postcss-loader',
            options: {
              postcssOptions: {
                plugins: [
                  fileURLToPath(import.meta.resolve('postcss-pxtorem')),
                ],
              },
            },
          },
        ],
        type: 'css/auto',
        generator: {
          exportsOnly: false,
        },
      },
    ],
  },
});
