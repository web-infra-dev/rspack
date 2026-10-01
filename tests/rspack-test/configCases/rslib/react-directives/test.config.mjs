/** @type {import("../../../..").TConfigCaseConfig} */
export default {
	findBundle: function (i, options) {
		if (i === 3) {
			return ["./bundle3.js"];
		}
	}
};
