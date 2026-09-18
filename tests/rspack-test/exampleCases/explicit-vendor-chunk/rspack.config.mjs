import path from 'node:path';
import { rspack as webpack } from '@rspack/core';
export default [
  {
    name: 'vendor',
    // mode: "development" || "production",
    entry: ['./vendor', './vendor2'],
    output: {
      path: path.resolve(import.meta.dirname, 'dist'),
      filename: 'vendor.js',
      library: 'vendor_[fullhash]',
    },
    plugins: [
      new webpack.DllPlugin({
        name: 'vendor_[fullhash]',
        path: path.resolve(import.meta.dirname, 'dist/manifest.json'),
      }),
    ],
  },

  {
    name: 'app',
    // mode: "development" || "production",
    dependencies: ['vendor'],
    entry: {
      pageA: './pageA',
      pageB: './pageB',
      pageC: './pageC',
    },
    output: {
      path: path.join(import.meta.dirname, 'dist'),
      filename: '[name].js',
    },
    plugins: [
      new webpack.DllReferencePlugin({
        manifest: path.resolve(import.meta.dirname, 'dist/manifest.json'),
      }),
    ],
  },
];
