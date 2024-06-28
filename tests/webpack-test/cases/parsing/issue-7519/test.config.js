const rspack = require("@rspack/core");
/**@type {import("@rspack/cli").Configuration} */
const config = {
	builtins: {
		treeShaking: false
	},
	plugins: [
		new rspack.DefinePlugin({
      'process.env.NODE_ENV': "'development'",
		})
	]
};
module.exports = config;
