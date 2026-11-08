import { createHashCase, describeByWalk } from "@rspack/test-tools";

describeByWalk(
	import.meta.filename,
	(name, src, dist) => {
		createHashCase(name, src, dist);
	},
	{
		level: 1,
		absoluteDist: false
	}
);
