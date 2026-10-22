/** @type {import('@rspack/test-tools').TConfigCaseConfig} */
export default {
	findBundle: function (i, options) {
		return i === 0 ? "./main.js" : "./module/main.mjs";
	}
};
