import { defineConfig } from '@rspack/cli';

export default defineConfig({
  module: {
    rules: [
      {
        test: /\.less$/,
        type: 'asset/source',
        use: [
          {
            loader: 'builtin:less-loader',
            options: { unknownLoaderOption: true },
          },
        ],
      },
    ],
  },
});
