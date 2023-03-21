/** @type {import("../../../../src/index").RspackOptions} */
module.exports = {
	context: __dirname,
	entry: {
		main: "./index"
	},
	node: {
		__dirname: "mock",
		__filename: "mock"
	}
};
