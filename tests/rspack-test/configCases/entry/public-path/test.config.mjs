/** @type {import("../../../..").TConfigCaseConfig} */
export default {
	findBundle: function () {
		return Array.from({ length: 10 }, (_, i) => `./bundle${i + 1}.js`);
	}
};
