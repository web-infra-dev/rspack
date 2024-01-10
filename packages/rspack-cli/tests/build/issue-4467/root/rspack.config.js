const { rspack } = require("@rspack/core");
const path = require("path");

const config = {
	entry: path.resolve(__dirname, "./index.js"),
	plugins: [
		new rspack.BannerPlugin({
			banner: ""
		})
	]
};

module.exports = config;
