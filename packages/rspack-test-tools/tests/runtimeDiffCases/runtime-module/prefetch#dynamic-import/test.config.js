/** @type {import("../../..").TDiffCaseConfig} */
module.exports = {
	modules: false,
	runtimeModules: [
		"webpack/runtime/chunk_prefetch_startup",
		"webpack/runtime/chunk_prefetch_trigger",
		"webpack/runtime/chunk_prefetch_function/prefetch"
	]
};
