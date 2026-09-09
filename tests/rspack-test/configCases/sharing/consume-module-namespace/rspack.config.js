const { sharing } = require('@rspack/core');

module.exports = [false, true].flatMap((eager) =>
  [false, true].map((concatenateModules) => ({
    mode: 'production',
    optimization: {
      concatenateModules,
      minimize: false,
    },
    plugins: [
      new sharing.SharePlugin({
        shared: {
          './cjs.js': { eager, singleton: true, requiredVersion: false },
          './esm.mjs': { eager, singleton: true, requiredVersion: false },
        },
      }),
    ],
  })),
);
