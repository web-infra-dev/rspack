/** @type {import("@rspack/core").Configuration} */
export default {
  module: {
    rules: [
      {
        test: /\.toml$/,
        type: 'json',
        parser: {
          parse: () => ({ foo: 'bar' }),
        },
      },
    ],
  },
};
