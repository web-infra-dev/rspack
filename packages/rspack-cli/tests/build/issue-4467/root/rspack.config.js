const path = require("path");

const config = {
	entry: path.resolve(__dirname, "./index.js"),
	builtins: {
		react: {}
	}
};

module.exports = config;
