/** @type {import('@rspack/test-tools').TConfigCaseConfig} */
module.exports = {
	findBundle: function (i, options) {
		return i === 0 ? "./main.js" : "./module/main.mjs";
	}
};
