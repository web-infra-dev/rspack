/** @type {import("@rspack/coredist").TConfigCaseConfig} */
module.exports = {
	findBundle: (i, options) => {
		return ["a.js", "b.js"];
	}
};
