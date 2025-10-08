const { SharePlugin } = require("@rspack/core").sharing;

/** @type {import("../../../../").Configuration} */
module.exports = {
  context: `${__dirname}/app1`,
  plugins: [
    new SharePlugin({
      shared: {
        lib1: {},
        lib2: {
          singleton: true,
        },
      },
    }),
  ],
};
