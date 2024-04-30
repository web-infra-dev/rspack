const rspack = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  plugins: [
    new rspack.BannerPlugin({
      banner: "global.bannerIndex = typeof global.bannerIndex === 'number' ? global.bannerIndex + 1 : 0;",
      raw: true,
    })
  ]
};
