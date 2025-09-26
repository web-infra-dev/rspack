/** @type {import("@rspack/core").Configuration} */
module.exports = [
	{
		entry: "./module1.js",
		target: "node",
		node: {
			__filename: false,
			__dirname: false
		},
		output: {
			library: {
				type: "commonjs"
			}
		},
		module: {
			parser: {
				javascript: {
					commonjs: {
						exports: "skipInEsm"
					}
				}
			}
		},
	},
	{
		entry: "./module2.js",
		target: "node",
		node: {
			__filename: false,
			__dirname: false
		},
		externals: {
			'node:module': 'module node:module'
		},
		externalsPresets: {
			node: false
		},
		output: {
			iife: false,
			module: true,
			library: {
				type: "modern-module"
			}
		},
		optimization: {
			avoidEntryIife: true,
		},
		experiments: {
			outputModule: true
		},
		module: {
			parser: {
				javascript: {
					commonjs: {
						exports: "skipInEsm"
					}
				}
			}
		},
	},
	{
		entry: "./test.js",
		externals: {
			"./bundle0.js": "commonjs ./bundle0.js",
			"./bundle1.mjs": "commonjs ./bundle1.mjs"
		},
		output: {
			filename: 'test.js'
		},
		target: "node",
		node: {
			__filename: false,
			__dirname: false
		}
	}
];
