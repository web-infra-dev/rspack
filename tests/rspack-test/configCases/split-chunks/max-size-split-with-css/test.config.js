/** @type {import('@rspack/test-tools').TConfigCaseConfig} */
module.exports = {
	findBundle: function (i, options) {
		// should split based on their file path
		return [
			// the total css size are not satisfied by minSize, so the css modules
			// are split and try again to see if the reset size satisfied the minSize
			// then its okay, so the js can be split
			"fragment-src_aaa_index_js.js",
			"fragment-src_aaa_small_css-src_bbb_index_js.js",
			"fragment-src_aaa_small_css-src_small_css.css",
			"fragment-src_ccc_50k-1_js-src_ccc_50k-2_js-src_ccc_50k-3_js-src_ccc_50k-4_js.js",
			"fragment-src_index_js.js",
			"main.js"
		];
	}
};
