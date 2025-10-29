const { createProdNormalCase, describeByWalk } = require("@rspack/test-tools");
const path = require("path");

describeByWalk(__filename, (name, src, dist) => {
	createProdNormalCase(name, src, dist);
}, {
	source: path.resolve(__dirname, "./normalCases"),
	dist: path.resolve(__dirname, `./js/normal-prod`),
	// FIXME: these cases throw errors in production
	exclude: [
		/parsing\/resolve-weak-context/,
		/parsing\/issue-7519/,
		/parsing\/api/,
		/mjs\/cjs-import-default/,
		/inner-graph\/simple/
	]
});
