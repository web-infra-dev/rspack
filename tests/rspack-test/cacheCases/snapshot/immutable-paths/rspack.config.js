const path = require("path");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	context: __dirname,
	cache: {
		type: "persistent",
		snapshot: {
			immutablePaths: [path.join(__dirname, "./file.js")]
		}
	}
};
