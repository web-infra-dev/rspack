"use strict";

module.exports = {
	moduleScope(scope, stats, options) {
	},
	findBundle(i) {
		return i === 0 ? [
			// "dynamic_css.bundle0.js",
			"bundle0.css",
			"bundle0.js"
		] : [
			"bundle1.css",
			"bundle1.mjs"
		];
	}
};
