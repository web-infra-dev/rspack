/** @type {import("../../..").TDiffCaseConfig} */
module.exports = {
	modules: false,
	// TODO: enable module_chunk_loading diff
	// close temporarily because of the latest webpack will use the public path when importing chunks
	// but this will break SSR which needs to use the dist directory in chunk loading but public path in runtime
	// runtimeModules: ["webpack/runtime/module_chunk_loading"]
	runtimeModules: ["webpack/runtime/public_path"]
};
