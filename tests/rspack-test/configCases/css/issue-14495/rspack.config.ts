import { defineConfig } from '@rspack/cli';

export default defineConfig({
  externals: {
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
  },
  target: 'web',
  mode: 'development',
  node: {
    __dirname: false,
    __filename: false,
  },
  output: {
    cssFilename: 'bundle0.css',
  },
  module: {
    generator: {
      'css/auto': {
        localIdentName: '[name]_module_css-[local]',
      },
    },
    rules: [
      {
        test: /\.module\.css$/,
        type: 'css/auto',
      },
    ],
  },
});
