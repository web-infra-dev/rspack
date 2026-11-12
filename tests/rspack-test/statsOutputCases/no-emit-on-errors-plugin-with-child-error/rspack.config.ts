import { defineConfig } from '@rspack/cli';
import { NoEmitOnErrorsPlugin } from '@rspack/core';
import TestChildCompilationFailurePlugin from './TestChildCompilationFailurePlugin.mjs';

export default defineConfig({
  entry: './index',
  output: {
    filename: 'bundle.js',
  },
  plugins: [
    new NoEmitOnErrorsPlugin(),
    new TestChildCompilationFailurePlugin({
      filename: 'child.js',
    }),
  ],
});
