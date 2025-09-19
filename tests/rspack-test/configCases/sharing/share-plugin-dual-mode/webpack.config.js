const { SharePlugin } = require("@rspack/core").sharing;

/** @type {import("../../../../").Configuration} */
module.exports = {
  context: `${__dirname}/cjs`,
  plugins: [
    new SharePlugin({
      shared: {
        lib: {},
        transitive_lib: {},
      },
    }),
  ],
};
