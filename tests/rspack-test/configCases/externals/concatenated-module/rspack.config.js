/** @type {(variant: boolean) => import("@rspack/core").Configuration} */
const config = o => ({
	externals: {
		"module-fs": o ? "module fs" : "module fs/promises",
		fs: o ? "node-commonjs fs" : "node-commonjs fs/promises",
		"module-fs-promises": o ? ["module fs", "promises"] : "module fs/promises",
		"fs-promises": o
			? ["node-commonjs fs", "promises"]
			: "node-commonjs fs/promises",
		"module-path": "module path",
		path: "node-commonjs path",
		"module-import-url": "module-import url",
		url: "node-commonjs url"
	},
	output: {
		module: true,
	},
	optimization: {
		concatenateModules: true,
		usedExports: true,
		providedExports: true,
		mangleExports: true
	},
	target: "node14",
});

module.exports = [config(false), config(true)];
