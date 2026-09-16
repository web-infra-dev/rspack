module.exports = function missingRuntimeModule() {
	throw new Error("Cannot find module './missing-runtime'");
};
