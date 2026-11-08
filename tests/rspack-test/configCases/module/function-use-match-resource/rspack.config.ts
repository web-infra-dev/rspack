import { defineConfig } from '@rspack/cli';

export default defineConfig({
  module: {
    rules: [
      {
        test: /__label__/,
        use: (info) => {
          return [
            {
              loader: './loader.mjs',
              options: info,
            },
          ];
        },
      },
    ],
  },
});
