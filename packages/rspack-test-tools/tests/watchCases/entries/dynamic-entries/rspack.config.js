const path = require("path");
const fs = require("fs");

let compiler;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: async () => {
		const context = compiler.context;
		const files = await fs.promises.readdir(context);
		let entries = files.filter(f => f.startsWith("index"));
		entries.sort();
		return entries.reduce((acc, e, i) => {
			acc[`bundle${i}`] = path.resolve(context, e);
			return acc;
		}, {});
	},
	output: {
		filename: "[name].js"
	},
	plugins: [
		function (c) {
			compiler = c;
		}
	]
};
