const { DefinePlugin } = require('@rspack/core');

module.exports = [true, 'flag'].flatMap((sideEffects) =>
  [false, true].map((minimize) => ({
    mode: 'production',
    module: {
      rules: [
        { test: /declared\.js$/, sideEffects: false },
        { test: /forced\.js$/, sideEffects: true },
      ],
    },
    output: { pathinfo: false },
    optimization: {
      concatenateModules: false,
      moduleIds: 'named',
      sideEffects,
      minimize,
    },
    plugins: [
      new DefinePlugin({
        MINIMIZED: JSON.stringify(minimize),
        ANALYZED: JSON.stringify(sideEffects === true),
      }),
    ],
  })),
);
