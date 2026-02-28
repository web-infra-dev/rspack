/** @type {import("../../../..").TConfigCaseConfig} */
module.exports = {
	findBundle: function (i, options) {
		return [
			"A.css",
			"shared.css",
			"main.js",
			"A.js",
			"shared.js",
			"B.js",
			"B-2.js"
		];
	}
};
