import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: {
    main: './index.js',
    css: './css',
  },
  output: {
    filename: '[name].js',
  },
  module: {
    generator: {
      'css/auto': {
        exportsConvention: 'camel-case',
        exportsOnly: true,
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
