// Export promises that resolve to the bootstrap module exports
export const testExport = import("./bootstrap").then(
	module => module.testExport
);
export default import("./bootstrap").then(module => module.default);
