import { defineConfig } from '@rspack/cli';

export default defineConfig([
  {
    target: 'web',
    optimization: {
      chunkIds: 'named',
    },

    module: {
      rules: [
        {
          test: /\.css$/,
          type: 'css/auto',
        },
      ],
    },
  },
]);
