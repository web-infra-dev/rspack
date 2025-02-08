/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		a: "./a.js",
		b: "./b.cjs",
		c: "./c.js",
		d: "./d.mjs",
		e: "./e/index.js",
		f: "./f/index.js",
		g: "./g/index.js",
		h: "./h/file.png"
	},
	module: {
		rules: [
			{
				test: /\.png$/,
				type: "asset/resource"
			}
		]
	},
	externals: {
		path: "node-commonjs path"
	},
	output: {
		filename: `[name].js`,
		module: true,
		libraryTarget: "modern-module",
		iife: false,
		chunkFormat: "module"
	},
	experiments: {
		outputModule: true
	},
	optimization: {
		concatenateModules: true,
		avoidEntryIife: true,
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
					expect(assets["a.js"]._value).toMatchSnapshot(
						"ESM export should concat"
					);
					expect(assets["b.js"]._value).toMatchSnapshot(".cjs should bail out");
					expect(assets["c.js"]._value).toMatchSnapshot(
						"unambiguous should bail out"
					);
					expect(assets["d.js"]._value).toMatchSnapshot(".mjs should concat");
					expect(assets["e.js"]._value).toMatchSnapshot(
						".cjs should bail out when bundling"
					);
					expect(assets["f.js"]._value).toMatchSnapshot(
						"external module should bail out when bundling"
					);
					expect(assets["g.js"]._value).toMatchSnapshot(
						"harmony export should concat, even with bailout reason"
					);
					expect(assets["h.js"]._value).toMatchSnapshot(
						"asset as entry should not be concatenated"
					);
				});
			};
			this.hooks.compilation.tap("testcase", handler);
		}
	]
};
