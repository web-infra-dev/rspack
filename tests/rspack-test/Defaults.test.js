const { createDefaultsCase, describeByWalk } = require("@rspack/test-tools");

describeByWalk(
	__filename,
	(name, src, dist) => {
		createDefaultsCase(name, src);
	},
	{
		type: "file"
	}
);
