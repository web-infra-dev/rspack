import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'development',
  entry: {
    main: './index.js',
  },
  optimization: {
    concatenateModules: true,
    minimize: false,
  },
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'css/module',
        parser: {
          namedExports: false,
        },
        generator: {
          exportsOnly: true,
          localIdentName: '[path][name][ext]__[local]',
        },
      },
    ],
  },
});
