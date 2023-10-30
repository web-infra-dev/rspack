const path = require("path");

const config = {
	entry: path.resolve(__dirname, "./index.js"),
	builtins: {
		react: {}
	},
	experiments: {
		rspackFuture: {
			disableTransformByDefault: true
		}
	}
};

module.exports = config;
