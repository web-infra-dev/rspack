/** @type {import("../../..").TDiffCaseConfig} */
module.exports = {
	modules: false,
	runtimeModules: [
		"webpack/runtime/chunk_preload_trigger",
		"webpack/runtime/chunk_prefetch_function/preload"
	]
};
