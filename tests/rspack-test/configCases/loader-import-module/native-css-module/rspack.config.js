/** @type {import("@rspack/core").Configuration[]} */
module.exports = ['css/module', 'css/auto'].flatMap((type) =>
  [true, false].map((exportsOnly) => ({
    module: {
      rules: [
        {
          test: /index\.js$/,
          loader: './loader.js',
        },
        {
          test: /\.module\.css$/,
          type,
          generator: {
            exportsOnly,
            localIdentName: '[local]',
          },
        },
      ],
    },
  })),
);
