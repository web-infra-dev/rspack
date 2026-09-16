/** @type {import("@rspack/core").Configuration} */
export default {
  module: {
    rules: [
      {
        test: /__label__/,
        use: (info) => {
          return [
            {
              loader: './loader.js',
              options: info,
            },
          ];
        },
      },
    ],
  },
};
