var path = require("path");

/** @type {function(any, any): import("@rspack/core").Configuration} */
module.exports = (env, { testPath }) => ({
	entry: "./test",
	recordsOutputPath: path.resolve(testPath, "records.json"),
	target: "node",
	node: {
		__dirname: false
	}
});
