const path = require("path");
const fs = require("fs");
const rspack = require("@rspack/core");

let compiler;
let step = 0;
let entries;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: async () => {
		const context = compiler.context;
		if (step === 0) {
			const files = await fs.promises.readdir(context);
			entries = files.filter(f => f.startsWith("index"));
			entries.sort();
		} else if (step === 1) {
			entries.pop();
		} else {
			throw new Error(`unreachable step: ${step}`);
		}
		return entries.reduce((acc, e, i) => {
			acc[`bundle${i}`] = path.resolve(context, e);
			return acc;
		}, {});
	},
	output: {
		filename: "[name].js"
	},
	plugins: [
		new rspack.experiments.RemoveDuplicateModulesPlugin(),
		function (c) {
			compiler = c;
			c.hooks.done.tap("test", () => {
				step += 1;
			});
		}
	]
};
