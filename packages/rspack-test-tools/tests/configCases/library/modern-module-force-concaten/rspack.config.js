const path = require("path");

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
					expect(assets["a.js"]._value).toMatchFileSnapshot(
						path.join(__dirname, "__snapshot__", "a.js.txt"),
						"ESM export should concat"
					);
					expect(assets["b.js"]._value).toMatchFileSnapshot(
						path.join(__dirname, "__snapshot__", "b.js.txt"),
						".cjs should bail out"
					);
					expect(assets["c.js"]._value).toMatchFileSnapshot(
						path.join(__dirname, "__snapshot__", "c.js.txt"),
						"unambiguous should bail out"
					);
					expect(assets["d.js"]._value).toMatchFileSnapshot(
						path.join(__dirname, "__snapshot__", "d.js.txt"),
						".mjs should concat"
					);
					expect(assets["e.js"]._value).toMatchFileSnapshot(
						path.join(__dirname, "__snapshot__", "e.js.txt"),
						".cjs should bail out when bundling"
					);
					expect(assets["f.js"]._value).toMatchFileSnapshot(
						path.join(__dirname, "__snapshot__", "f.js.txt"),
						"external module should bail out when bundling"
					);
					expect(assets["g.js"]._value).toMatchFileSnapshot(
						path.join(__dirname, "__snapshot__", "g.js.txt"),
						"harmony export should concat, even with bailout reason"
					);
					expect(assets["h.js"]._value).toMatchFileSnapshot(
						path.join(__dirname, "__snapshot__", "h.js.txt"),
						"asset as entry should not be concatenated"
					);
				});
			};
			this.hooks.compilation.tap("testcase", handler);
		}
	]
};
