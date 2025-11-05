/** @type {import("../../../..").TConfigCaseConfig} */
module.exports = {
	findBundle: function (i, options) {
		// should split based on their file path
		return [
			"main.js",
			"fragment-src_aaa_sync_recursive_.js",
			"fragment-src_bbb_sync_recursive_.js",
			"fragment-src_index_js.js"
		];
	}
};
