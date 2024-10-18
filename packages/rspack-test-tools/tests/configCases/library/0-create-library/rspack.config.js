/** @type {function(any, any): import("@rspack/core").Configuration[]} */
module.exports = (env, { testPath }) => [
	{
		entry: {
			"a": "./modern-module-force-concatenation/a.js",
			"b": "./modern-module-force-concatenation/b.cjs",
			"c": "./modern-module-force-concatenation/c.js",
			"d": "./modern-module-force-concatenation/d.mjs",
			"e": "./modern-module-force-concatenation/e/index.js",
			"f": "./modern-module-force-concatenation/f/index.js",
			"g": "./modern-module-force-concatenation/g/index.js"
		},
		externals: {
			path: 'node-commonjs path',
		},
		output: {
			filename: `modern-module-force-concatenation/[name].js`,
			module: true,
			iife: false,
			chunkFormat: "module",
			library: {
				type: 'modern-module',
			},
		},
		experiments: {
			outputModule: true
		},
		optimization: {
			concatenateModules: true,
			minimize: false
		},
		plugins: [
			function () {
				/**
				 * @param {import("@rspack/core").Compilation} compilation compilation
				 * @returns {void}
				 */
				const handler = compilation => {
					compilation.hooks.afterProcessAssets.tap("testcase", assets => {
						expect(assets['modern-module-force-concatenation/a.js']._value).toMatchSnapshot("harmony export should concat");
						expect(assets['modern-module-force-concatenation/b.js']._value).toMatchSnapshot(".cjs should bail out");
						expect(assets['modern-module-force-concatenation/c.js']._value).toMatchSnapshot("unambiguous should bail out");
						expect(assets['modern-module-force-concatenation/d.js']._value).toMatchSnapshot(".mjs should concat");
						expect(assets['modern-module-force-concatenation/e.js']._value).toMatchSnapshot(".cjs should bail out when bundling");
						expect(assets['modern-module-force-concatenation/f.js']._value).toMatchSnapshot("external module should bail out when bundling");
						expect(assets['modern-module-force-concatenation/g.js']._value).toMatchSnapshot("harmony export should concat, even with bailout reason");
					});
				};
				this.hooks.compilation.tap("testcase", handler);
			}
		]
	},
	{
		entry: {
			main: './modern-module-non-entry-module-export/index.js',
		},
		output: {
			module: true,
			chunkFormat: "module",
			filename: "modern-module-non-entry-module-export/[name].js",
			library: {
				type: 'modern-module',
			},
		},
		optimization: {
			concatenateModules: true,
		},
		experiments: {
			outputModule: true,
		},
	},
];
