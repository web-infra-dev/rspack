import { createTreeShakingCase, describeByWalk } from "@rspack/test-tools";

describeByWalk(
	import.meta.filename,
	(name, src, dist) => {
		createTreeShakingCase(name, src, dist);
	},
	{
		level: 1
	}
);
