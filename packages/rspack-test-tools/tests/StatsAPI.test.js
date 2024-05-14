const { createStatsAPICase, describeByWalk } = require("..");
const srcDir = __dirname;

describeByWalk(__filename, (name, testConfig, dist) => {
	createStatsAPICase(name, srcDir, "none", testConfig);
}, {
	absoluteDist: false,
	level: 1,
	type: "file",
});

