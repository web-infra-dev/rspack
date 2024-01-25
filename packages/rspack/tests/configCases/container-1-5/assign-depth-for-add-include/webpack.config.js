const rspack = require("@rspack/core");

module.exports = {
	plugins: [
		new rspack.sharing.ProvideSharedPlugin({
			provides: ["./a/index.js"]
		})
	]
};
