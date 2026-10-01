import { NoEmitOnErrorsPlugin } from '@rspack/core';
import TestChildCompilationFailurePlugin from './TestChildCompilationFailurePlugin.mjs';

/** @type {import("@rspack/core").Configuration} */
export default {
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
};
