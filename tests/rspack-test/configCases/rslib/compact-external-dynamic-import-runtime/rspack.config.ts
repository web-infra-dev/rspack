import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

const {
  experiments: { RslibPlugin },
} = rspack;

const basic = defineConfig({
  plugins: [new RslibPlugin()],
  output: {
    filename: `[name].js`,
    chunkFilename: `async.js`,
    library: {
      type: 'commonjs-static',
    },
    iife: false,
  },
  externals: {
    react: 'react-alias',
    vue: 'vue-alias',
    angular: 'angular-alias',
    svelte: 'svelte-alias',
    lit: 'lit-alias',
    solid: 'solid-alias',
    jquery: 'jquery-alias',
  },
  externalsType: 'commonjs-import',
  optimization: {
    minimize: false,
  },
});

export default defineConfig([
  {
    entry: {
      main: './main.js',
    },
    ...basic,
  },
  {
    entry: {
      main2: './main2.js',
    },
    ...basic,
  },
  {
    entry: {
      index: './index.js',
    },
    output: {
      filename: 'index.js',
    },
  },
]);
