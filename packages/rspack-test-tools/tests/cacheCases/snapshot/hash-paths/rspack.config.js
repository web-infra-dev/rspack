const path = require("path");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	context: __dirname,
	experiments: {
		cache: {
			type: "persistent",
			snapshot: {
				hashPaths: [path.join(__dirname, "./file.js")]
			}
		}
	}
};
