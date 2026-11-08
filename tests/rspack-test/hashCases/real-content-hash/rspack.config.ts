import { defineConfig } from '@rspack/cli';
import path from 'node:path';

const base = defineConfig({
  mode: 'production',
  entry: {
    index: {
      import: './index',
      runtime: 'runtime',
    },
    a: './a',
    b: './b',
  },
  module: {
    rules: [
      {
        test: /\.(png|jpg)$/,
        type: 'asset/resource',
      },
    ],
  },
  optimization: {
    realContentHash: true,
  },
  stats: 'normal',
});

export default defineConfig([
  {
    ...base,
    name: 'a-normal',
    context: path.resolve(import.meta.dirname, 'a'),
    devtool: false,
    output: {
      path: path.resolve(import.meta.dirname, './dist/a-normal'),
      filename: '[contenthash]-[contenthash:6].js',
      assetModuleFilename: '[contenthash][ext]',
    },
  },
  {
    ...base,
    name: 'b-normal',
    context: path.resolve(import.meta.dirname, 'b'),
    devtool: false,
    output: {
      path: path.resolve(import.meta.dirname, './dist/b-normal'),
      filename: '[contenthash]-[contenthash:6].js',
      assetModuleFilename: '[contenthash][ext]',
    },
  },
  {
    ...base,
    name: 'a-source-map',
    context: path.resolve(import.meta.dirname, 'a'),
    devtool: 'source-map',
    output: {
      path: path.resolve(import.meta.dirname, './dist/a-source-map'),
      filename: '[contenthash]-[contenthash:6].js',
      assetModuleFilename: '[contenthash][ext]',
    },
  },
  {
    ...base,
    name: 'b-source-map',
    context: path.resolve(import.meta.dirname, 'b'),
    devtool: 'source-map',
    output: {
      path: path.resolve(import.meta.dirname, './dist/b-source-map'),
      filename: '[contenthash]-[contenthash:6].js',
      assetModuleFilename: '[contenthash][ext]',
    },
  },
]);
