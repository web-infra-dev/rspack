import path from "node:path";
import { createMultiCompilerCase, describeByWalk } from "@rspack/test-tools";
const srcDir = path.resolve(import.meta.dirname, "./fixtures");

describeByWalk(
	import.meta.filename,
	(name, testConfig, dist) => {
		createMultiCompilerCase(name, srcDir, dist, testConfig);
	},
	{
		level: 1,
		type: "file"
	}
);
