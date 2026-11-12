import { defineConfig } from '@rspack/cli';

export default defineConfig({
  module: {
    parser: {
      'css/auto': {
        namedExports: false,
      },
    },
    rules: [
      {
        test: /\.module\.css/,
        type: 'css/auto',
      },
      {
        test: /\.css$/,
        type: 'css/auto',
      },
    ],
  },
});
