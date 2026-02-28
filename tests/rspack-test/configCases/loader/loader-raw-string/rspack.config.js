const path = require("path");
const file = path.resolve(__dirname, "lib.js");
const createUse = loaders => loaders.map(l => ({ loader: l }));
/** @type {import("@rspack/core").Configuration} */
module.exports = {
	context: __dirname,
	module: {
		rules: [
			{
				test: file,
				resourceQuery: /case-1/,
				use: createUse(["./raw", "./string", "./raw"])
			},
			{
				test: file,
				resourceQuery: /case-2/,
				use: createUse(["./string", "./raw", "./string"])
			}
		]
	}
};
