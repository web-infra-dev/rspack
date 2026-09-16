/**
 * @type {import('@rspack/core').RspackOptions}
 */
export default {
  module: {
    rules: [
      {
        resolve: {
          alias: {
            foo: './not-exist',
          },
        },
      },
      {
        resolve: {
          alias: {
            foo: './exist',
          },
        },
      },
    ],
  },
};
