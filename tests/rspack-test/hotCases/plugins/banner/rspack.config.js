const { rspack } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	plugins: [
		new rspack.BannerPlugin({
			banner:
				"globalThis.bannerIndex = typeof globalThis.bannerIndex === 'number' ? globalThis.bannerIndex + 1 : 0;",
			raw: true
		})
	]
};
