const { rspack } = require('@rspack/core');

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  entry: {
    main: ['./index.js'],
  },
  plugins: [
    new rspack.DefinePlugin({
      ENV: {
        NODE_ENV: '"production"',
        DEBUG: true,
        DEEP: { A: '1', B: '2' },
      },
    }),
  ],
};
