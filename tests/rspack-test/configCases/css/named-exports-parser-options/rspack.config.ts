import { defineConfig } from '@rspack/cli';

export default defineConfig({
  target: 'node',
  mode: 'development',
  module: {
    rules: [
      {
        test: /\.css/,
        type: 'css/auto',
      },
      {
        resourceQuery: /\?default/,
        parser: {
          namedExports: false,
        },
        type: 'css/module',
      },
      {
        resourceQuery: /\?named/,
        parser: {
          namedExports: true,
        },
        type: 'css/module',
      },
    ],
  },
});
