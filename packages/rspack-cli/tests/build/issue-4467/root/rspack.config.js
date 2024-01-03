const path = require("path");

const config = {
	entry: path.resolve(__dirname, "./index.js"),
	builtins: {
		banner: {
			banner: ""
		}
	}
};

module.exports = config;
