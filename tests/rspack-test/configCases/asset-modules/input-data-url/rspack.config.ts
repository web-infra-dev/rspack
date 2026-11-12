import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'development',
  module: {
    rules: [
      {
        mimetype: 'image/svg+xml+external',
        type: 'asset/resource',
        generator: {
          filename: '[hash].svg',
        },
      },
    ],
  },
  target: 'web',
});
