import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'development',
  output: {
    environment: {
      templateLiteral: false,
    },
  },
  module: {
    rules: [
      {
        test: /\.(png|svg|jpg)$/,
        type: 'asset/resource',
      },
    ],
  },
});
