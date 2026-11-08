import { createDefaultsCase, describeByWalk } from "@rspack/test-tools";

describeByWalk(
	import.meta.filename,
	(name, src, dist) => {
		createDefaultsCase(name, src);
	},
	{
		type: "file"
	}
);
