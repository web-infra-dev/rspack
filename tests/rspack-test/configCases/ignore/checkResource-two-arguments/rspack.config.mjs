import { IgnorePlugin } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  entry: './test.js',
  plugins: [
    new IgnorePlugin({
      checkResource(resource, context) {
        return /ignored-module/.test(resource) && /folder-b/.test(context);
      },
    }),
  ],
};
