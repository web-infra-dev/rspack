import { defineConfig } from '@rspack/cli';

export default defineConfig({
  module: {
    generator: {
      css: {
        exportsOnly: false,
      },
    },
    rules: [
      {
        test: /\.module\.css$/,
        type: 'css/module',
        generator: {
          localIdentName: '[path][name][ext]-[local]',
          exportsOnly: true,
        },
      },
      {
        test: /\.css$/,
        type: 'css',
      },
    ],
  },
});
