/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'development',
  module: {
    rules: [
      {
        dependency: 'url',
        type: 'asset',
      },
    ],
  },
};
