import { defineConfig } from '@rspack/cli';
import path from 'node:path';

const base = defineConfig({
  mode: 'production',
  entry: './src/index.js',
  devtool: false,
  output: {
    filename: 'main.js',
    assetModuleFilename: '[contenthash][ext]',
  },
  module: {
    rules: [
      {
        test: /\.(png|jpg)$/,
        type: 'asset/resource',
      },
    ],
  },
  stats: 'normal',
  context: import.meta.dirname,
});

export default defineConfig([
  {
    ...base,
    output: {
      ...base.output,
      path: path.resolve(import.meta.dirname, './dist/enable'),
    },
    optimization: {
      realContentHash: true,
    },
  },
  {
    ...base,
    output: {
      ...base.output,
      path: path.resolve(import.meta.dirname, './dist/disable'),
    },
    optimization: {
      realContentHash: false,
    },
  },
]);
