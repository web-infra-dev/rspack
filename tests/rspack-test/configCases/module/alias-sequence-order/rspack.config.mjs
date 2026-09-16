/**
 * @type {import('@rspack/core').RspackOptions}
 */
export default {
  module: {
    rules: [
      {
        resolve: {
          alias: {
            'foo/bar': './exist',
          },
        },
      },
      {
        resolve: {
          alias: {
            foo: './not-exist',
          },
        },
      },
    ],
  },
};
