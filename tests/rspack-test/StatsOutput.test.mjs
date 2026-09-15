import { createStatsOutputCase, describeByWalk } from "@rspack/test-tools";

describeByWalk(
	import.meta.filename,
	(name, src, dist) => {
		createStatsOutputCase(name, src, dist);
	},
	{
		level: 1
	}
);
