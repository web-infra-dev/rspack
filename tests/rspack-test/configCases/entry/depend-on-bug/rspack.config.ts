import { defineConfig } from '@rspack/cli';

/** @typedef {import("@rspack/core").Compiler} Compiler */
/** @typedef {import("@rspack/core").Compilation} Compilation */
/** @typedef {import("@rspack/core").Configuration} Configuration */

export default defineConfig({
  entry() {
    return Promise.resolve({
      app: { import: './app.js', dependOn: ['other-vendors'] },
      page1: { import: './page1.js', dependOn: ['app'] },
      'other-vendors': './other-vendors',
    });
  },
  target: 'web',
  output: {
    filename: '[name].js',
  },
});
