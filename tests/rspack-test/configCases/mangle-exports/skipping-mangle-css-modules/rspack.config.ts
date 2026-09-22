import { defineConfig } from '@rspack/cli';

export default defineConfig({
  optimization: {
    minimize: false,
    moduleIds: 'named',
  },
  module: {
    rules: [
      {
        test: /\.module\.css$/,
        type: 'css/module',
        generator: {
          exportsOnly: true,
        },
        parser: {
          namedExports: false,
        },
      },
    ],
  },
  entry: {
    main: './index.js',
  },
});
