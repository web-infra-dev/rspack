const path = require("path");
const { describeByWalk, createConfigNewCodeSplittingCase } = require("../dist");

function v(name) {
	return path.join(__dirname, `new-code-splitting ${name}`)
}

// Run tests rspack-test-tools/tests/configCases
describeByWalk(v("config cases"), (name, src, dist) => {
	createConfigNewCodeSplittingCase(name, src, dist);
}, {
	source: path.resolve(__dirname, "./configCases"),
	dist: path.resolve(__dirname, `./js/new-code-splitting-config`),
});

// Run tests webpack-tests/configCases
describeByWalk(v("config cases (webpack test)"), (name, src, dist) => {
	createConfigNewCodeSplittingCase(name, src, dist);
}, {
	source: path.resolve(__dirname, "../../../tests/webpack-test/configCases"),
	dist: path.resolve(__dirname, `./js/new-code-splitting-webpack-config`),
	exclude: [/disabled/]
});
