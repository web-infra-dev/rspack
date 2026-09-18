/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'development',
  module: {
    rules: [
      {
        mimetype: 'image/svg+xml+external',
        type: 'asset/resource',
        generator: {
          filename: '[hash].svg',
        },
      },
    ],
  },
  target: 'web',
};
