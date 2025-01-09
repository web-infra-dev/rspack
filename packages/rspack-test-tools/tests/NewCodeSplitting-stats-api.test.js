const path = require("path");
const { describeByWalk, createStatsAPINewCodeSplittingCase } = require("..");

describeByWalk('new code splitting stats api', (name, testConfig, dist) => {
	createStatsAPINewCodeSplittingCase(name, __dirname, "none", testConfig);
}, {
	absoluteDist: false,
	level: 1,
	type: "file",
	source: path.resolve(__dirname, './statsApiCases'),
	dist: path.resolve(__dirname, `./js/new-code-splitting-stats-api`),
});
