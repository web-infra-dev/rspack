import { defineConfig } from '@rspack/cli';

export default defineConfig({
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'css/module',
        generator: {
          exportsOnly: true,
        },
      },
    ],
  },
});
