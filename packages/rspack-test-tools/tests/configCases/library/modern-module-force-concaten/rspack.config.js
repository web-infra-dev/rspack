/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		"a": "./a.js",
		"b": "./b.cjs",
		"c": "./c.js",
		"d": "./d.mjs"
	},
	output: {
		filename: `[name].js`,
		module: true,
		libraryTarget: "modern-module",
		iife: false,
		chunkFormat: "module",
	},
	externalsType: "module",
  externals: [
    (data, callback) => {
      if (data.contextInfo.issuer) {
        return callback(null, data.request)
      }
      callback()
    },
  ],
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
					expect(assets['a.js']._value).toMatchSnapshot();
					expect(assets['b.js']._value).toMatchSnapshot();
					expect(assets['c.js']._value).toMatchSnapshot();
					expect(assets['d.js']._value).toMatchSnapshot();
				});
			};
			this.hooks.compilation.tap("testcase", handler);
		}
	]
};
