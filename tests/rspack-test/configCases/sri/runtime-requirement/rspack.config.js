const { SubresourceIntegrityPlugin } = require("@rspack/core");

module.exports = {
	mode: "production",
	target: "web",
	entry: {
		main: "./index.js"
	},
	output: {
		crossOriginLoading: "anonymous",
	},
	plugins: [new SubresourceIntegrityPlugin()],
};
