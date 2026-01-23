const path = require("path");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	devtool: "source-map",
	module: {
		rules: [
			{
				test: /lib[\\/]index\.js$/,
				extractSourceMap: true,
			}
		]
	},
	plugins: [
		{
			apply(compiler) {
				compiler.hooks.afterCompile.tap("Test", compilation => {
					for (const file of compilation.fileDependencies) {
						// Should already normalized (do not contains '../' like '/extract-source-map-path-normalization/lib/../lib/src/foo.js')
						// In the initial start Watchpack will add the fileDependencies to watch list, and then
						// do a scan (DirectoryWatcher.doScan) to find the actual files that exists in the fs,
						// and use the file path (**which is already normalized by nodejs path.join()**), if the
						// fileDependencies' path is not normalized, then file path miss match, watchpack will
						// emit a "initial-missing" event to trigger a rebuild.
						expect(path.normalize(file)).toBe(file);
					}
				});
			}
		}
	]
};
