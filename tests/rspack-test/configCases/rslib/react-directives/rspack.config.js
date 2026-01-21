const {
	experiments: { RslibPlugin }
} = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
const baseConfig = (index, mjs = false) => ({
	entry: {
		index: {
			import: './index.js',
			filename: `bundle${index}${mjs? '.mjs' : '.js'}`
		}
	},
	target: "node",
	node: {
		__filename: false,
		__dirname: false
	}
});

module.exports = [
	// CJS output
	{
		...baseConfig(0),
		externals: {
			react: "react"
		},
		output: {
			library: {
				type: "commonjs"
			}
		},
		plugins: [new RslibPlugin()]
	},
	// ESM output (without EsmLibraryPlugin)
	{
		...baseConfig(1, true),
		experiments: {
			outputModule: true
		},
		externals: {
			react: "module react"
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
		...baseConfig(2, true),
		experiments: {
			outputModule: true
		},
		externals: {
			react: "module react"
		},
		output: {
			module: true,
			library: {
				type: "modern-module"
			}
		},
		plugins: [new RslibPlugin()]
	},
	// Test entry
	{
		entry: "./test.js",
		target: "node",
		output: {
			filename: 'bundle3.js'
		},
		node: {
			__filename: false,
			__dirname: false
		}
	}
];
