const {
	experiments: { RslibPlugin, EsmLibraryPlugin }
} = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
const baseConfig = {
	entry: {
		index: "./index.js"
	},
	target: "node",
	node: {
		__filename: false,
		__dirname: false
	}
};

module.exports = [
	// CJS output
	{
		...baseConfig,
		output: {
			library: {
				type: "commonjs"
			}
		},
		plugins: [new RslibPlugin()]
	},
	// ESM output (without EsmLibraryPlugin)
	{
		...baseConfig,
		experiments: {
			outputModule: true
		},
		externals: {
			os: "module os"
		},
		output: {
			module: true,
			library: {
				type: "modern-module"
			}
		},
		plugins: [new RslibPlugin()]
	},
	// ESM output (with EsmLibraryPlugin)
	{
		...baseConfig,
		experiments: {
			outputModule: true
		},
		externals: {
			os: "module os"
		},
		output: {
			module: true,
			library: {
				type: "modern-module"
			}
		},
		plugins: [new RslibPlugin(), new EsmLibraryPlugin()]
	},
	// Test entry
	{
		entry: "./test.js",
		target: "node",
		node: {
			__filename: false,
			__dirname: false
		}
	}
];
