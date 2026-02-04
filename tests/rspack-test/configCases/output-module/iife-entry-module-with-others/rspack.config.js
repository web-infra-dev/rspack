/** @type {import("@rspack/core").Configuration} */
const base = {
	optimization: {
		concatenateModules: true
	},
	target: "es2020"
};

/** @type {import("@rspack/core").Configuration[]} */
module.exports = [
	{
		...base,
		name: "module-avoidEntryIife-false",
		output: {
			module: true,
			filename: "module-avoidEntryIife-false.mjs"
		},
		optimization: {
			...base.optimization
		}
	},
	{
		...base,
		name: "module-avoidEntryIife-true",
		output: {
			module: true,
			filename: "module-avoidEntryIife-true.mjs"
		},
		optimization: {
			...base.optimization,
			avoidEntryIife: true
		}
	},
	{
		name: "test-output",
		entry: "./test.js",
		output: {
			filename: "test.js"
		}
	}
];
