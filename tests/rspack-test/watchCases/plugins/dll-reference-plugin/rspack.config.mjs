import { rspack as webpack } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  optimization: {
    concatenateModules: true,
  },
  plugins: [
    new webpack.DllReferencePlugin({
      name: "function(id) { return {default: 'ok'}; }",
      scope: 'dll',
      content: {
        './module': {
          id: 1,
          buildMeta: {
            exportsType: 'namespace',
            providedExports: ['default'],
          },
        },
      },
    }),
  ],
  ignoreWarnings: [/is not friendly for incremental/],
};
