/** @type {import("../../../../src/index").RspackOptions} */
module.exports = [
	{
		entry: "./a",
		target: "web",
		output: {
			filename: "a.js",
			scriptType: "module"
		}
	},
	{
		entry: "./index"
	}
];
