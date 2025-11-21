const { createHotNormalCase, describeByWalk } = require("@rspack/test-tools");
const path = require("path");

describeByWalk(__filename, (name, src, dist) => {
	createHotNormalCase(name, src, dist);
}, {
	source: path.resolve(__dirname, "./normalCases"),
	dist: path.resolve(__dirname, `./js/normal-hot`),
	exclude: [
		/esm-commonjs-mix-decorator/
	]
});
