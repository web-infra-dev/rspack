import { defineConfig } from '@rspack/cli';

export default defineConfig({
  module: {
    rules: [
      {
        test: /\.js$/,
        resource: /[\\/]entry\.js$/,
        loader: './loader.mjs',
      },
      {
        test: (resource) => /\.js$/.test(resource),
        resource: /[\\/]async-entry\.js$/,
        loader: './loader.mjs',
      },
    ],
  },
});
