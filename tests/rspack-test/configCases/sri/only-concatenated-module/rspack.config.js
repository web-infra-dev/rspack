const { rspack } = require("@rspack/core");
const path = require("path");
const fs = require("fs");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: "web",
	externals: {
		path: "require('path')",
		fs: "require('fs')"
	},
	node: {
		__dirname: false
	},
	output: {
		crossOriginLoading: "anonymous"
	},
	optimization: {
		concatenateModules: true,
		minimize: false,
		chunkIds: "named",
		moduleIds: "named",
	},
	plugins: [
		new rspack.experiments.SubresourceIntegrityPlugin(),
		{
			apply(compiler) {
				compiler.hooks.done.tap('TestPlugin', () => {
					const mainPath = path.join(compiler.options.output.path, "bundle0.js");
					const mainContent = fs.readFileSync(mainPath, "utf-8");
					expect(mainContent).toContain('.sriHashes = {"chunk": "sha384-');
				});
			}
		}
	]
};
