const rspack = require("@rspack/core");

module.exports = {
  plugins: [
    new rspack.BannerPlugin({
      banner: "global.bannerIndex = typeof global.bannerIndex === 'number' ? global.bannerIndex + 1 : 0;",
      raw: true,
    })
  ]
};
