const path = require("path");
const readDir = require("./readdir");
const { rspack } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = (_, { testPath }) => {
	return {
		output: {
			clean: true
		},
		plugins: [
			new rspack.DllPlugin({
				name: "[name]_dll",
				path: path.resolve(testPath, "manifest.json")
			}),
			compiler => {
				compiler.hooks.afterEmit.tap("Test", compilation => {
					const outputPath = compilation.getPath(compiler.outputPath, {});
					expect(readDir(outputPath)).toMatchInlineSnapshot(`
						Object {
						  directories: Array [],
						  files: Array [
						    bundle0.js,
						    manifest.json,
						  ],
						}
					`);
				});
			}
		]
	};
};
