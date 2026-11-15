const { DefinePlugin } = require('@rspack/core');

module.exports = [false, true].flatMap((module) =>
  [false, true].flatMap((concatenateModules) =>
    [true, 'flag'].map((sideEffects) => ({
      mode: 'production',
      output: {
        module,
        library: { type: module ? 'module' : 'commonjs2' },
        filename: module ? 'bundle.mjs' : 'bundle.js',
        pathinfo: false,
      },
      module: {
        rules: [
          { test: /declared\.js$/, sideEffects: false },
          { test: /forced\.js$/, sideEffects: true },
        ],
      },
      optimization: {
        concatenateModules,
        moduleIds: 'named',
        sideEffects,
        minimize: false,
      },
      plugins: [
        new DefinePlugin({
          MODULE_OUTPUT: JSON.stringify(module),
          CONCATENATED: JSON.stringify(concatenateModules),
          ANALYZED: JSON.stringify(sideEffects === true),
        }),
      ],
    })),
  ),
);
