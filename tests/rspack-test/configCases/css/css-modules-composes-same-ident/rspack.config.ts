import { defineConfig } from '@rspack/cli';

export default defineConfig({
  module: {
    generator: {
      'css/auto': {
        localIdentName: '[path][name]-[local]',
      },
    },
    rules: [
      {
        test: /\.css$/,
        type: 'css/auto',
      },
    ],
  },
});
