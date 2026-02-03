/** @type {import("@rspack/core").Configuration} */
module.exports = [
	{
		name: 'module1',
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
		name: 'module2',
		entry: {bundle1: "./module2.js"},
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
			module: true,
			library: {
				type: "modern-module"
			}
		},
		optimization: {
			// minimize: false
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
		name: 'test',
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
	},
];
