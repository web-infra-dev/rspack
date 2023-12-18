const rspack = require("@rspack/core");
/** @type {import('@rspack/cli').Configuration} */
const config = {
	experiments: {
		rspackFuture: {
			newTreeshaking: true
		}
	},
	optimization: {
		minimize: false,
		moduleIds: 'named'
	},
	entry: "./index.js",
};
module.exports = config;
