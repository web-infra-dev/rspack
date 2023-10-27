const rspack = require("@rspack/core");
/** @type {import('@rspack/cli').Configuration} */
const config = {
	mode: "development",
	entry: {
		main: {
			import: ["./index.js"]
		}
	},
	module: {
		rules: [],
		parser: {
			asset: {
				dataUrlCondition: {
					maxSize: 1
				}
			}
		}
	},
	optimization: {
		splitChunks: {
			cacheGroups: {
				vendor: {
					chunks: "all",
					name: "vendor",
					test: /common/
				}
			}
		}
	},
	plugins: [
		new rspack.HtmlRspackPlugin(),
		new rspack.DefinePlugin({
			"process.env.NODE_ENV": "'development'"
		})
	]
};
module.exports = config;
