import { createStatsAPICase, describeByWalk } from "@rspack/test-tools";
const srcDir = import.meta.dirname;

describeByWalk(
	import.meta.filename,
	(name, testConfig, dist) => {
		createStatsAPICase(name, srcDir, "none", testConfig);
	},
	{
		absoluteDist: false,
		level: 1,
		type: "file"
	}
);
