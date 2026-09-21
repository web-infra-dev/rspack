import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: {
    bundle0: {
      import: './index.js',
      layer: 'entry-layer',
    },
  },
  output: {
    pathinfo: 'verbose',
  },
  module: {
    rules: [
      {
        test: /module-layer-change/,
        layer: 'layer',
      },
      {
        test: /module-other-layer-change/,
        layer: 'other-layer',
      },
      {
        test: /module\.js$/,
        issuerLayer: 'other-layer',
        loader: './loader.mjs',
        options: {
          value: 'other',
        },
      },
      {
        test: /module\.js$/,
        issuerLayer: 'layer',
        loader: './loader.mjs',
        options: {
          value: 'ok',
        },
      },
      {
        test: /module\.js$/,
        issuerLayer: 'entry-layer',
        loader: './loader.mjs',
        options: {
          value: 'entry',
        },
      },
      {
        test: /dynamic-module-layer/,
        layer: 'dynamic-layer',
      },
    ],
  },
  externals: [
    {
      external1: 'var 42',
    },
    {
      external2: 'var 42',
    },
  ],
});
