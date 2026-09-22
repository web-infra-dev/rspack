import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: {
    main: './index.js',
  },
  module: {
    parser: {
      'css/module': {
        namedExports: false,
      },
    },
    rules: [
      {
        test: /\.module\.css$/,
        type: 'css/module',
        generator: {
          exportsOnly: true,
        },
      },
    ],
  },
});
