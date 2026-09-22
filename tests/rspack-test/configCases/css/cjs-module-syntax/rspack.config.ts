import { defineConfig } from '@rspack/cli';

export default defineConfig([
  {
    target: 'web',
    mode: 'development',
    module: {
      generator: {
        'css/auto': {
          esModule: false,
        },
      },
      rules: [
        {
          test: /\.css$/,
          type: 'css/auto',
        },
      ],
    },
  },
  {
    target: 'node',
    mode: 'development',
    module: {
      generator: {
        'css/auto': {
          esModule: false,
        },
      },
      rules: [
        {
          test: /\.css$/,
          type: 'css/auto',
        },
      ],
    },
  },
]);
