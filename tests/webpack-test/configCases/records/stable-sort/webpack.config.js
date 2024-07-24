var path = require("path");

/** @type {function(any, any): import("@rspack/core").Configuration} */
module.exports = (env, { testPath }) => ({
	mode: "development",
	entry: "./test",
	recordsOutputPath: path.resolve(testPath, "records.json"),
	optimization: {
		chunkIds: "size"
	},
	target: "node",
	node: {
		__dirname: false
	}
});
