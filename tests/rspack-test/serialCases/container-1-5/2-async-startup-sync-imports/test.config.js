/** @type {import('@rspack/test-tools').TConfigCaseConfig} */
module.exports = {
	findBundle: function (i, options) {
		if (i === 0) return "./main.js";
		if (i === 1) return "./module/main.mjs";
		return "./node/main.cjs";
	}
};
