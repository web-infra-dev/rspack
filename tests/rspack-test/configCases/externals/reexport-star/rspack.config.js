/** @type {import("@rspack/core").Configuration} */
module.exports = [
	{
		entry: {
			case1: "./case1.js",
			case2: "./case2.js",
			case3: "./case3/index.js",
			case4: "./case4.js",
			case5: "./case5.js",
			case6: "./case6.js",
			index: "./index.js"
		},
		output: {
			module: true,
			filename: "[name].mjs",
			chunkFormat: "module",
			library: {
				type: "modern-module"
			}
		},
		externals: {
			external1: "module external1-alias",
			external2: "module-import external2-alias"
		},
		optimization: {
			avoidEntryIife: true
		}
	}
];
