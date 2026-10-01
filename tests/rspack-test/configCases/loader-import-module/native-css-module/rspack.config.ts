import { defineConfig } from '@rspack/cli';

export default defineConfig(
  ['css/module', 'css/auto'].flatMap((type) =>
    [true, false].map((exportsOnly) => ({
      module: {
        rules: [
          {
            test: /index\.js$/,
            loader: './loader.mjs',
          },
          {
            test: /\.module\.css$/,
            type,
            generator: {
              exportsOnly,
              localIdentName: '[local]',
            },
          },
        ],
      },
    })),
  ),
);
