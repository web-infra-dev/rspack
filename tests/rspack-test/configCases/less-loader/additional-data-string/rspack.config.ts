import { defineConfig } from '@rspack/cli';

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
        test: /\.less$/,
        use: [
          {
            loader: 'less-loader',
            options: {
              additionalData: '@background: coral;',
            },
          },
        ],
        type: 'css',
        generator: {
          exportsOnly: false,
        },
      },
    ],
  },
});
