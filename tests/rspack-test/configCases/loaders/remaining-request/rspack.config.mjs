/** @type {import("@rspack/core").Configuration} */
export default {
  module: {
    rules: [
      {
        test: /a\.js$/,
        use: [
          './loader1.mjs',
          {
            loader: './loader2.mjs',
            ident: 'loader2.mjs',
            options: {
              f: function () {
                return 'ok';
              },
            },
          },
        ],
      },
      {
        test: /b\.js$/,
        use: [
          './loader1.mjs',
          {
            loader: './loader2.mjs',
            options: {
              f: function () {
                return 'ok';
              },
            },
          },
        ],
      },
      {
        test: /c\.js$/,
        use: './loader1.mjs',
      },
      {
        test: /c\.js$/,
        loader: './loader2.mjs',
        options: {
          f: function () {
            return 'ok';
          },
        },
      },
    ],
  },
};
