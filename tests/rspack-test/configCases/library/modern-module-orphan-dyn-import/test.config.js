/** @type {import('@rspack/test-tools').TConfigCaseConfig} */
module.exports = {
	findBundle: (i, options) => {
		return ["index.mjs"];
	}
};
