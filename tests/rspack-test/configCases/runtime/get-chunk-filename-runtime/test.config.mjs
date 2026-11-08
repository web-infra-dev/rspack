/** @type {import("@rspack/coredist").TConfigCaseConfig} */
export default {
	findBundle: (i, options) => {
		return ["a.js", "b.js"];
	}
};
