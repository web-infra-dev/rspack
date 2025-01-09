const path = require("path");
const { describeByWalk, createConfigNewCodeSplittingCase } = require("..");

function v(name) {
	return path.join(__dirname, `new-code-splitting ${name}`)
}

// Run tests rspack-test-tools/tests/configCases
describeByWalk("new-code-splitting config cases", (name, src, dist) => {
	createConfigNewCodeSplittingCase(name, src, dist);
}, {
	source: path.resolve(__dirname, "./configCases"),
	dist: path.resolve(__dirname, `./js/new-code-splitting-config`),
});
