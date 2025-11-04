/** @type {import("../../../..").TConfigCaseConfig} */
module.exports = {
	findBundle: function (i, options) {
		// should split based on their file path
		return [
			"main.js",
			"fragment-src_aaa_50k-1_js-src_aaa_50k-2_js-src_aaa_50k-3_js-src_aaa_50k-4_js.js",
			"fragment-src_aaa_sync_recursive_-src_bbb_50k-1_js-src_bbb_50k-2_js-src_bbb_50k-3_js-src_bbb_5-8391f9.js",
			"fragment-src_index_js.js"
		];
	}
};
