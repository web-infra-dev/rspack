const { createProdNormalCase, describeByWalk } = require("@rspack/test-tools");
const path = require("path");

describeByWalk(__filename, (name, src, dist) => {
	createProdNormalCase(name, src, dist);
}, {
	source: path.resolve(__dirname, "./normalCases"),
	dist: path.resolve(__dirname, `./js/normal-prod`),
});
